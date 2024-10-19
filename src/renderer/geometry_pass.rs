use std::cmp::Ordering;
use std::default;
use std::mem::size_of;
use std::sync::Arc;

use vulkano::buffer::allocator::SubbufferAllocatorCreateInfo;
use vulkano::buffer::{BufferUsage, allocator::SubbufferAllocator};

use vulkano::command_buffer::{
    DrawIndexedIndirectCommand, PrimaryAutoCommandBuffer, SubpassEndInfo,
};
use vulkano::descriptor_set::allocator::{StandardDescriptorSetAllocator, StandardDescriptorSetAllocatorCreateInfo};
use vulkano::device::{Device, DeviceOwned, Queue};
use vulkano::image::{ImageLayout, SampleCount};
use vulkano::memory::allocator::{StandardMemoryAllocator, MemoryTypeFilter};
use vulkano::render_pass::{
    AttachmentDescription, AttachmentReference, RenderPass, RenderPassCreateInfo, Subpass,
    SubpassDescription,
};

use crate::command_buffer::{CommandBufferFather, new_cpu_buffer_from_iter};

use crate::components::visual::MeshVisual;
use crate::components::GOTransformUniform;
use crate::components::ProjectionUniformData;
use crate::framebuffer::{Framebuffer, FramebufferBinder};

use crate::material::MaterialShaderProgramType;
use crate::resource_manager::ResourceManager;
use crate::shader::ShaderProgramBinder;
use crate::time::UniformTime;
use crate::types::Vec4;
use crate::types::{ArrayInto, Mat4, Vec3};
use crate::{resource_manager, texture::*};

use super::{bump_memory_allocator_new_default, BumpMemoryAllocator};

pub(super) type DrawList = Vec<(GOTransformUniform, Arc<MeshVisual>)>;

/*pub trait Cullable {
    fn check_in_frustum(
        &self,
        projection_data: ProjectionUniformData,
        owner_transform: Mat4,
    ) -> bool;
}

impl Cullable for MeshVisual {
    fn check_in_frustum(
        &self,
        projection_data: ProjectionUniformData,
        owner_transform: Mat4,
    ) -> bool {
        let points = self.bbox_corners();
        let model_view_projection = projection_data.full_matrix() * owner_transform;
        fast_check_figure_in_matrix(model_view_projection, &points)
    }
}

impl Cullable for Light {
    fn check_in_frustum(
        &self,
        projection_data: ProjectionUniformData,
        owner_transform: Mat4,
    ) -> bool {
        let points = self.lock().unwrap().bbox_corners();
        let model_view_projection = projection_data.full_matrix() * owner_transform;
        fast_check_figure_in_matrix(model_view_projection, &points)
    }
}*/

pub fn check_in_frustum(points: &[Vec3], projection_data: ProjectionUniformData, object_transform: Mat4) -> bool
{
    let model_view_projection = projection_data.full_matrix() * object_transform;
    fast_check_figure_in_matrix(model_view_projection, &points)
}

