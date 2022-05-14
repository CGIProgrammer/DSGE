extern crate winit;
extern crate vulkano;
extern crate bytemuck;
extern crate half;

mod teapot;
mod time;

use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::Version;

#[macro_use]
mod shader;
mod mesh;
mod types;
mod glenums;
mod utils;
mod references;
mod framebuffer;
mod texture;
mod material;
#[macro_use]
mod game_object;
mod renderer;
mod components;

use mesh::*;
use texture::*;
use game_object::*;

trait Radian
{
    fn rad(&self) -> Self;
}

impl Radian for f32
{
    fn rad(&self) -> Self
    {
        self*3.1415926535/180.0
    }
}

use std::collections::HashMap;

fn main() {
    let required_extensions = vulkano_win::required_extensions();
        let vk_instance = Instance::new(InstanceCreateInfo {
            enabled_extensions: required_extensions,
            max_api_version: Some(Version::major_minor(1, 2)),
            ..Default::default()
        }).unwrap();
    let mut renderer = renderer::Renderer::offscreen(vk_instance, [1920, 1080]);
    let mut timer = time::Timer::new();

    let mut img = Texture::from_file(renderer.queue().clone(), "data/texture/image_img.dds").unwrap();
    img.set_anisotropy(Some(16.0));
    img.set_mipmap(MipmapMode::Linear);
    img.set_mag_filter(TextureFilter::Linear);
    img.update_sampler();
    let img = TextureRef::construct(img);
    let inputs = 
    HashMap::from([
        (String::from("image"), img.clone())
    ]);
    let queue = renderer.queue().clone();
    for i in 0..10 {
        let tu = timer.next_frame();
        renderer.update_timer(&tu);
        
        renderer.execute(inputs.clone());
        renderer.wait();
        
        let out = renderer.render_result();
        out.take().save(queue.clone(), format!("data/render_result/{}.jpg", i));
    }
}