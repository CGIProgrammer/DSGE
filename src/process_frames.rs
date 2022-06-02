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
    let mut renderer = renderer::Renderer::offscreen(vk_instance, [480, 360]);
    let mut timer = time::Timer::new();
    let queue = renderer.queue().clone();
    let device = queue.device().clone();

    let img = Texture::new_empty(
        "frame",
        TextureDimensions::Dim2d { width: 384, height: 288, array_layers: 1 },
        TexturePixelFormat::R8G8B8A8_SRGB,
        device.clone()
    ).unwrap(); //renderer.queue().clone(), "/media/ivan/b74968cd-84c1-49ba-8a04-e0829fef9c9a/torrent/dmb/frames_dmb_sr/frame_038005.jpg").unwrap();
    let source = "/media/ivan/b74968cd-84c1-49ba-8a04-e0829fef9c9a/Видео/brrrr/frames_sr_x2/";
    let target = "/media/ivan/b74968cd-84c1-49ba-8a04-e0829fef9c9a/Видео/brrrr/frames_fsr/";
    println!("Begin");
    for i in 1..=9000 {
        img.load_data(queue.clone(), format!("{}/{:06}.png", source, i)).unwrap();
        println!("frame {}      \r", i);
        let tu = timer.next_frame();
        renderer.update_timer(tu);
        
        renderer.execute(HashMap::from([
            (String::from("image"), img.clone())
        ]));
        renderer.wait();
        
        let out = renderer.render_result();
        out.save(queue.clone(), format!("{}/{:06}.png", target, i));
    }
    println!("Finished");
}