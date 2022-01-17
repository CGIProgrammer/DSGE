
use crate::mesh::{Mesh, MeshRef, MeshBinder};
use crate::references::*;
use crate::framebuffer::*;
use crate::shader::*;
use crate::texture::*;
use std::collections::HashMap;
use crate::types::*;

use std::sync::Arc;
use vulkano::device::{Queue, Device};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, SubpassContents, PrimaryAutoCommandBuffer};
use vulkano::render_pass::{RenderPassDesc, SubpassDesc, RenderPass, AttachmentDesc, Subpass};

type StageIndex = u16;
type StageInputIndex = u32;
type StageOutputIndex = u64;

#[derive(Clone, Debug)]
struct RenderStageOutputSocket
{
    render_stage_id: StageIndex,
    output: StageOutputIndex
}

impl std::hash::Hash for RenderStageOutputSocket
{
    fn hash<H>(&self, state: &mut H)
    where
        H: std::hash::Hasher,
    {
        state.write_u32(self.render_stage_id as u32);
        state.write_u32(self.output as u32);
        state.finish();
    }
}

impl PartialEq for RenderStageOutputSocket {
    fn eq(&self, other: &Self) -> bool {
        self.render_stage_id == other.render_stage_id &&
        self.output == other.output
    }
}

impl Eq for RenderStageOutputSocket { }

#[derive(Clone, Debug)]
struct RenderStageInputSocket
{
    render_stage_id: StageIndex,
    input:  StageInputIndex,
}

impl std::hash::Hash for RenderStageInputSocket
{
    fn hash<H>(&self, state: &mut H)
    where
        H: std::hash::Hasher,
    {
        state.write_u32(self.render_stage_id as u32);
        state.write_u32(self.input as u32);
        state.finish();
    }
}

impl PartialEq for RenderStageInputSocket {
    fn eq(&self, other: &Self) -> bool {
        self.render_stage_id == other.render_stage_id &&
        self.input == other.input
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

impl Eq for RenderStageInputSocket {}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
struct RenderStageLink
{
    _from: RenderStageOutputSocket,
    _to: RenderStageInputSocket
}

#[derive(Clone)]
struct RenderStage
{
    _id: StageIndex,
    _program: ShaderProgramRef,
    _resolution: (u16, u16),
    _accumulators: HashMap<StageOutputIndex, TextureRef>,
    _executed: bool,
}

impl RenderStage
{
    /*pub fn new(shader: &ShaderProgramRef, width: u16, height: u16) -> Self
    {
        let res = Self {
            _program: shader.clone(),
            _resolution: (width, height),
            _accumulators: HashMap::new(),
            _executed: false
        };
        res
    }*/

    pub fn executed(&self) -> bool
    {
        self._executed
    }

    pub fn execute(&mut self)
    {
        self._executed = true;
    }

    pub fn reset(&mut self)
    {
        self._executed = false;
    }

    pub fn id(&self) -> StageIndex
    {
        self._id
    }

    pub fn is_output_acc(&self, output: StageOutputIndex) -> bool
    {
        self._accumulators.contains_key(&output)
    }

    pub fn set_accumulator(&mut self, output: StageOutputIndex)
    {
        self._accumulators.insert(output, 
            Texture::new_empty_2d(
                format!("stage_{}_{:?}_accumulator", self._id, output).as_str(),
                self._resolution.0, self._resolution.1, TexturePixelFormat::RGBA16f, self._program.take().device().clone()).unwrap()
        );
    }

    pub fn accumulator(&self, output: StageOutputIndex) -> TextureRef
    {
        self._accumulators.get(&output).unwrap().clone()
    }

    pub fn swap_accumulator(&mut self, output: StageOutputIndex, new_buff: &TextureRef) -> TextureRef
    {
        let texture = self._accumulators.get(&output).unwrap().clone();
        self._accumulators.insert(output, new_buff.clone());
        texture
    }

    pub fn remove_accumulator(&mut self, output: StageOutputIndex)
    {
        self._accumulators.remove(&output);
    }
}

pub struct RenderPostprocessingGraph
{
    _render_stage_id_counter: StageIndex,
    _stages: HashMap<StageIndex, RenderStage>,
    _links: Vec<RenderStageLink>,
    _buffers: Vec<TextureRef>,
    _busy_buffers: HashMap<RenderStageLink, TextureRef>,
    _inputs: HashMap<RenderStageInputSocket, TextureRef>,
    _outputs: HashMap<StageInputIndex, TextureRef>,
    _framebuffer : FramebufferRef,
    _screen_plane : MeshRef,
    _device : Arc<Device>,
    _queue : Arc<Queue>,
    _render_pass : Arc<vulkano::render_pass::RenderPass>
}

use vulkano::image::{ImageLayout, SampleCount};

impl RenderPostprocessingGraph
{
    pub fn new(queue: Arc<Queue>) -> Self
    {
        let device = queue.device();
        let max_attachments = 4;
        let mut attachments = Vec::new();
        let mut subpasses = Vec::new();
        for i in 0..max_attachments {
            attachments.push(
                AttachmentDesc {
                    format: TexturePixelFormat::RGBA16f.vk_format(),
                    samples: SampleCount::Sample1,
                    load: vulkano::render_pass::LoadOp::DontCare,
                    store: vulkano::render_pass::StoreOp::Store,
                    stencil_load: vulkano::render_pass::LoadOp::Clear,
                    stencil_store: vulkano::render_pass::StoreOp::Store,
                    initial_layout: ImageLayout::ColorAttachmentOptimal,
                    final_layout: ImageLayout::ColorAttachmentOptimal,
                }
            );
            let mut subpass_atts = Vec::new();
            for j in 0..i {
                subpass_atts.push((j, ImageLayout::ColorAttachmentOptimal));
            }
            let subpass_desc = SubpassDesc {
                color_attachments: subpass_atts,
                depth_stencil: None,
                input_attachments: vec![],
                resolve_attachments: vec![],
                preserve_attachments: vec![],
            };
            subpasses.push(subpass_desc);
        }
        let render_pass_desc = RenderPassDesc::new(
            attachments,
            subpasses,
            vec![]
        );
        Self {
            _render_stage_id_counter: 1,
            _stages: HashMap::new(),
            _links: Vec::new(),
            _buffers: Vec::new(),
            _busy_buffers: HashMap::new(),
            _framebuffer: Framebuffer::new(1280, 720),
            _screen_plane: Mesh::make_screen_plane(device.clone()).unwrap(),
            _inputs: HashMap::new(),
            _outputs: HashMap::new(),
            _device: device.clone(),
            _queue: queue.clone(),
            _render_pass: RenderPass::new(device.clone(), render_pass_desc).unwrap()
        }
    }

    /*pub fn uniform<T: ShaderStructUniform + std::marker::Send + std::marker::Sync + Clone + 'static>(&mut self, data: &T)
    {
        for (_, stage) in &self._stages {
            let mut prog = stage._program.take();

            prog.uniform(data, 1);
            /*if data.is_texture() {
                prog.uniform(format!("{}_res", name));
            }*/
        }
    }*/

