use crate::components::Component;
use crate::game_object::GameObject;
use nalgebra::Rotation3;
use crate::types::{Mat4, Transform3};

#[derive(Default)]
pub struct MotionExample
{
    //rotation: nalgebra::Quaternion<f32>
}

impl Component for MotionExample
{
    fn on_start(&mut self, obj: &mut GameObject)
    {
        //self.rotation = nalgebra::Quaternion::from(obj.transform().local);
        //obj.transform().local;
        //Rotation3::<f32>::from_matrix(&obj.transform().local);
        //Transform3::<f32>::from_matrix_unchecked(obj.transform().local).rotation();
        //Rotation3::
    }

    fn on_loop(&mut self, obj: &mut GameObject)
    {
        let transform = obj.transform_mut();
        let rotation = Rotation3::<f32>::from_euler_angles(0.0, 0.0, 0.1);
        /*let rotation4 = Mat4::new(
            rotation[0], rotation[1], rotation[2], 0.0,
            rotation[3], rotation[4], rotation[5], 0.0,
            rotation[6], rotation[7], rotation[8], 0.0,
            0.0, 0.0, 0.0, 1.0
        );
        transform.local = rotation4 * transform.local;*/
        transform.local.rotate(&rotation);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
