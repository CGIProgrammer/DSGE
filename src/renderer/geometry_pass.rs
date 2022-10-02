use std::cmp::Ordering;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread::JoinHandle;

use num::Integer;
use vulkano::buffer::{
    BufferUsage, CpuAccessibleBuffer, CpuBufferPool, ImmutableBuffer, TypedBufferAccess,
};
use vulkano::command_buffer::CommandBufferUsage;
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, DrawIndexedIndirectCommand, PrimaryAutoCommandBuffer,
    SecondaryAutoCommandBuffer,
};
use vulkano::device::{Device, DeviceOwned, Queue};
use vulkano::image::{ImageLayout, SampleCount};
use vulkano::pipeline::graphics::viewport::Viewport;
use vulkano::render_pass::{
    AttachmentDescription, AttachmentReference, RenderPass, RenderPassCreateInfo, Subpass,
    SubpassDescription,
};
use vulkano::sync::GpuFuture;

use crate::components::visual::MeshVisual;
use crate::components::GOTransformUniform;
use crate::components::ProjectionUniformData;
use crate::framebuffer::{Framebuffer, FramebufferBinder};
use crate::material::MaterialShaderType;
use crate::mesh::MeshView;
use crate::shader::ShaderProgramBinder;
use crate::types::Vec3;
use crate::{texture::*, VULKANO_BUFFER_ATOMIC_SIZE};

pub(super) type DrawList = Vec<(GOTransformUniform, Arc<MeshVisual>)>;
type LoopSignal = Option<(Viewport, ProjectionUniformData, DrawList)>;

struct DrawWorker {
    join_handle: JoinHandle<()>,
    loop_signal: mpsc::Sender<LoopSignal>,
    cb_result: mpsc::Receiver<SecondaryAutoCommandBuffer>,
}

/// Буфер для сохранения результатов прохода геометрии
pub struct GeometryPass {
    _frame_buffer: Framebuffer,
    _device: Arc<Device>,
    _queue: Arc<Queue>,
    _geometry_pass: Arc<RenderPass>,
    _albedo: Texture,    // Цвет поверхности
    _normals: Texture,   // Нормали
    _specromet: Texture, // specromet - specular, roughness, metallic. TODO пока ничем не заполняется
    _vectors: Texture,   // Векторы скорости. TODO пока ничем не заполняется
    _depth: Texture,     // Глубина. TODO пока ничем не заполняется
}