    // Подать текстуру на вход узла постобработчика
    pub fn set_input(&mut self, stage: StageIndex, input: StageInputIndex, tex: &TextureRef)
    {
        self._inputs.insert(RenderStageInputSocket {render_stage_id: stage, input: input}, tex.clone());
    }

    /*
     * Получение текстуры-выхода
     * Input потому, что это вход для нулевой ноды, являющейся выходом дерева
     */
    pub fn get_output(&self, name: &StageInputIndex) -> Option<TextureRef>
    {
        let result = self._outputs.get(name);
        if result.is_some() {
            Some(result.unwrap().clone())
        } else {
            None
        }
    }

    /* 
     * Добавление узла постобработки
     */
    pub fn add_stage(&mut self, program: &ShaderProgramRef, width: u16, height: u16) -> StageIndex
    {
        let render_stage = RenderStage { 
            _id: self._render_stage_id_counter,
            _program: program.clone(),
            _resolution: (width, height),
            _accumulators: HashMap::new(),
            _executed: false
        };
        let rsid = self._render_stage_id_counter;
        self._stages.insert(rsid, render_stage);
        self._render_stage_id_counter += 1;
        rsid
    }

    /*
     * Добавить связь между узлами.
     * Связь может быть циклической только если зацикленному выходу узла явно указано
     * использовать накопительный буфер
     */
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
    
    /*
     * Выполнить граф потобработки.
     * TODO: сделать проверку неуказанных циклов, иначе при наличии циклов без
     * накопительных буферов будет переполняться стек.
     */
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
        
