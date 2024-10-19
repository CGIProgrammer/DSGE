use std::collections::HashMap;
use std::path::Path;

use byteorder::ReadBytesExt;

use crate::components::{light::*, CameraComponent, MeshVisual};
use crate::game_object::*;
use crate::material::MaterialRef;
use crate::mesh::*;
use crate::resource_manager::ResourceManager;
use crate::texture::*;
use crate::types::Mat4;
use crate::utils::read_struct;

#[allow(dead_code)]
struct MaterialStruct {
    diffuse_value: [f32; 4],
    metallic_value: f32,
    specular_value: f32,
    roughness_value: f32,
    emission_value: [f32; 3],
    transp_rough: f32,
    blend_method: i32,
    shadow_method: i32,
}

#[repr(packed)]
#[allow(dead_code)]
struct LightStruct {
    energy: f32,
    color: [f32; 3],
    typenum: u32,
    shadow: u32,
    shadow_mode: u32,
    z_near: f32,
    z_far: f32,
    size: f32,
    inner_angle: f32,
    angle: f32,
}

fn read_string(reader: &mut std::fs::File) -> String {
    let mut bytes: Vec<u8> = Vec::new();
    loop {
        match reader.read_u8() {
            Ok(0u8) => break,
            Ok(byte) => bytes.push(byte),
            Err(_) => break,
        }
    }
    let result = String::from_utf8_lossy(bytes.as_slice()).to_string();
    return result;
}

fn read_texture(
    reader: &mut std::fs::File,
    resource_manager: &mut ResourceManager,
) -> (String, Texture) {
    let name = read_string(reader);
    let filepath = read_string(reader);
    let filepath = filepath
        .replace("./data/textures/", "")
        .replace("data/textures/", "");
    let result = resource_manager.get_texture(&filepath).unwrap();
    println!("Текстура {name} {filepath}, {}x{}, mip {:?}", result.width(), result.height(), result._vk_image_access.mip_levels());
    (name, result)
}

fn read_material(
    reader: &mut std::fs::File,
    textures: &HashMap<String, Texture>,
    resource_manager: &mut ResourceManager,
) -> MaterialRef {
    let name = read_string(reader);
    let fixed_struct: MaterialStruct = read_struct(reader).unwrap();
    let diffuse_texture = read_string(reader);
    let metallic_texture = read_string(reader);
    let specular_texture = read_string(reader);
    let roughness_texture = read_string(reader);
    let emission_texture = read_string(reader);
    let normals_texture = read_string(reader);
    let new_material = resource_manager.new_material(&name);
    {
        let mut mat = new_material.lock();
        mat.set_parameter("diffuse", fixed_struct.diffuse_value.into())
            .unwrap();
        mat.set_parameter("roughness", fixed_struct.roughness_value.into())
            .unwrap();
        mat.set_parameter("specular", fixed_struct.specular_value.into())
            .unwrap();
        mat.set_parameter("glow", fixed_struct.emission_value[0].into())
            .unwrap();
        mat.set_parameter("metallic", fixed_struct.metallic_value.into())
            .unwrap();
        mat.set_parameter("blend_method", fixed_struct.blend_method.into())
            .unwrap();
        mat.set_parameter("shadow_method", fixed_struct.shadow_method.into())
            .unwrap();
        if diffuse_texture != "" {
            let texture = textures.get(&diffuse_texture).unwrap();
            mat.set_parameter("use_diffuse_map", 1.into()).unwrap();
            mat.replace_texture("fDiffuseMap", &texture).unwrap();
        } else {
            mat.set_parameter("use_diffuse_map", 0.into()).unwrap();
        };
        if metallic_texture != "" {
            let texture = textures.get(&metallic_texture).unwrap();
            mat.set_parameter("use_metallic_map", 1.into()).unwrap();
            mat.replace_texture("fMetallicMap", &texture).unwrap();
        } else {
            mat.set_parameter("use_metallic_map", 0.into()).unwrap();
        };
        if normals_texture != "" {
            let texture = textures.get(&normals_texture).unwrap();
            mat.set_parameter("use_normal_map", 1.into()).unwrap();
            mat.replace_texture("fNornalMap", &texture).unwrap();
        } else {
            mat.set_parameter("use_normal_map", 0.into()).unwrap();
        };
        if roughness_texture != "" {
            let texture = textures.get(&roughness_texture).unwrap();
            mat.set_parameter("use_roughness_map", 1.into()).unwrap();
            mat.replace_texture("fRoughnessMap", &texture).unwrap();
        } else {
            mat.set_parameter("use_roughness_map", 0.into()).unwrap();
        };
        if specular_texture != "" {
            let texture = textures.get(&specular_texture).unwrap();
            mat.set_parameter("use_specular_map", 1.into()).unwrap();
            mat.replace_texture("fSpecularMap", &texture).unwrap();
        } else {
            mat.set_parameter("use_specular_map", 0.into()).unwrap();
        };
        if emission_texture != "" {
            let texture = textures.get(&emission_texture).unwrap();
            mat.set_parameter("use_emission_map", 1.into()).unwrap();
            mat.replace_texture("fEmissionMap", &texture).unwrap();
        } else {
            mat.set_parameter("use_emission_map", 0.into()).unwrap();
        };
    }

    new_material
}

