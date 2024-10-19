use crate::shader::ShaderStructUniform;
use std::time::SystemTime;
use std140;

#[derive(Clone)]
pub struct Timer {
    sys_time: SystemTime,
    last_time: SystemTime,
    frame: u32,
}

impl Default for Timer {
    fn default() -> Self {
        Self {
            sys_time: SystemTime::now(),
            last_time: SystemTime::now(),
            frame: 0,
        }
    }
}

crate::fast_impl_ssu! {
    #[derive(Default)]
    layout(std140) struct UniformTime as Time
    {
        uptime: f32,
        delta:  f32,
        frame:  u32
    }
}

impl Timer {
    pub fn new() -> Self {
        Self {
            sys_time: SystemTime::now(),
            last_time: SystemTime::now(),
            frame: 0,
        }
    }

    pub fn next_frame(&mut self) -> UniformTime {
        let delta = self.last_time.elapsed().unwrap().as_micros() as f64 / 1000000.0;
        let uptime = self.sys_time.elapsed().unwrap().as_micros() as f64 / 1000000.0;
        let result = UniformTime {
            frame: std140::uint(self.frame),
            delta: std140::float(delta as f32),
            uptime: std140::float(uptime as f32),
        };
        self.frame += 1;
        self.last_time = SystemTime::now();
        result
    }
}
