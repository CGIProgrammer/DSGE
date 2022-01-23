
use crate::mesh::{Mesh, MeshRef, MeshBinder};
use crate::references::*;
use crate::framebuffer::*;
use crate::shader::*;
use crate::texture::*;
use std::collections::HashMap;

use std::sync::Arc;
use vulkano::device::{Queue, Device};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer};
use vulkano::render_pass::{RenderPassDesc, SubpassDesc, RenderPass, AttachmentDesc, Subpass};

type StageIndex = u16;
type StageInputIndex = u32;
type StageOutputIndex = u64;

mod accumulator_test;

/// Выход ноды постобработки.
/// Задаётся:
///   1. уникальным номером ноды `render_stage_id`
///   2. номером выхода ноды
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct RenderStageOutputSocket
{
    render_stage_id: StageIndex,
    output: StageOutputIndex
}

/// Вход ноды постобработки.
/// Задаётся:
///   1. уникальным номером ноды `render_stage_id`
///   2. номером входа ноды
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct RenderStageInputSocket
{
    render_stage_id: StageIndex,
    input:  StageInputIndex,
}

/// Связь нод постобработки
#[derive(Clone, Hash, PartialEq, Eq, Debug)]
struct RenderStageLink
{
    _from: RenderStageOutputSocket,
    _to: RenderStageInputSocket
}

/// Нода (она же стадия) постобработки
#[derive(Clone)]
struct RenderStage
{
    _id: StageIndex,
    _program: ShaderProgramRef,
    _resolution: (u16, u16),
    _outputs: Vec<(Option<TextureRef>, TexturePixelFormat)>,
    _executed: bool,
    _render_pass: Arc<RenderPass>
}

impl RenderStage
{
    /// Возвращает true, если нода помечена как выполненная
    fn executed(&self) -> bool
    {
        self._executed
    }

    /// Пометить ноду выполненной
    fn mark_executed(&mut self)
    {
        self._executed = true;
    }

    /// Сбросить статус выполнения ноды
    fn reset(&mut self)
    {
        self._executed = false;
    }

    /// Проверяет назначен ли выходу ноды накопительный буфер
    fn is_output_acc(&self, output: StageOutputIndex) -> bool
    {
        if (output as usize) < (self._outputs.len() as usize) {
            unsafe { self._outputs.get_unchecked(output as usize).0.is_some() }
        } else {
            false
        }
    }

    /// Возвращает накопительный буфер
    fn get_accumulator_buffer(&self, output: StageOutputIndex) -> TextureRef
    {
        self._outputs.get(output as usize).unwrap().0.as_ref().unwrap().clone()
        //self._accumulators.get(&output).unwrap().clone()
    }

    /// Меняет накопительный буфер на `new_buff` и возвращает предыдущий
    fn swap_accumulator_buffer(&mut self, output: StageOutputIndex, new_buff: &TextureRef) -> TextureRef
    {
        let output = self._outputs.get_mut(output as usize).unwrap();
        let texture = output.0.clone().unwrap();
        output.0 = Some(new_buff.clone());
        texture
    }
}

/// Граф постобработки.
/// Весь процесс рендеринга, кроме geometry pass может выполняться здесь.
/// Работает по принципу создания нод с фильтрами в виде шейдеров и соединения
/// их между собой.
/// Поддерживаются петли с применением накопительных буферов.
/// Память под буферы выделяется автоматически по мере необходимости.
/// Для перед вызовом функции выполнения, следует назначить входные
/// и выходные изображения.
pub struct RenderPostprocessingGraph
{
    _render_stage_id_counter: StageIndex,       // Счётчик ID
    _stages: HashMap<StageIndex, RenderStage>,  // Ноды
    _links: Vec<RenderStageLink>,               // Связи
    _buffers: Vec<TextureRef>,                  // Буферы для нод
    _busy_buffers: HashMap<RenderStageLink, TextureRef>,    // Занятые буферы
    _inputs: HashMap<RenderStageInputSocket, TextureRef>,   // Входящие текстуры
    _outputs: HashMap<StageInputIndex, TextureRef>,         // Текстуры на выходе
    _framebuffer : FramebufferRef,              // Буфер кадра
    _screen_plane : MeshRef,                    // Плоскость для вывода изображений
    _device : Arc<Device>,
    _queue : Arc<Queue>
}