fn read_object(
    reader: &mut std::fs::File,
    resource_manager: &mut ResourceManager,
    materials: &HashMap<String, MaterialRef>,
    meshes: &HashMap<String, MeshRef>,
) -> GameObjectRef {
    let name = read_string(reader);
    //println!("Объект \"{}\"", name);
    let _pname = read_string(reader);
    let _pbname = read_string(reader);
    //println!("pname: \"{}\", pbname: \"{}\"", _pname, _pbname);
    let obj_mutex = GameObject::new(name);
    let mut obj = obj_mutex.lock_write();
    let mut transform = read_struct::<[f32; 12], std::fs::File>(reader)
        .unwrap()
        .to_vec();
    transform.extend([0.0, 0.0, 0.0, 1.0]);
    let transform = Mat4::from_vec(transform.to_vec()).transpose();
    let _hidden: bool = reader.read_u8().unwrap() != 0;
    let is_static: bool = reader.read_u8().unwrap() != 0;
    let has_mesh: bool = reader.read_u8().unwrap() != 0;
    let has_camera: bool = reader.read_u8().unwrap() != 0;
    let has_light: bool = reader.read_u8().unwrap() != 0;
    let _has_skeleton: bool = reader.read_u8().unwrap() != 0;
    let _has_physics: bool = reader.read_u8().unwrap() != 0;

    if has_mesh {
        let mesh_name = read_string(reader);
        let material_name = read_string(reader);
        if !materials.contains_key(&material_name) {
            panic!("Материал \"{}\" не найден", material_name);
        }
        if !meshes.contains_key(&mesh_name) {
            panic!("Меш \"{}\" не найден", mesh_name);
        }
        let mesh = meshes[&mesh_name].clone();
        let material = materials[&material_name].clone();
        let mesh_component = MeshVisual::new(mesh, material, true);
        //println!("Тип: полисетка");
        obj.add_component(mesh_component);
    };
    if has_camera {
        let camera_component = CameraComponent::new(1.0, 60.0 * 3.1415926535 / 180.0, 0.1, 30.0);
        //println!("Тип: камера");
        obj.add_component(camera_component);
    };
    if has_light {
        let light_struct: LightStruct = read_struct(reader).unwrap();
        //let resolution: u16 = if light_struct.shadow {512} else {0};
        match light_struct.typenum {
            0 => {
                let point = PointLight::new(
                    light_struct.energy,
                    light_struct.color.into(),
                    light_struct.z_near,
                    light_struct.z_far,
                    match light_struct.shadow_mode {
                        0 => ShadowMapMode::None,
                        1 => ShadowMapMode::Static(resource_manager.point_light_shadow_map_array().dims()[0] as _),
                        2 => ShadowMapMode::FullyDynamic(resource_manager.point_light_shadow_map_array().dims()[0] as _),
                        3 => ShadowMapMode::SemiDynamic(resource_manager.point_light_shadow_map_array().dims()[0] as _),
                        _ => unreachable!(),
                    },
                    resource_manager.command_buffer_father(),
                    resource_manager.allocator().clone()
                );
                obj.add_component(point);
            },
            1 => {
                let sun = SunLight::new(
                    light_struct.size,
                    light_struct.energy,
                    light_struct.color.into(),
                    0.1,
                    100.0,
                    match light_struct.shadow_mode {
                        0 => ShadowMapMode::None,
                        1 => ShadowMapMode::Static(resource_manager.sun_light_shadow_map_array().dims()[0] as _),
                        2 => ShadowMapMode::FullyDynamic(resource_manager.sun_light_shadow_map_array().dims()[0] as _),
                        3 => ShadowMapMode::SemiDynamic(resource_manager.sun_light_shadow_map_array().dims()[0] as _),
                        _ => unreachable!(),
                    },
                    resource_manager.command_buffer_father(),
                    resource_manager.allocator().clone()
                );
                obj.add_component(sun);
            },
            2 => {
                let spotlight = Spotlight::new(
                    light_struct.energy,
                    light_struct.color,
                    light_struct.angle,
                    light_struct.inner_angle,
                    light_struct.z_near,
                    light_struct.z_far,
                    match light_struct.shadow_mode {
                        0 => ShadowMapMode::None,
                        1 => ShadowMapMode::Static(resource_manager.spotlight_shadow_map_array().dims()[0] as _),
                        2 => ShadowMapMode::FullyDynamic(resource_manager.spotlight_shadow_map_array().dims()[0] as _),
                        3 => ShadowMapMode::SemiDynamic(resource_manager.spotlight_shadow_map_array().dims()[0] as _),
                        _ => unreachable!(),
                    },
                    resource_manager.command_buffer_father(),
                    resource_manager.allocator().clone()
                );
                obj.add_component(spotlight);
            },
            unknown_type => panic!("Неподдерживаемый источник света: {unknown_type}."),
        };
    };
    //println!("Location {}, {}, {}", transform[12], transform[13], transform[14]);
    obj.set_static(false);
    if let Some(obj_transform) = obj.transform_mut() {
        obj_transform.local = transform;
        obj_transform.global = transform;
        obj_transform.global = transform;
        obj_transform.global_prev = transform;
    }
    obj.set_static(is_static);
    drop(obj);
    obj_mutex
}

