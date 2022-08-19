
use crate::mesh::{Mesh, MeshRef, MeshCommandSet};
use crate::references::*;
use crate::framebuffer::*;
use crate::shader::*;
use crate::texture::*;
use crate::types::*;
use crate::time::UniformTime;
use std::collections::HashMap;
use bytemuck::{Zeroable, Pod};
use std::sync::Arc;
use vulkano::device::{Queue, Device};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer};
use vulkano::render_pass::{RenderPassCreateInfo, SubpassDescription, RenderPass, AttachmentDescription, AttachmentReference};
use vulkano::image::{ImageLayout, SampleCount};

type StageIndex = u16;
type StageInputIndex = String;
type StageOutputIndex = u64;

mod accumulator_test;
mod rolling_hills;
mod copy;
mod fsr;
mod lighting_pass;

/// Выход ноды постобработки.
/// Задаётся:
///   1. уникальным номером ноды `render_stage_id`
///   2. номером выхода ноды
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct RenderStageOutputSocket
{
    render_stage_id: StageIndex,
    output: StageOutputIndex,
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

#[derive(Clone)]
enum RenderStageOutput
{
    Generic {
        pix_fmt: TexturePixelFormat,
        filtering: TextureFilter
    },
    Accumulator {
        buffer: Texture,
        pix_fmt: TexturePixelFormat,
        filtering: TextureFilter
    }
}

impl RenderStageOutput
{
    #[inline]
    pub fn buffer(&self) -> Option<&Texture>
    {
        match self {
            Self::Accumulator{ref buffer,..} => Some(buffer),
            _ => None
        }
    }

    #[inline]
    pub fn new_accumulator(tex: Texture) -> Self
    {
        let pix_fmt = tex.pix_fmt();
        let filtering = tex.mag_filter();
        Self::Accumulator {
            buffer: tex,
            pix_fmt: pix_fmt,
            filtering: filtering,
        }
    }

    #[inline]
    pub fn pix_fmt(&self) -> TexturePixelFormat
    {
        match self
        {
            Self::Generic {pix_fmt,..} => *pix_fmt,
            Self::Accumulator {pix_fmt,..} => *pix_fmt,
        }
    }

    #[inline]
    pub fn filtering(&self) -> TextureFilter
    {
        match self
        {
            Self::Generic {filtering,..} => *filtering,
            Self::Accumulator {filtering,..} => *filtering,
        }
    }
}

/// Нода (она же стадия) постобработки
#[derive(Clone)]
struct RenderStage
{
    _id: StageIndex,
    _program: ShaderProgramRef,
    _uniform_buffer : ShaderProgramUniformBuffer,
    _resolution: TextureDimensions,
    _outputs: Vec<RenderStageOutput>,
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
            //self._outputs.get_unchecked(output as usize).0.is_some() }
            let output = unsafe { self._outputs.get_unchecked(output as usize) };
            match output {
                RenderStageOutput::Accumulator {..} => true,
                RenderStageOutput::Generic {..} => false,
            }
        } else {
            false
        }
    }

    /// Возвращает накопительный буфер
    fn get_accumulator_buffer(&self, output: StageOutputIndex) -> &Texture
    {
        self._outputs.get(output as usize).unwrap().buffer().unwrap()
        //self._accumulators.get(&output).unwrap().clone()
    }

    /// Меняет накопительный буфер на `new_buff` и возвращает предыдущий
    fn swap_accumulator_buffer(&mut self, output: StageOutputIndex, new_buff: &Texture) -> Texture
    {
        let rs_output = self._outputs.remove(output as usize);
        let texture = rs_output.buffer().clone().unwrap();
        self._outputs.insert(output as usize, RenderStageOutput::new_accumulator(new_buff.clone()));
        texture.clone()
    }

    /*pub fn uniform<T>(&mut self, data: &T)
        where T: ShaderStructUniform + std::marker::Send + std::marker::Sync + Clone + 'static
    {
        self._program.take_mut().uniform(data, 0);
    }*/
}

