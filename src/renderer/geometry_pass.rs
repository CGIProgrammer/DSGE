use std::sync::Arc;
use std::thread::{JoinHandle};
use std::sync::mpsc;

use vulkano::command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer, SecondaryAutoCommandBuffer};
use vulkano::device::{Device, Queue};
use vulkano::render_pass::{RenderPass, SubpassDescription, AttachmentDescription, AttachmentReference, RenderPassCreateInfo, Subpass};
use vulkano::image::{ImageLayout, SampleCount};
use vulkano::command_buffer::CommandBufferUsage;
use vulkano::pipeline::graphics::viewport::Viewport;

use crate::components::ProjectionUniformData;
use crate::components::{GOTransformUniform, AbstractVisual};
use crate::texture::*;
use crate::framebuffer::{Framebuffer, FramebufferBinder};

type DrawList = Vec<(GOTransformUniform, RcBox<dyn AbstractVisual>)>;
type LoopSignal = Option<(Viewport, ProjectionUniformData, DrawList)>;

struct DrawWorker
{
    join_handle: JoinHandle<()>,
    loop_signal: mpsc::Sender<LoopSignal>,
    cb_result: mpsc::Receiver<SecondaryAutoCommandBuffer>,
}

/// Буфер для сохранения результатов прохода геометрии
pub struct GeometryPass
{
    _frame_buffer: Framebuffer,
    _device      : Arc<Device>,
    _queue       : Arc<Queue>,
    _geometry_pass : Arc<RenderPass>,
    _albedo      : Texture, // Цвет поверхности
    _normals     : Texture, // Нормали
    _specromet   : Texture, // specromet - specular, roughness, metallic. TODO пока ничем не заполняется
    _vectors     : Texture, // Векторы скорости. TODO пока ничем не заполняется
    _depth       : Texture,  // Глубина. TODO пока ничем не заполняется
    _workers     : Vec<DrawWorker>
}

impl GeometryPass
{
    #[inline]
    pub fn is_parallel(&self) -> bool
    {
        self._workers.len() > 0
    }
}

impl Drop for GeometryPass
{
    fn drop(&mut self)
    {
        while self._workers.len() > 0
        {
            let worker = self._workers.remove(0);
            worker.loop_signal.send(None).unwrap();
            worker.join_handle.join().unwrap();
        }
        self._workers.clear();
    }
}

