use crate::command_buffer::{CommandBufferFather, PrimaryCommandBufferAssembler};
use crate::framebuffer::*;
use crate::mesh::{Mesh, MeshCommandSet, MeshRef};
use crate::references::*;
use crate::shader::*;
use crate::texture::*;
use crate::time::UniformTime;
use crate::types::*;
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::pipeline::graphics::rasterization::CullMode;
use vulkano::pipeline::graphics::vertex_input::VertexBufferDescription;
use std::collections::HashMap;
use std::sync::Arc;
use vulkano::device::{Device, Queue};
use vulkano::image::{ImageLayout, SampleCount};
use vulkano::render_pass::{
    AttachmentDescription, AttachmentReference, RenderPass, RenderPassCreateInfo,
    SubpassDescription,
};

use super::{bump_memory_allocator_new_default, BumpMemoryAllocator};

type StageIndex = u16;
type StageInputIndex = String;
type StageOutputIndex = u64;

mod debug_overlay;
mod fsr;
mod lightintg;
mod rolling_hills;
mod stack_buffer;
mod temporal_filter;
mod y_flip;

/// Выход ноды постобработки.
/// Задаётся:
///   1. уникальным номером ноды `render_stage_id`
///   2. номером выхода ноды
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct RenderStageOutputSocket {
    render_stage_id: StageIndex,
    output: StageOutputIndex,
    stack_index: u8,
}

/// Вход ноды постобработки.
/// Задаётся:
///   1. уникальным номером ноды `render_stage_id`
///   2. номером входа ноды
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct RenderStageInputSocket {
    render_stage_id: StageIndex,
    input: StageInputIndex,
}

/// Связь нод постобработки
#[derive(Clone, Hash, PartialEq, Eq, Debug)]
struct RenderStageLink {
    _from: RenderStageOutputSocket,
    _to: RenderStageInputSocket,
}

#[derive(Clone)]
enum RenderStageOutput {
    Generic {
        pix_fmt: TexturePixelFormat,
    },
    Stack {
        buffers: Vec<Texture>,
        pointer: usize,
        pix_fmt: TexturePixelFormat,
    },
}

impl RenderStageOutput {
    #[inline]
    pub fn is_stack(&self) -> bool {
        match self {
            Self::Stack { .. } => true,
            Self::Generic { .. } => false,
        }
    }

    #[inline]
    pub fn get_buffer_with_offset(&self, offset: i32) -> Option<Texture> {
        let offset = offset as usize;

        match self {
            Self::Stack {
                buffers, pointer, ..
            } => match buffers.get((*pointer + offset).rem_euclid(buffers.len())) {
                Some(tex) => Some(tex.clone()),
                None => None,
            },
            _ => None,
        }
    }

    fn shift_stack(&mut self, buffer: &Texture) -> Texture {
        match self {
            RenderStageOutput::Generic {
                pix_fmt: _,
            } => todo!(),
            RenderStageOutput::Stack {
                buffers, pointer, ..
            } => {
                *pointer = ((*pointer as isize - 1).rem_euclid(buffers.len() as isize)) as _;
                let poped_buffer = buffers[*pointer].clone();
                buffers[*pointer] = buffer.clone();
                poped_buffer
            }
        }
    }

    #[inline]
    pub fn new_stack(
        command_buffer_father: &CommandBufferFather,
        allocator: Arc<BumpMemoryAllocator>,
        dimensions: TextureDimensions,
        size: usize,
        pix_fmt: TexturePixelFormat,
    ) -> Self {
        let buffers = (0..size)
            .map(|_| {
                let mut buffer = Texture::new(
                    format!("{pix_fmt:?} x {size}_stack").as_str(),
                    dimensions,
                    false,
                    TextureViewType::Dim2d,
                    pix_fmt,
                    TextureUseCase::Attachment,
                    allocator.clone(),
                )
                .unwrap();
                //buffer.clear_color(queue.clone());
                buffer.set_address_mode([TextureRepeatMode::ClampToEdge; 3]);
                buffer
            })
            .collect::<Vec<_>>();
        Self::Stack {
            buffers: buffers,
            pointer: 0,
            pix_fmt: pix_fmt,
        }
    }