/// Буфер для сохранения результатов прохода геометрии
pub struct GeometryPass {
    _command_buffer_father: CommandBufferFather,
    _allocator: Arc<BumpMemoryAllocator>,
    _ds_allocator: Arc<StandardDescriptorSetAllocator>,
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

fn fast_check_figure_in_matrix(model_view_projection: Mat4, points: &[Vec3]) -> bool {
    let mut back = 0;
    let mut front = 0;
    let mut left = 0;
    let mut right = 0;
    let mut top = 0;
    let mut bottom = 0;
    for point in points {
        let p = model_view_projection * Vec4::new(point.x, point.y, point.z, 1.0);
        let dist = p.w;
        let p = p.xyz() / dist;
        if dist < 0.0 {
            back += 1;
            continue;
        }
        if p.z > 1.0 {
            front += 1;
        }
        if p.x < -1.0 {
            left += 1;
        }
        if p.x > 1.0 {
            right += 1;
        }
        if p.y < -1.0 {
            bottom += 1;
        }
        if p.y > 1.0 {
            top += 1;
        }
    }
    if back == points.len() {
        return false;
    }
    if left == points.len() {
        return false;
    }
    if right == points.len() {
        return false;
    }
    if top == points.len() {
        return false;
    }
    if bottom == points.len() {
        return false;
    }
    if front == points.len() {
        return false;
    }
    true
}

pub fn cull_objects(projection_data: ProjectionUniformData, draw_list: &DrawList) -> DrawList {
    //let matrix = projection_data.transform_inverted * projection_data.projection;
    draw_list
        .iter()
        .filter_map(|(transform, visual)| {
            if check_in_frustum(&visual.bbox_corners(), projection_data, transform.transform.into_mat4()) {
                Some((*transform, visual.clone()))
            } else {
                None
            }
            /*let corners = visual.mesh().bbox_corners();
            let model = Mat4::from_iterator(transform.transform);
            let model_trajection = trajection * model;
            if fast_check_figure_in_matrix(model_trajection, &corners) {
                return Some((*transform, visual.clone()));
            } else {
                return None
            }*/
        })
        .collect()
}

pub(super) fn build_geometry_pass(
    framebuffer: &mut Framebuffer,
    projection_data: ProjectionUniformData,
    timer: UniformTime,
    draw_list: DrawList,
    shader_type: MaterialShaderProgramType,
    subpass: Subpass,
    command_buffer_father: &CommandBufferFather,
    allocator: Arc<BumpMemoryAllocator>,
    ds_allocator: Arc<StandardDescriptorSetAllocator>,
) -> Result<Arc<PrimaryAutoCommandBuffer>, String> {
    let mut draw_list = cull_objects(projection_data, &draw_list);
    /*let cam_pos = Vec3::new(
        projection_data.transform[12],
        projection_data.transform[13],
        projection_data.transform[14],
    );*/
    if draw_list.is_empty() {
        let render_pass = subpass.render_pass().clone();
        let mut cbb = command_buffer_father.new_primary()?;
        cbb
            .bind_framebuffer(framebuffer, render_pass, false, false)
            .unwrap()
            .end_render_pass(SubpassEndInfo::default())
            .unwrap();
        let cb = cbb.build_buffer();
        /*let cb = add_to_new_primary_command_buffer(queue.clone(), |command_buffer_builder| {
            command_buffer_builder
                .bind_framebuffer(framebuffer, render_pass, false, false)
                .unwrap()
                .end_render_pass()
                .unwrap();
        })?;*/
        return Ok(cb);
    }
    draw_list.sort_by(|(_, a), (_, b)| {
        let a_hash = a.shader_hash(shader_type);
        let b_hash = b.shader_hash(shader_type);
        match a_hash.cmp(&b_hash) {
            Ordering::Equal => (),
            nequal => return nequal,
        };
        match a.material_id().cmp(&b.material_id()) {
            Ordering::Equal => match a.mesh().buffer_id().cmp(&b.mesh().buffer_id()) {
                Ordering::Equal => {
                    /*let a_pos = Vec3::new(at.transform[12], at.transform[13], at.transform[14]);
                    let b_pos = Vec3::new(bt.transform[12], bt.transform[13], bt.transform[14]);
                    let a_vec = cam_pos - a_pos;
                    let b_vec = cam_pos - b_pos;
                    if let Some(ord) = b_vec.dot(&b_vec).partial_cmp(&a_vec.dot(&a_vec)) {
                        ord
                    } else {
                        Ordering::Equal
                    }*/
                    Ordering::Equal
                }
                cmp => cmp,
            },
            cmp => cmp,
        }
    });
    let transforms = draw_list
        .iter()
        .map(|(transform, _)| transform.clone())
        .collect::<Vec<_>>();

        //println!("Camera location {:?}", &projection_data.transform[12..15]);
    let _diic_size = std::mem::size_of::<DrawIndexedIndirectCommand>();
    let render_pass = subpass.render_pass().clone();

    let pcbb = command_buffer_father.new_primary_instant(|command_buffer_builder| {
        /*let instance_buffer = command_buffer_builder
            .new_buffer_on_device_from_iter(
                BufferUsage::VERTEX_BUFFER,
                allocator.as_ref(),
                transforms.clone()
            ).unwrap();*/
        
        let instance_buffer = new_cpu_buffer_from_iter(
            BufferUsage::VERTEX_BUFFER,
            allocator.clone(),
            transforms.clone()
        ).unwrap();

        command_buffer_builder
            .bind_framebuffer(framebuffer, render_pass, false, false)
            .unwrap();

        let mut last_shader = 0u64;
        let mut instance_counter = 0u32;
        let mut first_instance_index = 0u32;
        let (_, mut camera_uniform_buffer) = {
            let (shader_program, mut camera_uniform_buffer) = draw_list[0]
                .1
                .material()
                .lock()
                .use_in_subpass(
                    command_buffer_father,
                    allocator.clone(),
                    ds_allocator.clone(),
                    &shader_type,
                    subpass.clone()
                )
                .clone();
            camera_uniform_buffer.uniform(
                allocator.clone(),
                projection_data.clone(),
                crate::material::SHADER_CAMERA_SET,
                0,
            );
            camera_uniform_buffer.uniform(
                allocator.clone(),
                framebuffer.render_resolution(),
                crate::material::SHADER_CAMERA_SET,
                1,
            );
            //dbg!(framebuffer.render_resolution());
            camera_uniform_buffer.uniform(allocator.clone(), timer, crate::material::SHADER_CAMERA_SET, 2);
            camera_uniform_buffer
                .build_uniform_sets(ds_allocator.clone(), &[crate::material::SHADER_CAMERA_SET])
                .unwrap();
            camera_uniform_buffer.clear_uniform_set(crate::material::SHADER_MATERIAL_DATA_SET);
            camera_uniform_buffer.clear_uniform_set(crate::material::SHADER_TEXTURE_SET);
            (shader_program, camera_uniform_buffer.clone())
        };
        let mut _draw_call_count = 0;
        let mut _shader_switches = 0;
        let mut _vbo_switches = 0;
        let mut _objects = 0;
        let mut indirect_commands =
            Vec::<DrawIndexedIndirectCommand>::with_capacity(draw_list.len());
        let last_index = draw_list.len() - 1;
        let indirect_args_pool = SubbufferAllocator::new(
            allocator.clone(),
            SubbufferAllocatorCreateInfo {
                buffer_usage: BufferUsage::INDIRECT_BUFFER,
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
        );
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
            unsafe {

                if new_material_group {
                    let (shd, mut uni) = visual
                        .material()
                        .lock()
                        .use_in_subpass(command_buffer_father, allocator.clone(), ds_allocator.clone(), &shader_type, subpass.clone())
                        .clone();
                    if shd.hash() != last_shader {
                        command_buffer_builder
                            .bind_shader_program(&shd)
                            .unwrap()
                            .bind_shader_uniforms(ds_allocator.clone(), &mut camera_uniform_buffer, false)
                            .unwrap();
                        _shader_switches += 1;
                        last_shader = shd.hash();
                    }
                    command_buffer_builder
                        .bind_shader_uniforms(ds_allocator.clone(), &mut uni, false)
                        .unwrap();
                }
                if new_mesh_group {
                    indirect_commands.clear();
                    command_buffer_builder
                        .bind_vertex_buffers_unchecked(0, (mesh.vertex_buffer().clone(), instance_buffer.clone()))
                        .bind_index_buffer_unchecked(mesh.index_buffer().clone());
                }
                if new_instance_group {
                    first_instance_index = i as _;
                    instance_counter = 0;
                }
                instance_counter += 1;
                _objects += 1;

                if end_instance_group {
                    indirect_commands.push(
                        visual
                            .mesh()
                            .indirect_command(first_instance_index, instance_counter),
                    );
                }
                if end_mesh_group {
                    _vbo_switches += 1;
                }
                if end_mesh_group || end_material_group {
                    if indirect_commands.len() <= 1 {
                        for (_, ic) in indirect_commands.iter().enumerate() {
                            command_buffer_builder
                                .draw_indexed_unchecked(
                                    ic.index_count,
                                    ic.instance_count,
                                    ic.first_index,
                                    ic.vertex_offset as _,
                                    ic.first_instance,
                                );
                            _draw_call_count += 1;
                        }
                    } else {
                        // Для помойных интеловских видюх. На остальных ошибка выравнивания по 64 байта не возникает.
                        /*while (indirect_commands.len() * diic_size % VULKANO_BUFFER_ATOMIC_SIZE) != 0 {
                            indirect_commands.push(DrawIndexedIndirectCommand::default());
                        }*/
                        let indirect_buffer = indirect_args_pool
                            .allocate_slice(indirect_commands.len() as _).unwrap();
                        let draw_count = indirect_buffer.len() as u32;
                        let stride = size_of::<DrawIndexedIndirectCommand>() as u32;
                        indirect_buffer
                            .write().unwrap()
                            .copy_from_slice(&indirect_commands);
                        command_buffer_builder
                            .draw_indexed_indirect_unchecked(indirect_buffer, draw_count, stride);
                        _draw_call_count += 1;
                    }
                    indirect_commands.clear();
                }
            };
        }
        // #[cfg(debug_assertions)]
        // {
        //     println!("draw_call_count {_draw_call_count}, objects_count {_objects}, shader_switches {_shader_switches}, vbo_switches {_vbo_switches}");
        // }
        command_buffer_builder.end_render_pass(SubpassEndInfo::default()).unwrap();
    })?.1;
    Ok(pcbb)
}

impl GeometryPass {
    pub fn new(width: u16, height: u16, resource_manager: &ResourceManager) -> Self {
        //println!("Создание нового G-буфера {}x{}", width, height);
        let command_buffer_father = CommandBufferFather::new(resource_manager.queue().clone());
        let allocator = Arc::new(bump_memory_allocator_new_default(resource_manager.device().clone()));
        let ds_allocator = Arc::new(StandardDescriptorSetAllocator::new(
            resource_manager.device().clone(),
            StandardDescriptorSetAllocatorCreateInfo::default()
        ));
        let queue = command_buffer_father.queue().clone();
        let device = queue.device();
        let formats = [
            TexturePixelFormat::R8G8B8A8_UNORM,
            TexturePixelFormat::R16G16B16A16_SNORM,
            TexturePixelFormat::R8G8B8A8_UNORM,
            TexturePixelFormat::R16G16B16A16_SNORM,
            TexturePixelFormat::D16_UNORM,
        ];

        let mut fb = Framebuffer::new(width, height);
        let dims = [width as u32, height as u32, 1];
        let mut albedo = Texture::new(
            "gAlbedo",
            dims,
            false,
            TextureViewType::Dim2d,
            formats[0],
            TextureUseCase::Attachment,
            resource_manager.allocator().clone(),
        )
        .unwrap();
        let normals = Texture::new(
            "gNormals",
            dims,
            false,
            TextureViewType::Dim2d,
            formats[1],
            TextureUseCase::Attachment,
            resource_manager.allocator().clone(),
        )
        .unwrap();
        let masks = Texture::new(
            "gMasks",
            dims,
            false,
            TextureViewType::Dim2d,
            formats[2],
            TextureUseCase::Attachment,
            resource_manager.allocator().clone(),
        )
        .unwrap();
        let vectors = Texture::new(
            "gVectors",
            dims,
            false,
            TextureViewType::Dim2d,
            formats[3],
            TextureUseCase::Attachment,
            resource_manager.allocator().clone(),
        )
        .unwrap();
        let depth = Texture::new(
            "gDepth",
            dims,
            false,
            TextureViewType::Dim2d,
            formats[4],
            TextureUseCase::Attachment,
            resource_manager.allocator().clone(),
        )
        .unwrap();

        //albedo.clear_color(queue.clone());
        albedo.set_address_mode([crate::texture::TextureRepeatMode::ClampToEdge; 3]);
        //normals.clear_color(queue.clone());
        //masks.clear_color(queue.clone());
        //vectors.clear_color(queue.clone());

        fb.add_color_attachment(&albedo, Some([0.0, 0.0, 0.0, 0.0].into()))
            .unwrap();
        fb.add_color_attachment(&normals, Some([0.0, 0.0, 0.0, 0.0].into()))
            .unwrap();
        fb.add_color_attachment(&masks, Some([0.0, 0.0, 0.0, 0.0].into()))
            .unwrap();
        fb.add_color_attachment(&vectors, Some([0.0, 0.0, 0.0, 0.0].into()))
            .unwrap();
        fb.set_depth_attachment(&depth, Some(1.0.into()));

        let mut attachments = Vec::new();
        for fmt in formats {
            let (final_layout, initial_layout) = match fmt.is_depth() {
                true => (
                    ImageLayout::DepthStencilAttachmentOptimal,
                    ImageLayout::DepthStencilReadOnlyOptimal,
                ),
                false => (ImageLayout::ColorAttachmentOptimal, ImageLayout::ShaderReadOnlyOptimal),
            };
            let att = AttachmentDescription {
                format: fmt.vk_format(),
                samples: SampleCount::Sample1,
                load_op: vulkano::render_pass::AttachmentLoadOp::Clear,
                store_op: vulkano::render_pass::AttachmentStoreOp::Store,
                stencil_load_op: None,
                stencil_store_op: None,
                initial_layout: initial_layout,
                final_layout: final_layout,
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
            _command_buffer_father: command_buffer_father,
            _allocator: allocator,
            _ds_allocator: ds_allocator,
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
        timer: UniformTime,
        draw_list: DrawList,
    ) -> Result<Arc<PrimaryAutoCommandBuffer>, String> {
        build_geometry_pass(
            &mut self._frame_buffer,
            camera_data.clone(),
            timer,
            draw_list,
            MaterialShaderProgramType::base_gbuffer(),
            self._geometry_pass.clone().first_subpass(),
            &self._command_buffer_father,
            self._allocator.clone(),
            self._ds_allocator.clone()
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