impl GeometryPass
{
    pub fn new(width : u16, height : u16, queue : Arc<Queue>, parallel_draw_calls: bool) -> Self
    {
        //println!("Создание нового G-буфера {}x{}", width, height);
        let device = queue.device();
        let formats = [
            TexturePixelFormat::R16G16B16A16_UNORM,
            TexturePixelFormat::R8G8B8A8_SNORM,
            TexturePixelFormat::R8G8B8A8_UNORM,
            TexturePixelFormat::R16G16B16A16_SNORM,
            TexturePixelFormat::D16_UNORM
        ];

        let mut fb = Framebuffer::new(width, height);
        let dims = TextureDimensions::Dim2d{
            width: width as _,
            height: height as _,
            array_layers: 1
        };
        let mut albedo  = Texture::new_empty("gAlbedo",  dims, formats[0], device.clone()).unwrap();
        let mut normals = Texture::new_empty("gNormals", dims, formats[1], device.clone()).unwrap();
        let mut masks   = Texture::new_empty("gMasks",   dims, formats[2], device.clone()).unwrap();
        let mut vectors = Texture::new_empty("gVectors", dims, formats[3], device.clone()).unwrap();
        let depth       = Texture::new_empty("gDepth",   dims, formats[4], device.clone()).unwrap();

        albedo.clear_color(queue.clone());
        albedo.set_horizontal_address(crate::texture::TextureRepeatMode::ClampToEdge);
        albedo.set_vertical_address(crate::texture::TextureRepeatMode::ClampToEdge);
        albedo.update_sampler();
        normals.clear_color(queue.clone());
        masks.clear_color(queue.clone());
        vectors.clear_color(queue.clone());

        fb.add_color_attachment(&albedo,  [0.0, 0.0, 0.0, 0.0].into()).unwrap();
        fb.add_color_attachment(&normals, [0.0, 0.0, 0.0, 0.0].into()).unwrap();
        fb.add_color_attachment(&masks,   [0.0, 0.0, 0.0, 0.0].into()).unwrap();
        fb.add_color_attachment(&vectors, [0.0, 0.0, 0.0, 0.0].into()).unwrap();
        fb.set_depth_attachment(&depth, 1.0.into());

        let mut attachments = Vec::new();
        for fmt in formats {
            let img_layout = match fmt.is_depth() {
                true => ImageLayout::DepthStencilAttachmentOptimal,
                false => ImageLayout::ColorAttachmentOptimal
            };
            let att = AttachmentDescription {
                format: Some(fmt.vk_format()),
                samples: SampleCount::Sample1,
                load_op: vulkano::render_pass::LoadOp::Clear,
                store_op: vulkano::render_pass::StoreOp::Store,
                stencil_load_op: vulkano::render_pass::LoadOp::Clear,
                stencil_store_op: vulkano::render_pass::StoreOp::Store,
                initial_layout: img_layout,
                final_layout: img_layout,
                ..Default::default()
            };
            attachments.push(att);
        }
        let desc = RenderPassCreateInfo {
            attachments: attachments,
            subpasses: vec![SubpassDescription {
                color_attachments: vec![
                    Some(AttachmentReference{attachment: 0, layout: ImageLayout::ColorAttachmentOptimal, ..Default::default()}),
                    Some(AttachmentReference{attachment: 1, layout: ImageLayout::ColorAttachmentOptimal, ..Default::default()}),
                    Some(AttachmentReference{attachment: 2, layout: ImageLayout::ColorAttachmentOptimal, ..Default::default()}),
                    Some(AttachmentReference{attachment: 3, layout: ImageLayout::ColorAttachmentOptimal, ..Default::default()})
                ],
                depth_stencil_attachment: Some(AttachmentReference{
                    attachment: 4,
                    layout: ImageLayout::DepthStencilAttachmentOptimal,
                    ..Default::default()
                }),
                ..Default::default()
            }],
            ..Default::default()
        };

        let mut result = Self {
            _device : device.clone(),
            _queue  : queue.clone(),
            _geometry_pass : RenderPass::new(device.clone(), desc).unwrap(),
            _albedo : albedo,
            _normals : normals,
            _specromet : masks,
            _vectors : vectors,
            _depth : depth,
            _frame_buffer : fb,
            _workers : Vec::new()
        };
        if parallel_draw_calls {
            result.start_workers(3);
        }
        result
    }

    fn start_workers(&mut self, num: u32)
    {
        for i in 0..num {
            let (start_stop_tx, start_stop_rx) = mpsc::channel::<LoopSignal>();
            let (result_tx, result_rx) = mpsc::channel::<SecondaryAutoCommandBuffer>();
            let subpass = self.subpass();
            let device = self._device.clone();
            let queue = self._queue.clone();
            
            
            let worker = std::thread::spawn(move || {
                println!("Запуск потока вызовов отрисовки № {}.", i);
                let mut command_buffer_builder = Some(AutoCommandBufferBuilder::secondary_graphics(
                    device.clone(),
                    queue.family(),
                    CommandBufferUsage::OneTimeSubmit,
                    subpass.clone()
                ).unwrap());
                loop {
                    match start_stop_rx.recv() {
                        Ok(Some((viewport, camera_data, draw_list))) => {
                            let mut last_mesh = -1;
                            let mut last_material = -1;
                            command_buffer_builder = 
                            match command_buffer_builder {
                                Some(mut secondary_cbb) => {
                                    secondary_cbb.set_viewport(0, [viewport]);
                                    for (transform, visual_component) in draw_list
                                    {
                                        let mut component = visual_component.lock().unwrap();
                                        let (mesh_id, material_id) = (component.mesh_id(), component.material_id());
                                        component.on_geometry_pass_secondary(
                                            transform,
                                            camera_data,
                                            subpass.clone(),
                                            &mut secondary_cbb,
                                            mesh_id != last_mesh,
                                            material_id != last_material,
                                        ).unwrap();
                                        (last_mesh, last_material) = (mesh_id, material_id);
                                    }
                                    result_tx.send(secondary_cbb.build().unwrap()).unwrap();
                                    Some(AutoCommandBufferBuilder::secondary_graphics(
                                        device.clone(),
                                        queue.family(),
                                        CommandBufferUsage::OneTimeSubmit,
                                        subpass.clone()
                                    ).unwrap())
                                },
                                None => panic!()
                            };
                        },
                        Ok(None) => {
                            break;
                        },
                        Err(err) => panic!("Ошибка потока вызовов отрисовки № {}: {:?}", i, err)
                    }
                }
                println!("Завершение потока вызовов отрисовки № {}.", i);
            });
            let worker = DrawWorker {
                join_handle: worker,
                loop_signal: start_stop_tx,
                cb_result: result_rx
            };
            self._workers.push(worker);
        }
    }