    #[inline]
    pub fn pix_fmt(&self) -> TexturePixelFormat {
        match self {
            Self::Generic { pix_fmt, .. } => *pix_fmt,
            Self::Stack { pix_fmt, .. } => *pix_fmt,
        }
    }
}

/// Нода (она же стадия) постобработки
#[derive(Clone)]
struct RenderStage {
    _id: StageIndex,
    _program: ShaderProgramRef,
    _uniform_buffer: ShaderProgramUniformBuffer,
    _resolution: TextureDimensions,
    _input_filters: HashMap<String, TextureFilter>,
    _outputs: Vec<RenderStageOutput>,
    _executed: bool,
    _render_pass: Arc<RenderPass>,
}

impl RenderStage {
    #[inline]
    fn uniform_buffer(&mut self) -> &mut ShaderProgramUniformBuffer {
        &mut self._uniform_buffer
    }

    /// Возвращает true, если нода помечена как выполненная
    #[inline]
    fn executed(&self) -> bool {
        self._executed
    }

    /// Пометить ноду выполненной
    fn mark_executed(&mut self) {
        self._executed = true;
    }

    /// Сбросить статус выполнения ноды
    fn reset(&mut self) {
        self._executed = false;
    }

    /// Проверяет назначен ли выходу ноды стековый буфер
    fn is_output_stack(&self, output: StageOutputIndex) -> bool {
        if (output as usize) < (self._outputs.len() as usize) {
            //self._outputs.get_unchecked(output as usize).0.is_some() }
            let output = self._outputs.get(output as usize).unwrap();
            match output {
                RenderStageOutput::Stack { .. } => true,
                RenderStageOutput::Generic { .. } => false,
            }
        } else {
            false
        }
    }

    /// Возвращает изображение из стекового буфера
    fn get_accumulator_buffer(&self, output: StageOutputIndex, index: i32) -> Texture {
        self._outputs
            .get(output as usize)
            .unwrap()
            .get_buffer_with_offset(index)
            .unwrap()
            .clone()
        //self._accumulators.get(&output).unwrap().clone()
    }

    /// Меняет стековый буфер на `new_buff` и возвращает предыдущий
    fn shift_stack_buffer(&mut self, output: StageOutputIndex, new_buff: &Texture) -> Texture {
        self._outputs
            .get_mut(output as usize)
            .unwrap()
            .shift_stack(new_buff)
    }

    fn attach_image(&mut self, name: &str, image: &Texture) -> Result<(), String>
    {
        if let Some(filter) = self._input_filters.get(name) {
            let mut image = image.clone();
            let filter = filter.clone();
            //println!("Передача изображения на вход {name} с фильтром {filter:?}.");
            image.set_min_filter(filter);
            image.set_mag_filter(filter);
            image.set_mipmap_mode(MipmapMode::Nearest);
            self._uniform_buffer.uniform_sampler_by_name(&image, name)?;
        }
        Ok(())
    }

    /*pub fn uniform<T>(&mut self, data: &T)
        where T: ShaderStructUniform + std::marker::Send + std::marker::Sync + Clone + 'static
    {
        self._program.take_mut().uniform(data, 0);
    }*/
}

pub struct RenderStageBuilder {
    _dimensions: TextureDimensions,
    _fragment_shader: Shader,
    _output_accum: Vec<(TexturePixelFormat, u8)>,
    _input_filters: HashMap<String, TextureFilter>,
}

#[allow(dead_code)]
impl RenderStageBuilder {
    pub fn dimenstions(&mut self, width: u16, height: u16) -> &mut Self {
        self._dimensions = [width as u32, height as u32, 1];
        self
    }

    pub fn uniform<T: ShaderStructUniform>(
        &mut self,
        name: &str,
        array_length: ShaderUniformArrayLength,
    ) -> &mut Self {
        self._fragment_shader
            .uniform_autoincrement::<T>(name, array_length, 0)
            .unwrap();
        self
    }