pub struct RenderStageBuilder
{
    _dimenstions : TextureDimensions,
    _fragment_shader : Shader,
    _output_accum: Vec<(TexturePixelFormat, bool, TextureFilter)>,
    _inputs : u8
}

#[allow(dead_code)]
impl RenderStageBuilder
{
    pub fn dimenstions(&mut self, width: u16, height: u16) -> &mut Self
    {
        self._dimenstions = TextureDimensions::Dim2d{width: width as _, height: height as _, array_layers: 1};
        self
    }

    pub fn uniform<T: ShaderStructUniform>(&mut self, name: &str) -> &mut Self
    {
        self._fragment_shader.uniform_autoincrement::<T>(name, 0).unwrap();
        self
    }

    pub fn uniform_named_type<T: ShaderStructUniform>(&mut self, name: &str, _type: &str) -> &mut Self
    {
        self._fragment_shader.uniform_structure_autoincrement(name, _type, T::structure().as_str(), 0).unwrap();
        self
    }

    pub fn uniform_structure(&mut self, name: &str, _type: &str, structure: &str) -> &mut Self
    {
        self._fragment_shader.uniform_structure_autoincrement(name, _type, structure, 0).unwrap();
        self
    }

    pub fn input(&mut self, name: &str, dims: TextureView) -> &mut Self
    {
        self._fragment_shader.uniform_sampler(name, 1, self._inputs as _, dims).unwrap();
        //self._fragment_shader.uniform_sampler_autoincrement(name, self._inputs as usize, TextureView::Dim2d).unwrap();
        self._inputs += 1;
        self
    }

    pub fn output(&mut self, name: &str, pix_fmt: TexturePixelFormat, filtering: TextureFilter, accumulator: bool) -> &mut Self
    {
        let glsl_type = match pix_fmt.subpixels() {
            1 => AttribType::Float,
            2 => AttribType::FVec2,
            3 => AttribType::FVec3,
            4 => AttribType::FVec4,
            _ => panic!()
        };
        self._fragment_shader.output(name, glsl_type);
        self._output_accum.push((pix_fmt, accumulator, filtering));
        self
    }

    pub fn code(&mut self, code: &str) -> &mut Self
    {
        self._fragment_shader.code(code);
        self
    }

