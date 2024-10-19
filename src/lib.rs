extern crate bytemuck;
extern crate byteorder;
#[cfg(feature = "use_image")]
extern crate image;
extern crate num;
extern crate vulkano;
extern crate winit;
extern crate getset;
extern crate std140;

// Должно быть степенью двойки
const VULKANO_BUFFER_ATOMIC_SIZE: usize = 64;

pub mod teapot;
pub mod time;

pub mod command_buffer;
pub mod components;
pub mod framebuffer;
pub mod game_logic;
pub mod game_object;
pub mod glenums;
pub mod material;
pub mod mesh;
pub mod references;
pub mod renderer;
pub mod resource_manager;
pub mod scene;
pub mod shader;
pub mod texture;
pub mod types;
pub mod utils;