pub(super) fn build_geometry_pass(
    framebuffer: &mut Framebuffer,
    projection_data: ProjectionUniformData,
    draw_list: &DrawList,
    shader_type: MaterialShaderType,
    subpass: Subpass,
    queue: Arc<Queue>,
) -> PrimaryAutoCommandBuffer {
    let device = queue.device();
    let mut draw_list = draw_list.clone();
    let cam_pos = Vec3::new(
        projection_data.transform[12],
        projection_data.transform[13],
        projection_data.transform[14],
    );
    draw_list.sort_by(
        |(at, a), (bt, b)| match a.material_id().cmp(&b.material_id()) {
            Ordering::Equal => match a.mesh().buffer_id().cmp(&b.mesh().buffer_id()) {
                Ordering::Equal => {
                    let a_pos = Vec3::new(at.transform[12], at.transform[13], at.transform[14]);
                    let b_pos = Vec3::new(bt.transform[12], bt.transform[13], bt.transform[14]);
                    let a_vec = cam_pos - a_pos;
                    let b_vec = cam_pos - b_pos;
                    if let Some(ord) = b_vec.dot(&b_vec).partial_cmp(&a_vec.dot(&a_vec)) {
                        ord
                    } else {
                        Ordering::Equal
                    }
                }
                cmp => cmp,
            },
            cmp => cmp,
        },
    );
    let transforms = draw_list
        .iter()
        .map(|(transform, _)| transform.clone())
        .collect::<Vec<_>>();
    let instance_buffer = CpuAccessibleBuffer::from_iter(
        device.clone(),
        BufferUsage::vertex_buffer(),
        false,
        transforms.clone(),
    )
    .unwrap();

    //println!("Camera location {:?}", &projection_data.transform[12..15]);
    let diic_size = std::mem::size_of::<DrawIndexedIndirectCommand>();
    let indc_alignemt = diic_size.lcm(&(VULKANO_BUFFER_ATOMIC_SIZE)) / diic_size;
    let render_pass = subpass.render_pass().clone();
    let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
        device.clone(),
        queue.family(),
        CommandBufferUsage::OneTimeSubmit,
    )
    .unwrap();

    command_buffer_builder
        .bind_framebuffer(framebuffer, render_pass, false)
        .unwrap();
    let mut last_shader = 0u64;
    //let mut repeated_objects = Vec::with_capacity(draw_list.len());
    let mut instance_counter = 0u32;
    let mut first_instance_index = 0u32;
    //let (mut shader_program, mut uniform_buffer);
    let (_, mut camera_uniform_buffer) = {
        let (shader_program, mut camera_uniform_buffer) = draw_list[0]
            .1
            .material()
            .lock()
            .shader(&shader_type, subpass.clone())
            .clone();
        camera_uniform_buffer.uniform(
            projection_data.clone(),
            crate::material::SHADER_CAMERA_SET,
            0,
        );
        camera_uniform_buffer.build_uniform_sets(&[crate::material::SHADER_CAMERA_SET]);
        camera_uniform_buffer.clear_uniform_set(crate::material::SHADER_MATERIAL_DATA_SET);
        camera_uniform_buffer.clear_uniform_set(crate::material::SHADER_TEXTURE_SET);
        (shader_program, camera_uniform_buffer.clone())
    };
    let mut draw_call_count = 0;
    let mut shader_switches = 0;
    let mut objects = 0;
    let mut indirect_commands = Vec::<DrawIndexedIndirectCommand>::with_capacity(draw_list.len());
    let last_index = draw_list.len() - 1;
    let mut mesh_group_instance_count = 0;
    let indirect_args_pool: CpuBufferPool<DrawIndexedIndirectCommand> =
        CpuBufferPool::new(device.clone(), BufferUsage::all());
    for (i, (_, visual)) in draw_list.iter().enumerate() {
        let mesh = visual.mesh().clone();
        let material = visual.material().clone();
        let new_material_group = if i == 0 {
            true
        } else {
            material.box_id() != draw_list[i - 1].1.material().box_id()
        };
        let new_mesh_group = if i == 0 {
            true
        } else {
            mesh.buffer_id() != draw_list[i - 1].1.mesh().buffer_id()
        };
        let new_instance_group = if i == 0 {
            true
        } else {
            mesh.ref_id() != draw_list[i - 1].1.mesh().ref_id()
        };
        let end_instance_group = if i == last_index {
            true
        } else {
            mesh.ref_id() != draw_list[i + 1].1.mesh().ref_id()
        };
        let end_mesh_group = if i == last_index {
            true
        } else {
            mesh.buffer_id() != draw_list[i + 1].1.mesh().buffer_id()
        };
        let end_material_group = if i == last_index {
            true
        } else {
            material.box_id() != draw_list[i + 1].1.material().box_id()
        };

        if new_material_group {
            let (shd, mut uni) = visual
                .material().lock()
                .shader(&shader_type, subpass.clone()).clone();
            if shd.hash() != last_shader {
                command_buffer_builder
                    .bind_shader_program(&shd).unwrap()
                    .bind_shader_uniforms(&mut camera_uniform_buffer, false).unwrap();
                shader_switches += 1;
            }
            command_buffer_builder
                .bind_shader_uniforms(&mut uni, false)
                .unwrap();
            last_shader = shd.hash();
        }
        if new_mesh_group {
            indirect_commands.clear();
            mesh_group_instance_count = 0;
            command_buffer_builder
                .bind_vertex_buffers(0, (mesh.vertex_buffer().clone(), instance_buffer.clone()))
                .bind_index_buffer(mesh.index_buffer().clone());
        }
        if new_instance_group {
            first_instance_index = i as _;
            instance_counter = 0;
        }
        instance_counter += 1;
        mesh_group_instance_count += 1;
        objects += 1;
        if end_instance_group {
            indirect_commands.push(
                visual
                    .mesh()
                    .indirect_command(first_instance_index, instance_counter),
            );
        }
        if end_mesh_group || end_material_group {
            if indirect_commands.len() <= 0 {
                for (i, ic) in indirect_commands.iter().enumerate() {
                    command_buffer_builder
                        .draw_indexed(
                            ic.index_count,
                            ic.instance_count,
                            ic.first_index,
                            ic.vertex_offset as _,
                            ic.first_instance,
                            i == 0,
                        )
                        .unwrap();
                    draw_call_count += 1;
                }
            } else {
                /*for _ in 0..(indirect_commands.len() % indc_alignemt) {
                    indirect_commands.push(DrawIndexedIndirectCommand::default());
                }*/
                let indirect_buffer = indirect_args_pool.chunk(indirect_commands.clone()).unwrap();
                command_buffer_builder.draw_indexed_indirect(indirect_buffer).unwrap();
                draw_call_count += 1;
            }
        }
    }
    //println!("draw_call_count {draw_call_count}, objects_count {objects}, shader_switches {shader_switches}");
    command_buffer_builder.end_render_pass().unwrap();
    command_buffer_builder.build().unwrap()
}