    pub fn build(mut self, pp_graph: &mut PostprocessingPass) -> Result<StageIndex, String>
    {
        let device = pp_graph._device.clone();
        let queue = pp_graph._queue.clone();
        let mut program = ShaderProgram::builder();
        let v_shader = pp_graph.vertex_plane_shader()?;
        let f_shader = self._fragment_shader.build()?;
        program
            .vertex(&v_shader).unwrap()
            .fragment(f_shader).unwrap();

        let outputs = self._output_accum.iter().map(
            |(pix_fmt, accum, filter)| {
                let acc = 
                if *accum {
                    //println!("Создание накопительного буфера {}x{}", self._dimenstions.0, self._dimenstions.1);
                    let mut buffer = Texture::new_empty(
                        format!("stage_{}_{:?}_accumulator", pp_graph._render_stage_id_counter, pix_fmt).as_str(),
                        self._dimenstions, *pix_fmt, device.clone()).unwrap();
                    buffer.clear_color(queue.clone());
                    buffer.set_vertical_address(TextureRepeatMode::ClampToEdge);
                    buffer.set_horizontal_address(TextureRepeatMode::ClampToEdge);
                    buffer.update_sampler();

                    RenderStageOutput::Accumulator {
                        pix_fmt: *pix_fmt,
                        buffer: buffer,
                        filtering: *filter
                    }
                } else {
                    RenderStageOutput::Generic {
                        pix_fmt: *pix_fmt,
                        filtering: *filter
                    }
                };
                acc
            }
        ).collect();

        let attachments = self._output_accum.iter().map(
            |(pix_fmt, _, _)| {
                AttachmentDescription {
                    format: Some(pix_fmt.vk_format()),
                    samples: SampleCount::Sample1,
                    load_op: vulkano::render_pass::LoadOp::Clear,
                    store_op: vulkano::render_pass::StoreOp::Store,
                    stencil_load_op: vulkano::render_pass::LoadOp::Clear,
                    stencil_store_op: vulkano::render_pass::StoreOp::Store,
                    initial_layout: ImageLayout::ColorAttachmentOptimal,
                    final_layout: ImageLayout::ColorAttachmentOptimal,
                    ..Default::default()
                }
            }
        ).collect();

        let n = self._output_accum.len() as u32;
        let subpass_attachments = (0..n).map(|i|
            {
                Some(AttachmentReference{attachment: i, layout: ImageLayout::ColorAttachmentOptimal, ..Default::default()})
            }
        ).collect();

        let render_pass_desc = RenderPassCreateInfo{
            attachments: attachments,
            subpasses: vec![SubpassDescription {
                color_attachments: subpass_attachments,
                depth_stencil_attachment: None,
                ..Default::default()
            }],
            ..Default::default()
        };

        //println!("{}", program.take().fragment_shader_source());
        let render_pass = RenderPass::new(device.clone(), render_pass_desc).unwrap();
        let mut program = program.build(device.clone())?;
        program.use_subpass(render_pass.clone().first_subpass());
        let uniform_buffer = program.new_uniform_buffer();
        
        let stage = RenderStage {
            _id: pp_graph._render_stage_id_counter,
            _program: RcBox::construct(program),
            _uniform_buffer: uniform_buffer,
            _resolution: self._dimenstions,
            _outputs: outputs,
            _render_pass: render_pass,
            _executed: false
        };
        let result = pp_graph._render_stage_id_counter;
        pp_graph._stages.insert(result, stage);
        pp_graph._render_stage_id_counter += 1;
        Ok(result)
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
pub struct PostprocessingPass
{
    /// Счётчик ID
    _render_stage_id_counter: StageIndex,
    /// Ноды
    _stages: HashMap<StageIndex, RenderStage>,
    /// Связи
    _links: Vec<RenderStageLink>,
    /// Буферы для нод
    _buffers: Vec<Texture>,
    /// Занятые буферы
    _busy_buffers: HashMap<RenderStageLink, Texture>,
    /// Входящие текстуры
    _image_inputs: HashMap<RenderStageInputSocket, Texture>,
    // Входящие значения (uniform-переменные)
    //_uniform_inputs: String,
    /// Текстуры на выходе
    _outputs: HashMap<StageInputIndex, Texture>,
    /// Буфер кадра
    _framebuffer : Framebuffer,
    /// Плоскость для вывода изображений
    _screen_plane : MeshRef,

    pub timer : UniformTime,
    _device : Arc<Device>,
    _queue : Arc<Queue>
}

#[allow(dead_code)]
#[derive(Clone)]
pub enum NumericInput
{
    Scalar(f32),
    Vec2(Vec2),
    Vec3(Vec3),
    Vec4(Vec4),
    Mat2(Mat2),
    Mat3(Mat3),
    Mat4(Mat4),
}

#[allow(dead_code)]
impl PostprocessingPass
{
    pub fn new(queue: Arc<Queue>, width: u16, height: u16) -> Self
    {
        let device = queue.device();
        
        PostprocessingPass {
            _render_stage_id_counter: 1,
            _stages: HashMap::new(),
            _links: Vec::new(),
            _buffers: Vec::new(),
            _busy_buffers: HashMap::new(),
            _framebuffer: Framebuffer::new(width, height),
            _screen_plane: Mesh::make_screen_plane(queue.clone()).unwrap(),
            _image_inputs: HashMap::new(),
            //_uniform_inputs: String::new(),
            timer: Default::default(),
            _outputs: HashMap::new(),
            _device: device.clone(),
            _queue: queue.clone(),
        }
    }

    pub fn uniform_to_all<T>(&mut self, name: &String, data: T)
        where T: ShaderStructUniform + std::marker::Send + std::marker::Sync + Pod + 'static
    {
        for (_, rs) in &mut self._stages {
            drop(rs._uniform_buffer.uniform_by_name(data, name));
        }
    }

    pub fn uniform_to_stage<T>(&mut self, stage_id: StageIndex, name: &String, data: T)
        where T: ShaderStructUniform + std::marker::Send + std::marker::Sync + Pod + 'static
    {
        match self._stages.get_mut(&stage_id) {
            Some(stage) => drop(stage._uniform_buffer.uniform_by_name(data, name)),
            None => ()
        };
    }

    pub fn image_array_to_all(&mut self, name: &String, texures: &[&Texture], first_index: usize)
    {
        for (_, rs) in &mut self._stages {
            drop(rs._uniform_buffer.uniform_sampler_array_by_name(texures, first_index, name));
        }
    }

    pub fn image_to_all(&mut self, name: &String, data: &Texture)
    {
        for (_, rs) in &mut self._stages {
            drop(rs._uniform_buffer.uniform_sampler_by_name(data, name));
        }
    }

    pub fn image_to_stage(&mut self, stage_id: StageIndex, name: &String, data: &Texture)
    {
        match self._stages.get_mut(&stage_id) {
            Some(stage) => drop(stage._uniform_buffer.uniform_sampler_by_name(data, name)),
            None => ()
        };
    }

    pub fn resize_stage(&mut self, stage_id: StageIndex, width: u16, height: u16)
    {
        match self._stages.get_mut(&stage_id) {
            Some(stage) => {
                stage._resolution = TextureDimensions::Dim2d{width: width as _, height: height as _, array_layers: 1};
                let mut accs = Vec::new();
                for output in &mut stage._outputs {
                    let buff = match output {
                        RenderStageOutput::Accumulator{buffer, pix_fmt, filtering} => 
                        {
                            RenderStageOutput::Accumulator{
                                buffer: Texture::new_empty(buffer.name(), stage._resolution, *pix_fmt, self._device.clone()).unwrap(),
                                pix_fmt: *pix_fmt,
                                filtering: *filtering
                            }
                        },
                        RenderStageOutput::Generic{..} => 
                        {
                            output.clone()
                        }
                    };
                    accs.push(buff);
                }
                stage._outputs = accs;
            },
            None => ()
        };
    }

    pub fn stage_builder(device: Arc<Device>) -> RenderStageBuilder
    {
        let mut builder = Shader::builder(ShaderType::Fragment, device);
        builder
            .define("iResolution", "resolution.dimensions")
            .input("position", AttribType::FVec2)
            .input("fragCoordWp", AttribType::FVec2)
            .input("fragCoord", AttribType::FVec2)
            .uniform_autoincrement::<RenderResolution>("resolution", 0).unwrap()
            .uniform_autoincrement::<UniformTime>("timer", 0).unwrap();

        RenderStageBuilder {
            _dimenstions: TextureDimensions::Dim2d{ width: 256, height: 256, array_layers: 1 },
            _fragment_shader: builder,
            _inputs: 0,
            _output_accum: Vec::new()
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
        self._image_inputs.clear();
        self._outputs.clear();
    }

    /// Подать текстуру на вход узла постобработчика
    pub fn set_input(&mut self, stage: StageIndex, input: StageInputIndex, tex: &Texture)
    {
        self._image_inputs.insert(RenderStageInputSocket {render_stage_id: stage, input: input}, tex.clone());
    }

    /// Получение текстуры-выхода
    /// Input потому, что это вход для нулевой ноды, являющейся выходом дерева
    #[allow(dead_code)]
    pub fn get_output(&self, name: StageInputIndex) -> Option<&Texture>
    {
        self._outputs.get(&name)
    }

    /// Закрепление текстуры за входом нулевой ноды.
    /// Для всех входов, которым не назначены изображения создадутся новые.
    /// Если требуется выводить результат уже в существующее изображение,
    /// например swapchain-изображение, это то, что нужно.
    pub fn set_output(&mut self, name: StageInputIndex, texture: Texture)
    {
        self._outputs.insert(name, texture);
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
    pub fn execute_graph(&mut self) -> AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>
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
        
        self._image_inputs.clear();
        command_buffer_builder //.build().unwrap()
    }

    /// Запрос свободного изображения
    fn request_texture(&mut self, link: &RenderStageLink, pix_fmt: TexturePixelFormat, filtering: TextureFilter) -> &Texture
    {
        let resolution = self._stages.get(&link._from.render_stage_id).unwrap()._resolution;
        let mut texture: Option<Texture> = None;

        // Находим свободное изображение
        for tex in &self._buffers {
            let mut busy = false;
            self._busy_buffers.values().for_each(|busy_tex| {
                if tex.box_id()==busy_tex.box_id() {
                    busy = true;
                };
            });
            let buff = tex;
            if !busy &&
                buff.width() == resolution.width() as u32 &&
                buff.height() == resolution.height() as u32 &&
                buff.pix_fmt() == pix_fmt && 
                buff.mag_filter() == filtering
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
            //println!("Создание текстуры {} {}x{}", buffer_name, resolution.0, resolution.1);
            let mut tex = Texture::new_empty(buffer_name.as_str(), resolution, pix_fmt, self._device.clone()).unwrap();
            tex.clear_color(self._queue.clone());
            tex.set_horizontal_address(TextureRepeatMode::ClampToEdge);
            tex.set_vertical_address(TextureRepeatMode::ClampToEdge);
            tex.set_mag_filter(filtering);
            tex.set_min_filter(filtering);
            tex.update_sampler();

            self._buffers.push(tex.clone());
            texture = Some(tex.clone());
            self._busy_buffers.insert(link.clone(), tex.clone());
        }

        // Если выход ноды направлен на выход графа...
        if link._to.render_stage_id == 0 {
            if output_has_texture {
                // Возвращаем изображение, закреплённое за выходом, если оно назначено
                //println!("На выход {} назначено изображение. Берём его.", link._to.input);
                return self._outputs.get(&link._to.input).unwrap();
            }
            // Назначаем его, если оно не назначено.
            //println!("На выход {} не назначено изображение. Назначаем его.", link._to.input);
            self._outputs.insert(link._to.input.clone(), texture.clone().unwrap());
            return self._outputs.get(&link._to.input).unwrap();
        }
        return self._busy_buffers.get(link).unwrap();
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

    fn stage_by_id_mut(&mut self, id: StageIndex) -> &mut RenderStage
    {
        self._stages.get_mut(&id).unwrap()
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

        let resolution = self.stage_by_id(id)._resolution;
        let stage_shader = self.stage_by_id(id)._program.clone();
        //println!("Выполнение ноды {}", id);
        {
            let timer = self.timer;
            let image_inputs = self._image_inputs.clone();
            let stage = self.stage_by_id_mut(id);
            
            let mut program = stage_shader.lock_write();
            let render_pass = stage._render_pass.clone();

            program.use_subpass(render_pass.clone().first_subpass());
            
            drop(stage._uniform_buffer.uniform_by_name(
                RenderResolution{
                    width:  resolution.width() as f32,
                    height: resolution.height() as f32,
                    ..Default::default()
                }, &format!("resolution")));
            drop(stage._uniform_buffer.uniform_by_name(timer, &"timer".to_owned()));

            for (RenderStageInputSocket{render_stage_id, input}, tex) in &image_inputs {
                if render_stage_id == &id {
                    //drop(program.uniform_sampler_by_name(tex, input));
                    match stage._uniform_buffer.uniform_sampler_by_name(tex, &input) {
                        Ok(_) => (), //println!("Принимается входящее изображение {} на вход", input),
                        Err(_) => (), //println!("Для входящего изображения {} не назначена uniform-переменная", input),
                    };
                }
            }
        }
        //let stage = self.stage_by_id(id);
        let render_pass = self.stage_by_id(id)._render_pass.clone();
        let mut render_targets = HashMap::<StageOutputIndex, Texture>::new();
        for link in &links {
            if link._to.render_stage_id == id {
                let from_stage = self.stage_by_id(link._from.render_stage_id);
                if from_stage.is_output_acc(link._from.output) {
                    //println!("Принимается входящий накопительный буфер {} на вход", link._to.input);
                    let acc = from_stage.get_accumulator_buffer(link._from.output).clone();
                    drop(self.stage_by_id_mut(id)._uniform_buffer.uniform_sampler_by_name(&acc, &link._to.input));
                } else {
                    //println!("Принимается входящий буфер {} на вход", link._to.input);
                    let free_tex = self._busy_buffers.get(link).unwrap().clone();
                    drop(self.stage_by_id_mut(id)._uniform_buffer.uniform_sampler_by_name(&free_tex, &link._to.input));
                }
            }
            if link._from.render_stage_id == id {
                if render_targets.contains_key(&link._from.output) { continue; };
                //println!("Запрос буфера для записи в слот {}.", link._from.output);
                let output = self.stage_by_id(id)._outputs.get(link._from.output as usize).unwrap().clone();
                let _tex = self.request_texture(link, output.pix_fmt(), output.filtering());
                //let __tex = _tex.take_mut();
                render_targets.insert(link._from.output, _tex.clone());
            }
        }
        {
            let _ub = &self.stage_by_id(id)._uniform_buffer;
            let fb = &mut self._framebuffer;
            fb.reset_attachments();
            for ind in 0..render_targets.len() {
                let tex = 
                match render_targets.get(&(ind as _)) {
                    Some(tex) => tex,
                    None => panic!("Нода {} имеет неиспользованный выход {}.", id, ind)
                };
                fb.add_color_attachment(tex, [0.0, 0.0, 0.0, 1.0].into()).unwrap();
            }
            fb.view_port(resolution.width() as _, resolution.height() as _);

            let prog = &mut *stage_shader.lock();
            command_buffer_builder
                .bind_framebuffer(&mut *fb, render_pass.clone(), false).unwrap()
                .bind_shader_program(prog).unwrap()
        }
            .bind_shader_uniforms(&mut self.stage_by_id_mut(id)._uniform_buffer, false).unwrap()
            .bind_mesh(&*self._screen_plane.lock()).unwrap()
            .end_render_pass().unwrap();
            
        for output in 0..16 {
            if self.stage_by_id(id).is_output_acc(output)
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
                let new_acc = self.stage_by_id_mut(id).swap_accumulator_buffer(output, old_acc);
                self._buffers.push(new_acc);
            }
        }
        
        //self.replace_stage(id, &stage);

        //println!("Конец выполнения ноды {}", id);

        for link in &links {
            if link._to.render_stage_id == id {
                self.free_texture(link);
            }
        }
    }

    /// Стандартный вершинный шейдер для фильтров постобработки
    fn vertex_plane_shader(&self) -> Result<Shader, String>
    {
        let mut shader = Shader::builder(ShaderType::Vertex, self._device.clone());
        shader
            .default_vertex_attributes()
            .define("iResolution", "resolution.dimensions")
            .uniform_autoincrement::<RenderResolution>("resolution", 0).unwrap()
            .uniform_autoincrement::<UniformTime>("timer", 0).unwrap()
            .output("position", AttribType::FVec2)
            .output("fragCoordWp", AttribType::FVec2)
            .output("fragCoord", AttribType::FVec2)
            .code("void main()
{
    position = v_pos.xy;
    fragCoordWp = v_pos.xy*0.5+0.5;
    fragCoord = fragCoordWp*iResolution;
    gl_Position = vec4(v_pos.xy, 0.0, 1.0);
}");
        shader.build()?;
        Ok(shader)
    }
}

#[allow(dead_code)]
#[repr(C)]
#[derive(Clone, Copy, Default, Zeroable, Pod)]
struct RenderResolution
{
    pub width : f32,
    pub height : f32,
    dummy : [f32; 14]
}

impl ShaderStructUniform for RenderResolution
{
    fn structure() -> String
    {
        "{
            vec2 dimensions;
        }".to_owned()
    }

    fn glsl_type_name() -> String
    {
        "Resolution".to_owned()
    }

    fn texture(&self) -> Option<&Texture>
    {
        None
    }
}