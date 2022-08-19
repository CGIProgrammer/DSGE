use std::sync::Arc;
use std::path::Path;
use std::collections::HashMap;

use vulkano::device::Device;
use byteorder::ReadBytesExt;

use crate::mesh::*;
use crate::material::{MaterialRef, MaterialBuilder};
use crate::texture::*;
use crate::game_object::*;
use crate::types::Mat4;
use crate::components::{CameraComponent, MeshVisual, light::*};
use crate::utils::read_struct;

#[allow(dead_code)]
struct MaterialStruct
{
    diffuse_value: [f32; 3],
    metallic_value: f32,
    specular_value: f32,
    roughness_value: f32,
    emission_value: [f32; 3],
    transparency: f32,
    transp_rough: f32
}

#[repr(packed)]
#[allow(dead_code)]
struct LightStruct
{
    energy : f32,
    color: [f32; 3],
    typenum : u8,
    shadow : bool,
    znear : f32,
    zfar : f32,
    inner_angle : f32,
    angle : f32,
}

fn read_string(reader: &mut std::fs::File) -> String
{
    let mut bytes : Vec<u8> = Vec::new();
    loop {
        match reader.read_u8() {
            Ok(0u8) => break,
            Ok(byte) => bytes.push(byte),
            Err(_) => break
        }
    };
    let result = String::from_utf8_lossy(bytes.as_slice()).to_string();
    return result;
}

fn read_texture(reader: &mut std::fs::File, queue: Arc<vulkano::device::Queue>) -> (String, Texture)
{
    let name = read_string(reader);
    println!("Текстура \"{}\"", name);
    let filepath = read_string(reader);
    let mut texture = Texture::from_file(queue, filepath).unwrap();
    texture.set_anisotropy(None);
    texture.set_horizontal_address(TextureRepeatMode::Repeat);
    texture.set_vertical_address(TextureRepeatMode::Repeat);
    texture.update_sampler();
    (name, texture)
}

fn read_material(reader: &mut std::fs::File, queue: Arc<vulkano::device::Queue>, textures: &HashMap<String, Texture>) -> MaterialRef
{
    let name = read_string(reader);
    println!("Материал \"{}\"", name);
    let fixed_struct : MaterialStruct = read_struct(reader).unwrap();
    let device = queue.device().clone();
    let diffuse_texture = read_string(reader);
    let _metallic_texture = read_string(reader);
    let _specular_texture = read_string(reader);
    let _roughness_texture = read_string(reader);
    let _emission_texture = read_string(reader);
    let _normals_texture = read_string(reader);
    let mut material = MaterialBuilder::start(name.as_str(), device.clone());
    material
        .add_numeric_parameter("diffuse", fixed_struct.diffuse_value.into())
        .add_numeric_parameter("roughness", fixed_struct.roughness_value.into())
        .add_numeric_parameter("glow", fixed_struct.emission_value[0].into())
        .add_numeric_parameter("metallic", fixed_struct.metallic_value.into());
    if diffuse_texture != "" {
        let texture = textures[&diffuse_texture].clone();
        material
            .define("diffuse_map", "fDiffuseMap")
            .add_texture("fDiffuseMap", &texture);
    }
    material.build_mutex(device)
}

fn read_object(
    reader: &mut std::fs::File,
    materials: &HashMap<String, MaterialRef>,
    meshes: &HashMap<String, MeshRef>,
    device: Arc<Device>
) -> GameObjectRef
{
    let name = read_string(reader);
    println!("Объект \"{}\"", name);
    let _pname = read_string(reader);
    let _pbname = read_string(reader);
    //println!("pname: \"{}\", pbname: \"{}\"", _pname, _pbname);
    let obj_mutex = GameObject::new(name);
    let mut obj = obj_mutex.lock_write();
    let mut transform = read_struct::<[f32; 12], std::fs::File>(reader).unwrap().to_vec();
    transform.extend([0.0, 0.0, 0.0, 1.0]);
    let transform = Mat4::from_vec(transform.to_vec()).transpose();
    let _hidden: bool = reader.read_u8().unwrap() != 0;
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
        println!("Тип: полисетка");
        obj.add_component(mesh_component);
    };
    if has_camera {
        let camera_component = CameraComponent::new(1.0, 60.0 * 3.1415926535 / 180.0, 0.1, 100.0);
        println!("Тип: камера");
        obj.add_component(camera_component);
    };
    if has_light {
        let light_struct: LightStruct = read_struct(reader).unwrap();
        let resolution: u16 = if light_struct.shadow {512} else {0};
        let light = match light_struct.typenum {
            0 => Light::Point(PointLight::new(light_struct.energy, light_struct.color, resolution, device.clone())),
            1 => Light::Sun(SunLight::new(10.0, light_struct.energy, light_struct.color, resolution, device.clone())),
            2 => Light::Spot(SpotLight::new(
                light_struct.energy,
                light_struct.color,
                light_struct.angle,
                light_struct.inner_angle,
                light_struct.znear,
                light_struct.zfar,
                resolution,
                device.clone()
            )),
            _ => panic!("Неподдерживаемый источник света")
        };
        let zfar = light_struct.zfar;
        let znear = light_struct.znear;
        println!("Тип: свет ({}), zNear {}, zFar {}", light.ty(), znear, zfar);
        obj.add_component(light);
    };
    println!("Location {}, {}, {}", transform[12], transform[13], transform[14]);
    let obj_transform = obj.transform_mut();
    obj_transform.local = transform;
    obj_transform.global = transform;
    obj_transform.global = transform;
    obj_transform.global_prev = transform;
    drop(obj);
    obj_mutex
}

pub(super) fn read_scene<P : AsRef<Path> + ToString>(path: P, queue: Arc<vulkano::device::Queue>) -> (Vec<GameObjectRef>, Option<GameObjectRef>)
{
    let mut reader = std::fs::File::open(path).unwrap();
    let device = queue.device();
    let textures_count: u32 = read_struct(&mut reader).unwrap();
    println!("Загрузка текстур ({})", textures_count);
    let textures: HashMap<String, Texture> = (0..textures_count).map(|_| read_texture(&mut reader, queue.clone())).collect();

    let materials_count: u32 = read_struct(&mut reader).unwrap();
    println!("Загрузка материалов ({})", materials_count);
    let materials: HashMap<String, MaterialRef> = (0..materials_count).map(
        |_| {
            let material = read_material(&mut reader, queue.clone(), &textures);
            let name = material.lock().name().clone();
            (name, material)
        }
    ).collect();

    let meshes_count: u32 = read_struct(&mut reader).unwrap();
    println!("Загрузка мешей ({})", meshes_count);
    let meshes: HashMap<String, MeshRef> = (0..meshes_count).map(
        |_| {
            let mesh_name = read_string(&mut reader);
            let mesh_path = format!("data/mesh/{}", mesh_name);
            println!("Меш {}", mesh_name);
            let mut mesh_builder = Mesh::builder(mesh_name.as_str());
            mesh_builder.push_from_file(mesh_path.as_str()).unwrap();
            (mesh_name, mesh_builder.build_mutex(queue.clone()).unwrap())
        }
    ).collect();
    
    let objects_count : u32 = read_struct(&mut reader).unwrap();
    println!("Загрузка объектов ({})", objects_count);
    let objects: Vec<GameObjectRef> = (0..objects_count).map(|_| read_object(&mut reader, &materials, &meshes, device.clone())).collect();
    let camera = match objects.iter().find(|obj| (*obj).lock_write().camera().is_some())
    {
        Some(cam) => Some(cam.clone()),
        None => None
    };

    (objects, camera)
}