    pub fn uniform_named_type<T: ShaderStructUniform>(
        &mut self,
        name: &str,
        _type: &str,
        array_length: ShaderUniformArrayLength,
    ) -> &mut Self {
        self._fragment_shader
            .uniform_structure_autoincrement(name, _type, array_length, T::structure().as_str(), 0)
            .unwrap();
        self
    }

    pub fn uniform_structure(
        &mut self,
        name: &str,
        _type: &str,
        array_length: ShaderUniformArrayLength,
        structure: &str,
    ) -> &mut Self {
        self._fragment_shader
            .uniform_structure_autoincrement(name, _type, array_length, structure, 0)
            .unwrap();
        self
    }

    pub fn input(&mut self, name: &str, dims: TextureView, filter: TextureFilter, shadowmap: bool) -> &mut Self {
        self._fragment_shader
            .uniform_sampler(name, 1, self._input_filters.len() as _, dims, shadowmap)
            .unwrap();
        //self._fragment_shader.uniform_sampler_autoincrement(name, self._inputs as usize, TextureView::Dim2d).unwrap();
        self._input_filters.insert(name.to_owned(), filter);
        self
    }

    pub fn output(
        &mut self,
        name: &str,
        pix_fmt: TexturePixelFormat,
        stack: u8,
    ) -> &mut Self {
        let glsl_type = match pix_fmt.subpixels() {
            1 => AttribType::Float,
            2 => AttribType::FVec2,
            3 => AttribType::FVec3,
            4 => AttribType::FVec4,
            _ => panic!(),
        };
        self._fragment_shader.output(name, glsl_type);
        self._output_accum.push((pix_fmt, stack));
        self
    }

    pub fn code(&mut self, code: &str) -> &mut Self {
        self._fragment_shader.code(code);
        self
    }