        /*for (_, stage) in &self._stages {
            stage._program.take().reset_texture_counter();
        }*/
        self._inputs.clear();
        command_buffer_builder.build().unwrap()
    }

    fn detect_loop_back(&self, reference_st_id: StageIndex, current_st_id: StageIndex) -> bool
    {
        for link in &self._links {
            if link._to.render_stage_id == current_st_id {
                if link._from.render_stage_id == reference_st_id {
                    return true;
                } else {
                    let loop_detected = self.detect_loop_back(reference_st_id.clone(), link._from.render_stage_id.clone());
                    if loop_detected {
                        return false;
                    }
                }
            }
        }
        false
    }

    fn detect_loop_forward(&self, reference_st_id: StageIndex, current_st_id: StageIndex) -> bool
    {
        for link in &self._links {
            if link._from.render_stage_id == current_st_id {
                if link._to.render_stage_id == reference_st_id.clone() {
                    return true;
                } else {
                    let loop_detected = self.detect_loop_forward(reference_st_id.clone(), link._to.render_stage_id.clone());
                    if loop_detected {
                        return false;
                    }
                }
            }
        }
        false
    }

    fn request_texture(&mut self, link: &RenderStageLink) -> TextureRef
    {
        /*if self._accumulators.contains_key(&link._from) {
            return self._accumulators.get(&link._from).unwrap().clone();
        }*/
        let resolution = self._stages.get(&link._from.render_stage_id).unwrap()._resolution;
        let mut texture: Option<TextureRef> = None;
        for tex in &self._buffers {
            let mut busy = false;
            self._busy_buffers.values().for_each(|busy_tex| {
                if tex.box_id()==busy_tex.box_id() {
                    busy = true;
                };
            });
            if !busy &&
                tex.take().width() == resolution.0 as u32 &&
                tex.take().height() == resolution.1 as u32 {
                self._busy_buffers.insert(link.clone(), tex.clone());
                texture = Some(tex.clone());
            }
        }
        if texture.is_none() {
            let buffer_name = format!("render buffer for link from {}:{} to {}:{}",
                link._from.render_stage_id, link._from.output,
                link._to.render_stage_id, link._to.input);
            println!("Создание текстуры {} {}x{}", buffer_name, resolution.0, resolution.1);
            let mut _tex = Texture::new_empty_2d(buffer_name.as_str(), resolution.0, resolution.1, TexturePixelFormat::RGBA16f, self._device.clone()).unwrap();
            self._buffers.push(_tex.clone());
            texture = Some(_tex.clone());
            self._busy_buffers.insert(link.clone(), _tex.clone());
        }
        if link._to.render_stage_id == 0 {
            //println!("Передача текстуры {} на выход", link._to.input.clone());
            self._outputs.insert(link._to.input, texture.clone().unwrap());
        }
        texture.unwrap()
    }

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
    
    fn execute_stage(&mut self, id: StageIndex, command_buffer_builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>)
    {
        self._stages.get_mut(&id).unwrap().execute();
        let links = self._links.clone();

        for link in &links {
            let st_id = link._from.render_stage_id;
            let stage = self.stage_by_id(st_id);
            if link._to.render_stage_id == id && !stage.executed() {
                self.execute_stage(st_id, command_buffer_builder);
            }
        }
        //println!("Стадия {}", id);
        
        let stage = self.stage_by_id(id);
        let _prog = stage._program.clone();
        let mut program = _prog.take_mut();
        
        program.uniform(
            &RenderResolution{
                width: stage._resolution.0 as f32,
                height: stage._resolution.1 as f32
            },
            0
        );
        drop(stage);
        
        for (RenderStageInputSocket{render_stage_id, input}, tex) in &self._inputs {
            if render_stage_id == &id {
                program.uniform(tex, *input as usize + 1);
            }
        }

        let mut render_targets = Vec::<(StageOutputIndex, TextureRef)>::new();
        for link in &links {
            if link._to.render_stage_id == id {
                //println!("Передача текстуры {}:{:?} -> {}:{}", link._from.render_stage_id, link._from.output, link._to.render_stage_id, link._to.input);
                let from_stage = self.stage_by_id(link._from.render_stage_id);
                if from_stage.is_output_acc(link._from.output) {
                    let acc = from_stage.accumulator(link._from.output);
                    //println!("Использование накопительного буфера {} -> {}", acc.take().get_name(), link._to.input.as_str());
                    program.uniform(&acc, link._to.input as usize);
                } else {
                    let free_tex = self._busy_buffers.get(link).unwrap();
                    //println!("Использование свободного буфера {} -> {}", free_tex.take().get_name(), link._to.input.as_str());
                    program.uniform(free_tex, link._to.input as usize);
                }
            }
            if link._from.render_stage_id == id {
                let _tex = self.request_texture(link);
                render_targets.push((link._from.output, _tex.clone()));
            }
        }
        drop(program);
        render_targets.sort_by_key(|att| { att.0 });

        let mut fb = self._framebuffer.take_mut();
        fb.reset_attachments();
        for (_, tex) in &render_targets {
            fb.add_color_attachment(tex.clone(), [0.0, 0.0, 0.0, 0.0].into()).unwrap();
        }
        drop(fb);

        _prog.take_mut().make_pipeline(Subpass::from(self._render_pass.clone(), render_targets.len() as u32 - 1).unwrap());

        command_buffer_builder
            .bind_framebuffer(self._framebuffer.clone(), self._render_pass.clone()).unwrap()
            .bind_shader_program(&_prog)
            .bind_shader_uniforms(&_prog)
            .bind_mesh(&self._screen_plane);
            

        let mut stage = self.stage_by_id(id).clone();
        for output in 0..16 {
            if stage.is_output_acc(output)
            {
                let (_, used_acc) = render_targets.get(output as usize).unwrap();
                let mut buf_index = 0;
                for buf in &self._buffers {
                    if buf.box_id() == used_acc.box_id() {
                        break;
                    }
                    buf_index += 1;
                }
                let old_acc = &self._buffers.remove(buf_index);
                let new_acc = stage.swap_accumulator(output, old_acc);
                //println!("Переключение буферов ({}) {} <-> {}", self._buffers.len(), old_acc.take().get_name(), new_acc.take().get_name());
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
}