use vulkano::image::{ImageLayout, SampleCount};

impl RenderPostprocessingGraph
{
    pub fn new(queue: Arc<Queue>, width: u16, height: u16) -> Self
    {
        let device = queue.device();
        
        Self {
            _render_stage_id_counter: 1,
            _stages: HashMap::new(),
            _links: Vec::new(),
            _buffers: Vec::new(),
            _busy_buffers: HashMap::new(),
            _framebuffer: Framebuffer::new(width, height),
            _screen_plane: Mesh::make_screen_plane(device.clone()).unwrap(),
            _inputs: HashMap::new(),
            _outputs: HashMap::new(),
            _device: device.clone(),
            _queue: queue.clone(),
        }
    }

    /// Полный сброс. Нужен при изменении разрешения
    pub fn reset(&mut self)
    {
        self._render_stage_id_counter = 1;
        self._stages.clear();
        self._busy_buffers.clear();
        self._links.clear();
        self._buffers.clear();
        self._inputs.clear();
        self._outputs.clear();
    }

    /// Подать текстуру на вход узла постобработчика
    pub fn set_input(&mut self, stage: StageIndex, input: StageInputIndex, tex: &TextureRef)
    {
        self._inputs.insert(RenderStageInputSocket {render_stage_id: stage, input: input}, tex.clone());
    }

    /// Получение текстуры-выхода
    /// Input потому, что это вход для нулевой ноды, являющейся выходом дерева
    #[allow(dead_code)]
    pub fn get_output(&self, name: &StageInputIndex) -> Option<TextureRef>
    {
        let result = self._outputs.get(name);
        if result.is_some() {
            Some(result.unwrap().clone())
        } else {
            None
        }
    }

    /// Закрепление текстуры за входом нулевой ноды.
    /// Для всех входов, которым не назначены изображения создадутся новые.
    /// Если требуется выводить результат уже в существующее изображение,
    /// например swapchain-изображение, это то, что нужно.
    pub fn set_output(&mut self, name: StageInputIndex, texture: TextureRef)
    {
        self._outputs.insert(name, texture);
    }

    /// Добавление узла постобработки
    pub fn add_stage(&mut self, program: &ShaderProgramRef, outputs_decriptors: Vec<(bool, TexturePixelFormat)>, width: u16, height: u16) -> StageIndex
    {
        let n = outputs_decriptors.len();
        let device = program.take().device().clone();
        let outputs = outputs_decriptors.iter().map(
            |(accum, pix_fmt)| {
                let acc = 
                if *accum {
                    Some(Texture::new_empty_2d(
                        format!("stage_{}_{:?}_accumulator", self._render_stage_id_counter, pix_fmt).as_str(),
                        width, height, *pix_fmt, device.clone()).unwrap())
                } else {
                    None
                };
                (acc, *pix_fmt)
            }
        ).collect();

        let attachments = outputs_decriptors.iter().map(
            |(_, pix_fmt)| {
                AttachmentDesc {
                    format: pix_fmt.vk_format(),
                    samples: SampleCount::Sample1,
                    load: vulkano::render_pass::LoadOp::DontCare,
                    store: vulkano::render_pass::StoreOp::Store,
                    stencil_load: vulkano::render_pass::LoadOp::Clear,
                    stencil_store: vulkano::render_pass::StoreOp::Store,
                    initial_layout: ImageLayout::ColorAttachmentOptimal,
                    final_layout: ImageLayout::ColorAttachmentOptimal,
                }
            }
        ).collect();

        let subpass_attachments = (0..n).map(|i|
            {
                (i, ImageLayout::ColorAttachmentOptimal)
            }
        ).collect();

        let render_pass_desc = RenderPassDesc::new(
            attachments,
            vec![SubpassDesc {
                color_attachments: subpass_attachments,
                depth_stencil: None,
                input_attachments: vec![],
                resolve_attachments: vec![],
                preserve_attachments: vec![],
            }],
            vec![]
        );

        let render_stage = RenderStage { 
            _id: self._render_stage_id_counter,
            _program: program.clone(),
            _render_pass: RenderPass::new(device.clone(), render_pass_desc).unwrap(),
            _resolution: (width, height),
            _outputs: outputs,
            _executed: false
        };
        let rsid = self._render_stage_id_counter;
        self._stages.insert(rsid, render_stage);
        self._render_stage_id_counter += 1;
        rsid
    }