    pub fn build(
        mut self,
        pp_graph: &mut PostprocessingPass
    ) -> Result<StageIndex, String> {
        let queue = pp_graph._command_buffer_father.queue().clone();
        let device = queue.device().clone();
        let mut program = ShaderProgram::builder();
        let v_shader = pp_graph.vertex_plane_shader()?;
        let f_shader = self._fragment_shader.build()?;
        program
            .vertex(&v_shader)
            .unwrap()
            .fragment(f_shader)
            .unwrap();

        let outputs = self
            ._output_accum
            .iter()
            .map(|(pix_fmt, accum)| {
                let acc = if *accum > 0 {
                    //println!("Создание стекового буфера {}x{}", self._dimensions.width(), self._dimensions.height());
                    RenderStageOutput::new_stack(
                        &pp_graph._command_buffer_father,
                        pp_graph._allocator.clone(),
                        self._dimensions,
                        *accum as _,
                        *pix_fmt,
                    )
                } else {
                    RenderStageOutput::Generic {
                        pix_fmt: *pix_fmt,
                    }
                };
                acc
            })
            .collect();

        let attachments = self
            ._output_accum
            .iter()
            .map(|(pix_fmt, _)| AttachmentDescription {
                format: pix_fmt.vk_format(),
                samples: SampleCount::Sample1,
                load_op: vulkano::render_pass::AttachmentLoadOp::DontCare,
                store_op: vulkano::render_pass::AttachmentStoreOp::Store,
                stencil_load_op: Some(vulkano::render_pass::AttachmentLoadOp::DontCare),
                stencil_store_op: Some(vulkano::render_pass::AttachmentStoreOp::DontCare),
                initial_layout: ImageLayout::Undefined,
                final_layout: ImageLayout::ColorAttachmentOptimal,
                ..Default::default()
            })
            .collect();

        let n = self._output_accum.len() as u32;
        let subpass_attachments = (0..n)
            .map(|i| {
                Some(AttachmentReference {
                    attachment: i,
                    layout: ImageLayout::ColorAttachmentOptimal,
                    ..Default::default()
                })
            })
            .collect();

        let render_pass_desc = RenderPassCreateInfo {
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
        program.use_subpass::<[VertexBufferDescription; 1]>(render_pass.clone().first_subpass(), CullMode::None, None);
        let uniform_buffer = program.new_uniform_buffer();

        let stage = RenderStage {
            _id: pp_graph._render_stage_id_counter,
            _program: RcBox::construct(program),
            _uniform_buffer: uniform_buffer,
            _resolution: self._dimensions,
            _input_filters : self._input_filters,
            _outputs: outputs,
            _render_pass: render_pass,
            _executed: false,
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
/// Поддерживаются петли с применением стековых буферов.
/// Память под буферы выделяется автоматически по мере необходимости.
/// Для перед вызовом функции выполнения, следует назначить входные
/// и выходные изображения.
pub struct PostprocessingPass {
    /// Обычный аллокатор памяти
    _allocator: Arc<BumpMemoryAllocator>,
    /// Аллокатор для выделения памяти для наборов дескрипторов
    _ds_allocator: Arc<StandardDescriptorSetAllocator>,
    /// Конструктор буферов команд
    _command_buffer_father: CommandBufferFather,
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
    _framebuffer: Framebuffer,
    /// Плоскость для вывода изображений
    _screen_plane: MeshRef,

    pub timer: UniformTime,
    //_device: Arc<Device>,
    //_queue: Arc<Queue>,
}

#[allow(dead_code)]
#[derive(Clone)]
pub enum NumericInput {
    Scalar(f32),
    Vec2(Vec2),
    Vec3(Vec3),
    Vec4(Vec4),
    Mat2(Mat2),
    Mat3(Mat3),
    Mat4(Mat4),
}

#[allow(dead_code)]
impl PostprocessingPass {
    pub fn new(queue: Arc<Queue>, width: u16, height: u16) -> Self {
        let device = queue.device().clone();
        let allocator = bump_memory_allocator_new_default(device.clone());
        let cbf = CommandBufferFather::new(queue);
        let allocator = Arc::new(allocator);
        PostprocessingPass {
            _render_stage_id_counter: 1,
            _stages: HashMap::new(),
            _links: Vec::new(),
            _buffers: Vec::new(),
            _busy_buffers: HashMap::new(),
            _framebuffer: Framebuffer::new(width, height),
            _screen_plane: Mesh::make_screen_plane(&cbf, allocator.clone()).unwrap(),
            _image_inputs: HashMap::new(),
            //_uniform_inputs: String::new(),
            timer: Default::default(),
            _outputs: HashMap::new(),
            _allocator: allocator,
            _command_buffer_father: cbf,
            _ds_allocator: Arc::new(StandardDescriptorSetAllocator::new(device, Default::default())),
            //_device: device.clone(),
            //_queue: queue.clone(),
        }
    }

    pub fn device(&self) -> &Arc<Device>
    {
        self._command_buffer_father.queue().device()
    }

    pub fn uniform_to_all<T>(&mut self, name: &String, data: T)
    where
        T: ShaderStructUniform
            + std::fmt::Debug
            + std::marker::Send
            + std::marker::Sync
            + Copy
            + 'static,
    {
        for (_, rs) in &mut self._stages {
            drop(rs.uniform_buffer().uniform_by_name(self._allocator.clone(), data, name));
        }
    }

    pub fn uniform_array_to_all<T>(&mut self, name: &String, data: &[T])
    where
        T: ShaderStructUniform
            + std::fmt::Debug
            + std::marker::Send
            + std::marker::Sync
            + Clone
            + 'static,
    {
        for (_, rs) in &mut self._stages {
            match rs.uniform_buffer().uniform_location_by_name(name.as_str()) {
                Some((set, binding)) => rs.uniform_buffer().uniform_array(self._allocator.clone(), data, 0, set, binding),
                None => (),
            };
        }
    }

    pub fn uniform_to_stage<T>(&mut self, stage_id: StageIndex, name: &String, data: T)
    where
        T: ShaderStructUniform
            + std::fmt::Debug
            + std::marker::Send
            + std::marker::Sync
            + Clone
            + 'static,
    {
        match self._stages.get_mut(&stage_id) {
            Some(stage) => drop(stage.uniform_buffer().uniform_by_name(self._allocator.clone(), data, name)),
            None => (),
        };
    }

    pub fn image_array_to_all(&mut self, name: &String, texures: &[&Texture], first_index: u32) {
        for (_, rs) in &mut self._stages {
            drop(
                rs.uniform_buffer()
                    .uniform_sampler_array_by_name(texures, first_index, name),
            );
        }
    }

    pub fn image_array_to_stage(
        &mut self,
        stage_id: StageIndex,
        name: &String,
        texures: &[&Texture],
        first_index: u32,
    ) {
        let rs = self._stages.get_mut(&stage_id).unwrap();
        
        drop(
            rs.uniform_buffer()
                .uniform_sampler_array_by_name(texures, first_index, name),
        );
    }

    pub fn image_to_all(&mut self, name: &str, data: &Texture) {
        let ids = self._stages.iter().map(|(_, rs)| rs._id).collect::<Vec<_>>();
        for stage_id in ids {
            self.image_to_stage(stage_id, name, data);
        }
    }

    pub fn image_to_stage(&mut self, stage_id: StageIndex, name: &str, data: &Texture) {
        match self._stages.get_mut(&stage_id) {
            Some(stage) => {
                let _ = stage.attach_image(name, data);
            }
            None => (),
        };
    }

    pub fn resize_stage(&mut self, stage_id: StageIndex, width: u16, height: u16) {
        match self._stages.get_mut(&stage_id) {
            Some(stage) => {
                stage._resolution = [
                    width as _,
                    height as _,
                    1,
                ];
                let mut accs = Vec::new();
                for output in &mut stage._outputs {
                    let buff = match output {
                        RenderStageOutput::Stack {
                            buffers,
                            pix_fmt,
                            pointer: _,
                        } => RenderStageOutput::new_stack(
                            &self._command_buffer_father,
                            self._allocator.clone(),
                            stage._resolution,
                            buffers.len(),
                            *pix_fmt,
                        ),
                        RenderStageOutput::Generic { .. } => output.clone(),
                    };
                    accs.push(buff);
                }
                stage._outputs = accs;
            }
            None => (),
        };
    }

    pub fn stage_builder(device: Arc<Device>) -> RenderStageBuilder {
        let mut builder = Shader::builder(ShaderType::Fragment, device);
        builder
            .define("iResolution", "resolution.dimensions")
            .input(
                "position",
                AttribType::FVec2,
                FragmentInterpolation::NoPerspective,
            )
            .input(
                "fragCoord",
                AttribType::FVec2,
                FragmentInterpolation::NoPerspective,
            )
            .input(
                "pixelCoord",
                AttribType::FVec2,
                FragmentInterpolation::NoPerspective,
            )
            .uniform_autoincrement::<RenderResolution>(
                "resolution",
                ShaderUniformArrayLength::NotArray,
                0,
            )
            .unwrap()
            .uniform_autoincrement::<UniformTime>("timer", ShaderUniformArrayLength::NotArray, 0)
            .unwrap();

        RenderStageBuilder {
            _dimensions: [256, 256, 1],
            _fragment_shader: builder,
            _input_filters: Default::default(),
            _output_accum: Vec::new(),
        }
    }

    /// Полный сброс. Нужен при изменении разрешения
    pub fn reset(&mut self) {
        self._render_stage_id_counter = 1;
        self._stages.clear();
        self._busy_buffers.clear();
        self._links.clear();
        self._buffers.clear();
        self._image_inputs.clear();
        self._outputs.clear();
    }

    /// Подать текстуру на вход узла постобработчика
    pub fn set_input(&mut self, stage: StageIndex, input: StageInputIndex, tex: &Texture) {
        self._image_inputs.insert(
            RenderStageInputSocket {
                render_stage_id: stage,
                input: input,
            },
            tex.clone(),
        );
    }

    /// Получение текстуры-выхода
    /// Input потому, что это вход для нулевой ноды, являющейся выходом дерева
    #[allow(dead_code)]
    pub fn get_output(&self, name: StageInputIndex) -> Option<&Texture> {
        self._outputs.get(&name)
    }

    /// Закрепление текстуры за входом нулевой ноды.
    /// Для всех входов, которым не назначены изображения создадутся новые.
    /// Если требуется выводить результат уже в существующее изображение,
    /// например swapchain-изображение, это то, что нужно.
    pub fn set_output(&mut self, name: StageInputIndex, texture: Texture) {
        self._outputs.insert(name, texture);
    }

    fn check_loop(
        &self,
        links: &[RenderStageLink],
        reference: StageIndex,
        stage: StageIndex,
    ) -> bool {
        for link in links {
            if link._to.render_stage_id != stage {
                continue;
            }

            let node = &self._stages[&link._from.render_stage_id];
            let output = &node._outputs[link._from.output as usize];
            if output.is_stack() {
                continue;
            }
            if link._from.render_stage_id == reference {
                return true;
            } else {
                if self.check_loop(links, reference, link._from.render_stage_id) {
                    return true;
                }
            }
        }
        false
    }

    /// Добавить связь между узлами.
    /// Связь может быть циклической только если зацикленному выходу узла явно указано
    /// использовать стековый буфер.
    /// Для подключения узла к выходу графа параметр `to: StageIndex` должен быть равен 0.
    pub fn link_stages(
        &mut self,
        from: StageIndex,
        output: StageOutputIndex,
        stack_index: Option<usize>,
        to: StageIndex,
        input: StageInputIndex,
    ) -> Result<(), String> {
        let stack_index = match stack_index {
            Some(index) => index,
            None => 0,
        };
        //self.stage_by_id(from)._outputs[output as usize];
        let link = RenderStageLink {
            _from: RenderStageOutputSocket {
                render_stage_id: from,
                output: output,
                stack_index: stack_index as _,
            },
            _to: RenderStageInputSocket {
                render_stage_id: to,
                input: input,
            },
        };
        let links = [self._links.as_slice(), &[link.clone()]].concat();
        if self.check_loop(&links, from, from) || self.check_loop(&links, to, to) {
            Err("Обнаружена петля без стековых буферов.".to_owned())
        } else {
            self._links = links;
            Ok(())
        }
    }

    pub fn final_stages(&self) -> Vec<StageIndex> {
        self._links
            .iter()
            .filter_map(|RenderStageLink { _from, _to }| {
                if _to.render_stage_id == 0 {
                    Some(_from.render_stage_id)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    }

    /// Выполнить граф потобработки.
    pub fn execute_graph(&mut self) -> PrimaryCommandBufferAssembler {
        /*let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
            self.device().clone(),
            self._queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();*/
        let mut command_buffer_builder = self._command_buffer_father.new_primary().unwrap();

        for i in &mut self._stages {
            i.1.reset();
        }
        self._busy_buffers.clear();
        for fstage in self.final_stages() {
            self.execute_stage(fstage, &mut command_buffer_builder);
        }

        self._image_inputs.clear();
        command_buffer_builder
    }

    /// Запрос свободного изображения для записи
    fn request_texture(
        &mut self,
        link: &RenderStageLink,
        pix_fmt: TexturePixelFormat,
    ) -> Texture {
        let resolution = self
            ._stages
            .get(&link._from.render_stage_id)
            .unwrap()
            ._resolution;
        let mut texture: Option<Texture> = None;

        // Находим свободное изображение
        for tex in &self._buffers {
            let mut busy = false;
            self._busy_buffers.values().for_each(|busy_tex| {
                if tex.box_id() == busy_tex.box_id() {
                    busy = true;
                };
            });
            let buff = &tex;
            if !busy
                && buff.width() == resolution[0]
                && buff.height() == resolution[1]
                && buff.pix_fmt() == pix_fmt
            {
                self._busy_buffers.insert(link.clone(), tex.clone());
                texture = Some(tex.clone());
                break;
            }
        }
        let output_has_texture =
            link._to.render_stage_id == 0 && self._outputs.contains_key(&link._to.input);

        // Если свободное изображение не найдено и выходу не назначено изображение,
        // то создаём новое изображение
        if texture.is_none() && !output_has_texture {
            let buffer_name = format!(
                "render buffer for link from {}:{} to {}:{}",
                link._from.render_stage_id,
                link._from.output,
                link._to.render_stage_id,
                link._to.input
            );
            //println!("Создание текстуры {} {}x{}", buffer_name, resolution.width(), resolution.height());
            let mut tex = Texture::new(
                buffer_name.as_str(),
                resolution,
                false,
                TextureViewType::Dim2d,
                pix_fmt,
                TextureUseCase::Attachment,
                self._allocator.clone(),
            )
            .unwrap();
            //tex.clear_color(self._queue.clone());
            tex.set_address_mode([TextureRepeatMode::ClampToEdge; 3]);

            self._buffers.push(tex.clone());
            texture = Some(tex.clone());
            self._busy_buffers.insert(link.clone(), tex.clone());
        }

        if let Some(ref tex) = texture {
            for other_link in &self._links {
                if other_link._from == link._from && link != other_link {
                    self._busy_buffers.insert(other_link.clone(), tex.clone());
                }
            }
        }

        // Если выход ноды направлен на выход графа...
        if link._to.render_stage_id == 0 {
            if output_has_texture {
                // Возвращаем изображение, закреплённое за выходом, если оно назначено
                //println!("На выход {} назначено изображение. Берём его.", link._to.input);
                return self._outputs.get(&link._to.input).unwrap().clone();
            }
            // Назначаем его, если оно не назначено.
            //println!("На выход {} не назначено изображение. Назначаем его.", link._to.input);
            self._outputs
                .insert(link._to.input.clone(), texture.clone().unwrap());
            return self._outputs.get(&link._to.input).unwrap().clone();
        }
        return self._busy_buffers.get(link).unwrap().clone();
    }

    /// Освобождение выделенного изображения
    fn free_texture(&mut self, link: &RenderStageLink) {
        self._busy_buffers.remove(link);
    }

    fn stage_by_id(&mut self, id: StageIndex) -> &RenderStage {
        self._stages.get(&id).unwrap()
    }

    fn stage_by_id_mut(&mut self, id: StageIndex) -> &mut RenderStage {
        self._stages.get_mut(&id).unwrap()
    }

    fn replace_stage(&mut self, id: StageIndex, stage: &RenderStage) {
        self._stages.insert(id.clone(), stage.clone());
    }

    /// Выполнение ноды постобработки
    fn execute_stage(
        &mut self,
        id: StageIndex,
        command_buffer_builder: &mut PrimaryCommandBufferAssembler,
    ) {
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
        let allocator = self._allocator.clone();
        let ds_allocator = self._ds_allocator.clone();
        // println!("Выполнение ноды {}", id);
        {
            let timer = self.timer;
            let image_inputs = self._image_inputs.clone();
            let stage = self.stage_by_id_mut(id);

            let mut program = stage_shader.lock_write();
            let render_pass = stage._render_pass.clone();

            program.use_subpass::<[VertexBufferDescription; 1]>(render_pass.clone().first_subpass(), CullMode::None, None);

            drop(stage.uniform_buffer().uniform_by_name(
                allocator.clone(),
                RenderResolution {
                    width: resolution[0] as _,
                    height: resolution[1] as _,
                    ..Default::default()
                },
                &format!("resolution"),
            ));
            drop(
                stage
                    .uniform_buffer()
                    .uniform_by_name(allocator.clone(), timer, &"timer".to_owned()),
            );

            for (
                RenderStageInputSocket {
                    render_stage_id,
                    input,
                },
                tex,
            ) in &image_inputs
            {
                if render_stage_id == &id {
                    match stage.attach_image(&input, tex) {
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
                if from_stage.is_output_stack(link._from.output) {
                    // println!("Принимается входящий стековый буфер {}:{} на вход", link._to.input, link._from.stack_index);
                    let acc = from_stage
                        .get_accumulator_buffer(link._from.output, link._from.stack_index as _)
                        .clone();
                    drop(
                        self.stage_by_id_mut(id)
                            .attach_image(&link._to.input, &acc)
                    );
                } else {
                    // println!("Принимается буфер на вход {}", link._to.input);
                    let free_tex = self._busy_buffers.get(link).unwrap().clone();
                    drop(
                        self.stage_by_id_mut(id)
                            .attach_image(&link._to.input, &free_tex)
                    );
                }
            }
            if link._from.render_stage_id == id {
                if render_targets.contains_key(&link._from.output) {
                    continue;
                };
                //println!("Запрос буфера для записи в слот {}.", link._from.output);
                let output = self
                    .stage_by_id(id)
                    ._outputs
                    .get(link._from.output as usize)
                    .unwrap()
                    .clone();
                let _tex = self.request_texture(link, output.pix_fmt());
                //let __tex = _tex.take_mut();
                render_targets.insert(link._from.output, _tex.clone());
            }
        }
        {
            let fb = &mut self._framebuffer;
            fb.reset_attachments();
            for ind in 0..render_targets.len() {
                let tex = match render_targets.get(&(ind as _)) {
                    Some(tex) => tex,
                    None => panic!("Нода {} имеет неиспользованный выход {}.", id, ind),
                };
                //fb.add_color_attachment(tex, [0.0, 0.0, 0.0, 1.0].into()).unwrap();
                fb.add_color_attachment(tex, None).unwrap();
            }
            fb.view_port(resolution[0] as _, resolution[1] as _);

            let prog = &mut *stage_shader.lock();
            command_buffer_builder
                .bind_framebuffer(&mut *fb, render_pass.clone(), false, false)
                .unwrap();
            command_buffer_builder
                .bind_shader_program(prog)
                .unwrap()
        }
            .bind_shader_uniforms(ds_allocator.clone(), self.stage_by_id_mut(id).uniform_buffer(), false)
            .unwrap()
            .bind_mesh(&*self._screen_plane)
            .unwrap()
            .end_render_pass(Default::default())
            .unwrap();

        for output in 0..16 {
            if self.stage_by_id(id).is_output_stack(output) {
                let used_acc = render_targets.get(&(output as _)).unwrap();
                let mut buf_index = 0;
                for buf in &self._buffers {
                    if buf.box_id() == used_acc.box_id() {
                        break;
                    }
                    buf_index += 1;
                }
                let old_acc = &self._buffers.remove(buf_index);
                let new_acc = self.stage_by_id_mut(id).shift_stack_buffer(output, old_acc);
                self._buffers.push(new_acc);
            }
        }

        //self.replace_stage(id, &stage);

        // println!("Конец выполнения ноды {}", id);

        for link in &links {
            if link._to.render_stage_id == id {
                self.free_texture(link);
            }
        }
    }

    /// Стандартный вершинный шейдер для фильтров постобработки
    fn vertex_plane_shader(&self) -> Result<Shader, String> {
        let mut shader = Shader::builder(ShaderType::Vertex, self.device().clone());
        shader
            .default_vertex_attributes()
            .define("iResolution", "resolution.dimensions")
            .uniform_autoincrement::<RenderResolution>(
                "resolution",
                ShaderUniformArrayLength::NotArray,
                0,
            )
            .unwrap()
            .uniform_autoincrement::<UniformTime>("timer", ShaderUniformArrayLength::NotArray, 0)
            .unwrap()
            .output("position", AttribType::FVec2)
            .output("fragCoord", AttribType::FVec2)
            .output("pixelCoord", AttribType::FVec2)
            .code(
                "void main()
{
    position = v_pos.xy;
    fragCoord = v_pos.xy * vec2(0.5, 0.5) + 0.5;
    pixelCoord = vec2(fragCoord * resolution.dimensions);
    gl_Position = vec4(v_pos.xy, 0.0, 1.0);
}",
            );
        shader.build()?;
        Ok(shader)
    }
}


#[allow(dead_code)]
#[repr(C, align(16))]
#[derive(Clone, Copy, Default, Debug)]
pub struct RenderResolution {
    pub width: u32,
    pub height: u32,
}

impl RenderResolution {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            width: width as _,
            height: height as _,
            ..Default::default()
        }
    }
}

impl ShaderStructUniform for RenderResolution {
    fn structure() -> String {
        "{
            uvec2 dimensions;
        }"
        .to_owned()
    }

    fn glsl_type_name() -> String {
        "Resolution".to_owned()
    }

    fn texture(&self) -> Option<&Texture> {
        None
    }
}
