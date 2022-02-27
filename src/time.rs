use crate::shader::ShaderStructUniform;
use crate::texture::TextureRef;
use std::time::{SystemTime, Duration};

pub struct Timer
{
    sys_time: SystemTime,
    last_time: SystemTime,
    frame: u32
}

#[derive(Default, Clone)]
pub struct UniformTime
{
    pub uptime: f32,
    pub delta: f32,
    pub frame: u32
}

impl ShaderStructUniform for UniformTime
{
    fn structure() -> String
    {
        "{
            float uptime;
            float delta;
            uint frame;
        }".to_string()
    }

    fn glsl_type_name() -> String
    {
        String::from("Time")
    }

    fn texture(&self) -> Option<&TextureRef>
    {
        None
    }
}

impl Timer
{
    pub fn new() -> Self
    {
        Self {
            sys_time: SystemTime::now(),
            last_time: SystemTime::now(),
            frame: 0
        }
    }

    pub fn next_frame(&mut self) -> UniformTime
    {
        let result = UniformTime {
            frame: self.frame,
            delta: (self.last_time.elapsed().unwrap().as_micros() as f64 / 1000000.0) as _,
            uptime: (self.sys_time.elapsed().unwrap().as_micros() as f64 / 1000000.0) as _,
        };
        self.frame += 1;
        self.last_time = SystemTime::now();
        result
    }
}