    /// Добавить связь между узлами.
    /// Связь может быть циклической только если зацикленному выходу узла явно указано
    /// использовать накопительный буфер
    pub fn link_stages(&mut self, from: StageIndex, output: StageOutputIndex, to: StageIndex, input: StageInputIndex)
    {
        let link = RenderStageLink {
            _from : RenderStageOutputSocket {
                render_stage_id: from,
                output: output
            },
            _to : RenderStageInputSocket {
                render_stage_id: to,
                input: input
            }
        };
        self._links.push(link);
    }
    
    /// Выполнить граф потобработки.
    /// TODO: сделать проверку неуказанных циклов, иначе при наличии циклов без
    /// накопительных буферов будет переполняться стек.
    #[allow(dead_code)]
    pub fn execute_graph(&mut self) -> PrimaryAutoCommandBuffer
    {
        let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
            self._device.clone(),
            self._queue.family(),
            CommandBufferUsage::OneTimeSubmit,
        ).unwrap();

        for i in &mut self._stages {
            i.1.reset();
        }
        self._busy_buffers.clear();
        for link in self._links.clone() {
            if link._to.render_stage_id == 0 {
                self.execute_stage(link._from.render_stage_id, &mut command_buffer_builder);
                break;
            }
        }
        
        self._inputs.clear();
        command_buffer_builder.end_render_pass().unwrap();
        command_buffer_builder.build().unwrap()
    }

    /// Запрос свободного изображения
    fn request_texture(&mut self, link: &RenderStageLink, pix_fmt: TexturePixelFormat) -> TextureRef
    {
        let resolution = self._stages.get(&link._from.render_stage_id).unwrap()._resolution;
        let mut texture: Option<TextureRef> = None;

        // Находим свободное изображение
        for tex in &self._buffers {
            let mut busy = false;
            self._busy_buffers.values().for_each(|busy_tex| {
                if tex.box_id()==busy_tex.box_id() {
                    busy = true;
                };
            });
            if !busy &&
                tex.take().width() == resolution.0 as u32 &&
                tex.take().height() == resolution.1 as u32 &&
                pix_fmt == *tex.take().pix_fmt()
            {
                self._busy_buffers.insert(link.clone(), tex.clone());
                texture = Some(tex.clone());
                break;
            }
        }
        let output_has_texture = link._to.render_stage_id==0 && self._outputs.contains_key(&link._to.input);

        // Если свободное изображение не найдено и выходу не назначено изображение,
        // то создаём новое изображение
        if texture.is_none() && !output_has_texture {
            let buffer_name = format!("render buffer for link from {}:{} to {}:{}",
                link._from.render_stage_id, link._from.output,
                link._to.render_stage_id, link._to.input);
            println!("Создание текстуры {} {}x{}", buffer_name, resolution.0, resolution.1);
            let mut _tex = Texture::new_empty_2d(buffer_name.as_str(), resolution.0, resolution.1, pix_fmt, self._device.clone()).unwrap();
            self._buffers.push(_tex.clone());
            texture = Some(_tex.clone());
            self._busy_buffers.insert(link.clone(), _tex.clone());
        }

        // Если выход ноды направлен на выход графа...
        if link._to.render_stage_id == 0 {
            if output_has_texture {
                // Возвращаем изображение, закреплённое за выходом, если оно назначено
                return self._outputs.get(&link._to.input).unwrap().clone();
            }
            // Назначаем его, если оно не назначено.
            self._outputs.insert(link._to.input, texture.clone().unwrap());
        }
        texture.unwrap()
    }

    /// Освобождение выделенного изображения
    fn free_texture(&mut self, link: &RenderStageLink)
    {
        self._busy_buffers.remove(link);
    }

    fn stage_by_id(&mut self, id: StageIndex) -> &RenderStage
    {
        self._stages.get(&id).unwrap()
    }

    fn replace_stage(&mut self, id: StageIndex, stage: &RenderStage)
    {
        self._stages.insert(id.clone(), stage.clone());
    }
    
    /// Выполнение ноды постобработки
    fn execute_stage(&mut self, id: StageIndex, command_buffer_builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>)
    {
        self._stages.get_mut(&id).unwrap().mark_executed();
        let links = self._links.clone();

        for link in &links {
            let st_id = link._from.render_stage_id;
            let stage = self.stage_by_id(st_id);
            if link._to.render_stage_id == id && !stage.executed() {
                self.execute_stage(st_id, command_buffer_builder);
            }
        }
        
        let mut stage = self.stage_by_id(id).clone();
        let _prog = stage._program.clone();
        
        let mut program = _prog.take_mut();

        let render_pass = stage._render_pass.clone();
        
        program.make_pipeline(Subpass::from(render_pass.clone(), 0).unwrap());
        
        program.uniform(
            &RenderResolution{
                width: stage._resolution.0 as f32,
                height: stage._resolution.1 as f32
            },
            0
        );
        
        for (RenderStageInputSocket{render_stage_id, input}, tex) in &self._inputs {
            if render_stage_id == &id {
                program.uniform(tex, *input as usize);
            }
        }

        let mut render_targets = HashMap::<StageOutputIndex, TextureRef>::new();
        for link in &links {
            if link._to.render_stage_id == id {
                let from_stage = self.stage_by_id(link._from.render_stage_id);
                if from_stage.is_output_acc(link._from.output) {
                    let acc = from_stage.get_accumulator_buffer(link._from.output);
                    program.uniform(&acc, link._to.input as _);
                } else {
                    let free_tex = self._busy_buffers.get(link).unwrap();
                    program.uniform(free_tex, link._to.input as _);
                }
            }
            if link._from.render_stage_id == id {
                if render_targets.contains_key(&link._from.output) { continue; };
                let pix_fmt = stage._outputs.get(link._from.output as usize).unwrap().1;
                let _tex = self.request_texture(link, pix_fmt);
                render_targets.insert(link._from.output, _tex.clone());
            }
        }
        drop(program);

        let mut fb = self._framebuffer.take_mut();
        fb.reset_attachments();
        for ind in 0..render_targets.len() {
            let tex = render_targets.get(&(ind as _)).unwrap();
            fb.add_color_attachment(tex.clone(), [0.0, 0.0, 0.0, 0.0].into()).unwrap();
        }
        fb.view_port(stage._resolution.0, stage._resolution.1);
        drop(fb);

        command_buffer_builder
            .bind_framebuffer(self._framebuffer.clone(), render_pass.clone()).unwrap()
            .bind_shader_program(&_prog)
            .bind_shader_uniforms(&_prog)
            .bind_mesh(&self._screen_plane);

        for output in 0..16 {
            if stage.is_output_acc(output)
            {
                let used_acc = render_targets.get(&(output as _)).unwrap();
                let mut buf_index = 0;
                for buf in &self._buffers {
                    if buf.box_id() == used_acc.box_id() {
                        break;
                    }
                    buf_index += 1;
                }
                let old_acc = &self._buffers.remove(buf_index);
                let new_acc = stage.swap_accumulator_buffer(output, old_acc);
                self._buffers.push(new_acc);
            }
        }
        
        self.replace_stage(id, &stage);

        for link in &links {
            if link._to.render_stage_id == id {
                self.free_texture(link);
            }
        }
    }

    /// Стандартный вершинный шейдер для фильтров постобработки
    fn vertex_plane_shader(&self) -> Shader
    {
        let mut shader = Shader::builder(ShaderType::Vertex, self._device.clone());
        shader
            .default_vertex_attributes()
            .uniform::<RenderResolution>("iResolution", 0)
            .output("position", AttribType::FVec2)
            .output("fragCoordWp", AttribType::FVec2)
            .output("fragCoord", AttribType::FVec2)
            .code("void main()
{
    position = v_pos.xy;
    fragCoordWp = v_pos.xy*0.5+0.5;
    fragCoord.x = fragCoordWp.x*iResolution.width;
    fragCoord.y = fragCoordWp.y*iResolution.height;
    gl_Position = vec4(v_pos.xy, 0.0, 1.0);
}");
        shader.build().unwrap();
        shader
    }
}


#[derive(Clone)]
struct RenderResolution
{
    pub width : f32,
    pub height : f32
}

impl ShaderStructUniform for RenderResolution
{
    fn structure() -> String
    {
        "{
            float width;
            float height;
        }".to_string()
    }

    fn glsl_type_name() -> String
    {
        "Resolution".to_string()
    }

    fn texture(&self) -> Option<&TextureRef>
    {
        None
    }
}