impl GeometryPass {
    pub fn new(width: u16, height: u16, queue: Arc<Queue>) -> Self {
        //println!("Создание нового G-буфера {}x{}", width, height);
        let device = queue.device();
        let formats = [
            TexturePixelFormat::R16G16B16A16_UNORM,
            TexturePixelFormat::R8G8B8A8_SNORM,
            TexturePixelFormat::R8G8B8A8_UNORM,
            TexturePixelFormat::R16G16B16A16_SNORM,
            TexturePixelFormat::D16_UNORM,
        ];

        let mut fb = Framebuffer::new(width, height);
        let dims = TextureDimensions::Dim2d {
            width: width as _,
            height: height as _,
            array_layers: 1,
        };
        let mut albedo = Texture::new_empty("gAlbedo", dims, formats[0], device.clone()).unwrap();
        let mut normals = Texture::new_empty("gNormals", dims, formats[1], device.clone()).unwrap();
        let mut masks = Texture::new_empty("gMasks", dims, formats[2], device.clone()).unwrap();
        let mut vectors = Texture::new_empty("gVectors", dims, formats[3], device.clone()).unwrap();
        let depth = Texture::new_empty("gDepth", dims, formats[4], device.clone()).unwrap();

        albedo.clear_color(queue.clone());
        albedo.set_horizontal_address(crate::texture::TextureRepeatMode::ClampToEdge);
        albedo.set_vertical_address(crate::texture::TextureRepeatMode::ClampToEdge);
        albedo.update_sampler();
        normals.clear_color(queue.clone());
        masks.clear_color(queue.clone());
        vectors.clear_color(queue.clone());

        fb.add_color_attachment(&albedo, [0.0, 0.0, 0.0, 0.0].into())
            .unwrap();
        fb.add_color_attachment(&normals, [0.0, 0.0, 0.0, 0.0].into())
            .unwrap();
        fb.add_color_attachment(&masks, [0.0, 0.0, 0.0, 0.0].into())
            .unwrap();
        fb.add_color_attachment(&vectors, [0.0, 0.0, 0.0, 0.0].into())
            .unwrap();
        fb.set_depth_attachment(&depth, 1.0.into());

        let mut attachments = Vec::new();
        for fmt in formats {
            let img_layout = match fmt.is_depth() {
                true => ImageLayout::DepthStencilAttachmentOptimal,
                false => ImageLayout::ColorAttachmentOptimal,
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
                    Some(AttachmentReference {
                        attachment: 0,
                        layout: ImageLayout::ColorAttachmentOptimal,
                        ..Default::default()
                    }),
                    Some(AttachmentReference {
                        attachment: 1,
                        layout: ImageLayout::ColorAttachmentOptimal,
                        ..Default::default()
                    }),
                    Some(AttachmentReference {
                        attachment: 2,
                        layout: ImageLayout::ColorAttachmentOptimal,
                        ..Default::default()
                    }),
                    Some(AttachmentReference {
                        attachment: 3,
                        layout: ImageLayout::ColorAttachmentOptimal,
                        ..Default::default()
                    }),
                ],
                depth_stencil_attachment: Some(AttachmentReference {
                    attachment: 4,
                    layout: ImageLayout::DepthStencilAttachmentOptimal,
                    ..Default::default()
                }),
                ..Default::default()
            }],
            ..Default::default()
        };

        let result = Self {
            _device: device.clone(),
            _queue: queue.clone(),
            _geometry_pass: RenderPass::new(device.clone(), desc).unwrap(),
            _albedo: albedo,
            _normals: normals,
            _specromet: masks,
            _vectors: vectors,
            _depth: depth,
            _frame_buffer: fb,
        };
        result
    }

    pub fn build_geometry_pass(
        &mut self,
        camera_data: ProjectionUniformData,
        draw_list: DrawList,
    ) -> PrimaryAutoCommandBuffer {
        build_geometry_pass(
            &mut self._frame_buffer,
            camera_data.clone(),
            &draw_list,
            MaterialShaderType::Base,
            self._geometry_pass.clone().first_subpass(),
            self._queue.clone(),
        )
    }

    pub fn albedo(&self) -> &Texture {
        &self._albedo
    }

    pub fn normals(&self) -> &Texture {
        &self._normals
    }

    pub fn specromet(&self) -> &Texture {
        &self._specromet
    }

    pub fn vectors(&self) -> &Texture {
        &self._vectors
    }

    pub fn depth(&self) -> &Texture {
        &self._depth
    }

    pub fn frame_buffer(&self) -> &Framebuffer {
        &self._frame_buffer
    }

    pub fn render_pass(&self) -> &Arc<RenderPass> {
        &self._geometry_pass
    }

    pub fn subpass(&self) -> Subpass {
        self._geometry_pass.clone().first_subpass()
    }
}