pub(super) fn read_scene<P: AsRef<Path> + ToString + Clone>(
    path: P,
    resource_manager: &mut ResourceManager,
) -> (Vec<GameObjectRef>, Option<GameObjectRef>) {
    //let mut resource_manager = ResourceManager::new(queue.device().clone(), queue.clone(), super_resolution).unwrap();

    let mut reader = match std::fs::File::open(path.clone()) {
        Ok(rdr) => rdr,
        Err(_) => panic!("Сцена {:?} не найдена.", path.as_ref()),
    };
    let textures_count: u32 = read_struct(&mut reader).unwrap();
    println!("Загрузка текстур ({})", textures_count);
    let textures: HashMap<String, Texture> = (0..textures_count)
        .map(|_| read_texture(&mut reader, resource_manager))
        .collect();

    let materials_count: u32 = read_struct(&mut reader).unwrap();
    println!("Загрузка материалов ({})", materials_count);
    let materials: HashMap<String, MaterialRef> = (0..materials_count)
        .map(|_| {
            let material = read_material(&mut reader, &textures, resource_manager);
            let name = material.lock().name().clone();
            (name, material)
        })
        .collect();

    let meshes_count: u32 = read_struct(&mut reader).unwrap();
    println!("Загрузка мешей ({})", meshes_count);
    let mesh_names = (0..meshes_count)
        .map(|_| {
            let name = read_string(&mut reader);
            println!("Меш {name}");
            name.replace("data/mesh/", "")
        })
        .collect::<Vec<_>>();

    let meshes = resource_manager.get_batch_of_meshes(&mesh_names);

    let objects_count: u32 = read_struct(&mut reader).unwrap();
    println!("Загрузка объектов ({})", objects_count);
    let objects: Vec<GameObjectRef> = (0..objects_count)
        .map(|_| read_object(&mut reader, resource_manager, &materials, &meshes))
        .collect();
    let camera = match objects
        .iter()
        .find(|obj| (*obj).lock_write().camera().is_some())
    {
        Some(cam) => Some(cam.clone()),
        None => None,
    };
    resource_manager.flush_futures();
    (objects, camera)
}