    fn send_job_to_worker(&self, num: u32, camera_data: ProjectionUniformData, draw_list: DrawList)
    {
        self._workers[num as usize].loop_signal.send(Some((self._frame_buffer.viewport().clone(), camera_data, draw_list))).unwrap();
    }

    fn get_response_from_worker(&self, num: u32) -> SecondaryAutoCommandBuffer
    {
        self._workers[num as usize].cb_result.recv().unwrap()
    }

    pub fn build_geometry_pass(
        &mut self,
        camera_data: ProjectionUniformData,
        mut draw_list: DrawList
    ) -> PrimaryAutoCommandBuffer
    {
        let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
            self._device.clone(),
            self._queue.family(),
            CommandBufferUsage::OneTimeSubmit,
        ).unwrap();
        
        if self._workers.len() > 0 {
            command_buffer_builder.bind_framebuffer(&mut self._frame_buffer, self._geometry_pass.clone(), true).unwrap();
            let mut thr_data = (0..self._workers.len()).map(|_| DrawList::new()).collect::<Vec<_>>();
            while draw_list.len() > 0 {
                for i in 0..thr_data.len() {
                    if draw_list.len() > 0 {
                        thr_data[i].push(draw_list.remove(0));
                    } else {
                        break;
                    }
                }
            }
            for i in 0..thr_data.len() {
                self.send_job_to_worker(i as _, camera_data, thr_data[i].clone());
            }
            for i in 0..thr_data.len() {
                command_buffer_builder.execute_commands(self.get_response_from_worker(i as _)).unwrap();
            }
            command_buffer_builder
                .end_render_pass().unwrap();
        } else {
            command_buffer_builder.bind_framebuffer(&mut self._frame_buffer, self._geometry_pass.clone(), false).unwrap();
            let mut last_mesh = -1;
            let mut last_material = -1;

            for (transform, visual_component) in draw_list
            {
                let mut component = visual_component.lock().unwrap();
                let (mesh_id, material_id) = (component.mesh_id(), component.material_id());
                component.on_geometry_pass(
                    transform,
                    camera_data,
                    self._geometry_pass.clone().first_subpass(),
                    &mut command_buffer_builder,
                    mesh_id != last_mesh,
                    material_id != last_material,
                ).unwrap();
                (last_mesh, last_material) = (mesh_id, material_id);
            }
            command_buffer_builder.end_render_pass().unwrap();
        }
        let result = command_buffer_builder.build().unwrap();
        result
    }

    pub fn albedo(&self) -> &Texture
    {
        &self._albedo
    }

    pub fn normals(&self) -> &Texture
    {
        &self._normals
    }

    pub fn specromet(&self) -> &Texture
    {
        &self._specromet
    }

    pub fn vectors(&self) -> &Texture
    {
        &self._vectors
    }

    pub fn depth(&self) -> &Texture
    {
        &self._depth
    }

    pub fn frame_buffer(&self) -> &Framebuffer
    {
        &self._frame_buffer
    }

    pub fn render_pass(&self) -> &Arc<RenderPass>
    {
        &self._geometry_pass
    }

    pub fn subpass(&self) -> Subpass
    {
        self._geometry_pass.clone().first_subpass()
    }
}
