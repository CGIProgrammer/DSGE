use crate::types::*;

/// Trait для единообразной передачи uniform-структур и текстур в GLSL шейдер.
/// Он же используется для сборки GLSL шейдера
pub trait ShaderStructUniform {
    /// Должна возвращать текстовое представление структуры типа для GLSL
    fn structure() -> String {
        String::new()
    }

    /// Должна возвращать название типа
    fn glsl_type_name() -> String;

    /// Позволяет получить текстуру, если структура является таковой
    fn texture(&self) -> Option<&crate::texture::Texture> {
        None
    }
}

#[macro_export]
macro_rules! fast_impl_ssu {

    {$($rust_type: ty => $glsl_type: literal),*} => {
        #[doc = "ShaderStructUniform для $rust_type с простой трансляцией типа"]
        $(impl ShaderStructUniform for $rust_type {
            fn glsl_type_name() -> String {
                $glsl_type.to_owned()
            }
        })*
    };
    {
        $(#$idid: tt)*
        struct $struct_name: ident $(as $alias: tt)? {
            $(
                #[format($field_foromat: tt)]
                $field_name: ident: $field_type: ty,
            )*
        }
    } => {
        #[repr(C)]
        #[derive(Clone, Copy, Debug)]
        $(#$idid)*
        #[doc = "GLSL структура `"]
        $(#[doc = stringify!($alias)])?
        #[doc = "` с выравнивающим полем (dummy_numbers) и derive макросами."]
        pub struct $struct_name {
            $(
                #[format($field_foromat)]
                pub $field_name: $field_type
            ),*
        }

        impl ShaderStructUniform for $struct_name {
            fn structure() -> String
            {
                ["{",
                $(format!("    {} {};", <$field_type>::glsl_type_name().as_str(), stringify!($field_name)).as_str()),*,
                "}"].join("\n")
            }

            #[allow(unreachable_code)]
            fn glsl_type_name() -> String
            {
                $({
                    return stringify!($alias).to_owned();
                })?
                stringify!($struct_name).to_owned()
            }
        }
    };
    // Новый шаблон
    {
        $(#$idid: tt)*
        struct $struct_name: ident $(as $alias: tt)? {
            $($field_name: ident: $field_type: ty),*
        }
    } => {
        #[repr(C)]
        #[derive(Clone, Copy, Debug)]
        $(#$idid)*
        #[doc = "GLSL структура `"]
        $(#[doc = stringify!($alias)])?
        #[doc = "` с выравнивающим полем (dummy_numbers) и derive макросами."]
        pub struct $struct_name {
            $(pub $field_name: $field_type),*
        }

        impl ShaderStructUniform for $struct_name {
            fn structure() -> String
            {
                ["{",
                $(format!("    {} {};", <$field_type>::glsl_type_name().as_str(), stringify!($field_name)).as_str()),*,
                "}"].join("\n")
            }

            #[allow(unreachable_code)]
            fn glsl_type_name() -> String
            {
                $({
                    return stringify!($alias).to_owned();
                })?
                stringify!($struct_name).to_owned()
            }
        }
    };

    {
        $(#$idid: tt)*
        layout(std140) struct $struct_name: ident $(as $alias: ident)? 
        {
            $($field_name: ident: $field_type: tt),*
        }
    } => {
        #[derive(Clone, Copy, Debug)]
        $(#$idid)*
        #[doc = "GLSL структура `"]
        $(#[doc = stringify!($alias)])?
        #[doc = "` с выравнивающим полем (dummy_numbers) и derive макросами."]
        #[std140::repr_std140]
        pub struct $struct_name {
            $($field_name: crate::std_to_std140!($field_type)),*
        }

        impl $struct_name {
            pub fn default_new($($field_name: $field_type),*) -> Self
            {
                Self {
                    $($field_name: $field_name.into()),*
                }
            }

            #[inline(always)]
            $(pub fn $field_name(&self) -> $field_type {
                self.$field_name.into()
            })*
        }

        impl ShaderStructUniform for $struct_name {
            fn structure() -> String
            {
                
                ["{",
                $(format!("    {} {};", <crate::std_to_std140!($field_type)>::glsl_type_name().as_str(), stringify!($field_name)).as_str()),*,
                "}"].join("\n")
            }

            #[allow(unreachable_code)]
            fn glsl_type_name() -> String
            {
                #[allow(dead_code)]
                $({
                    return stringify!($alias).to_owned();
                })?
                stringify!($struct_name).to_owned()
            }
        }
    };
}

fast_impl_ssu! {
    f32  => "float",
    i32  => "int",
    u32  => "uint",
    Vec2 => "vec2",
    Vec3 => "vec3",
    Vec4 => "vec4",
    IVec2 => "ivec2",
    IVec3 => "ivec3",
    IVec4 => "ivec4",
    UVec2 => "uvec2",
    UVec3 => "uvec3",
    UVec4 => "uvec4",
    Mat2 => "mat2",
    Mat3 => "mat3",
    Mat4 => "mat4",
    [f32; 2] => "vec2",
    [f32; 3] => "vec3",
    [f32; 4] => "vec4",
    [i32; 2] => "ivec2",
    [i32; 3] => "ivec3",
    [i32; 4] => "ivec4",
    [u32; 2] => "uvec2",
    [u32; 3] => "uvec3",
    [u32; 4] => "uvec4",
    [[f32; 2]; 2] => "mat2",
    [[f32; 3]; 3] => "mat3",
    [[f32; 4]; 4] => "mat4",
    [f32; 16] => "mat4",
    std140::float => "float",
    std140::int => "int",
    std140::uint => "uint",
    std140::vec2 => "vec2",
    std140::vec3 => "vec3",
    std140::vec4 => "vec4",
    std140::ivec2 => "ivec2",
    std140::ivec3 => "ivec3",
    std140::ivec4 => "ivec4",
    std140::uvec2 => "uvec2",
    std140::uvec3 => "uvec3",
    std140::uvec4 => "uvec4",
    std140::mat2x2 => "mat2",
    std140::mat3x3 => "mat3",
    std140::mat4x4 => "mat4"
}

#[test]
fn test_fast_impl_ssu() {
    fast_impl_ssu! {
        struct InlineTestingStruct as Alias {
            one: std140::float,
            sdfhg: std140::vec2
        }
    }

    let _ty = InlineTestingStruct {
        one: std140::float(1.0),
        sdfhg: std140::vec2(1.0, 2.0)
    };
    println!("GLSL структура {} {}", InlineTestingStruct::glsl_type_name(), InlineTestingStruct::structure());
}

#[macro_export]
macro_rules! std_to_std140 {
    (i32          ) => { std140::int   };
    (IVec2        ) => { std140::ivec2 };
    (IVec3        ) => { std140::ivec3 };
    (IVec4        ) => { std140::ivec4 };
    (u32          ) => { std140::uint  };
    (UVec2        ) => { std140::uvec2 };
    (UVec3        ) => { std140::uvec3 };
    (UVec4        ) => { std140::uvec4 };
    (f32          ) => { std140::float };
    (Vec2         ) => { std140::vec2  };
    ([f32; 2])      => { std140::vec2  };
    (Vec3)          => { std140::vec3  };
    ([f32; 3])      => { std140::vec3  };
    (Vec4)          => { std140::vec4  };
    ([f32; 4])      => { std140::vec4  };
    (Mat2)          => { std140::mat2  };
    ([[f32; 2]; 2]) => { std140::mat2  };
    (Mat3)          => { std140::mat3  };
    ([f32; 9])      => { std140::mat3  };
    ([[f32; 3]; 3]) => { std140::mat3  };
    (Mat4)          => { std140::mat4  };
    ([f32; 16])     => { std140::mat4  };
    ([[f32; 4]; 4]) => { std140::mat4  };
    (f64)           => { std140::double };
    (DVec2)         => { std140::dvec2  };
    (DVec3)         => { std140::dvec3  };
    (DVec4)         => { std140::dvec4  };
    (DMat2)         => { std140::dmat2  };
    ([[f64; 2]; 2]) => { std140::dmat2  };
    (Mat3)          => { std140::dmat3  };
    ([f64; 9])      => { std140::dmat3  };
    ([[f64; 3]; 3]) => { std140::dmat3  };
    (Mat4)          => { std140::dmat4  };
    ([f64; 16])     => { std140::dmat4  };
    ([[f64; 4]; 4]) => { std140::dmat4  };

    ( int   ) => {std140::int  }; //{ i32   };
    ( ivec2 ) => {std140::ivec2}; //{ IVec2 };
    ( ivec3 ) => {std140::ivec3}; //{ IVec3 };
    ( ivec4 ) => {std140::ivec4}; //{ IVec4 };
    ( uint  ) => {std140::uint }; //{ u32   };
    ( uvec2 ) => {std140::uvec2}; //{ UVec2 };
    ( uvec3 ) => {std140::uvec3}; //{ UVec3 };
    ( uvec4 ) => {std140::uvec4}; //{ UVec4 };
    ( float ) => {std140::float}; //{ f32   };
    ( vec2  ) => {std140::vec2 }; //{ Vec2  };
    ( vec3  ) => {std140::vec3 }; //{ Vec3  };
    ( vec4  ) => {std140::vec4 }; //{ Vec4  };
    ( mat2  ) => {std140::mat2 }; //{ Mat2  };
    ( mat3  ) => {std140::mat3 }; //{ Mat3  };
    ( mat4  ) => {std140::mat4 }; //{ Mat4  };
    ( float ) => {std140::float}; //{ f64   };
    ( dvec2 ) => {std140::dvec2}; //{ DVec2 };
    ( dvec3 ) => {std140::dvec3}; //{ DVec3 };
    ( dvec4 ) => {std140::dvec4}; //{ DVec4 };
    ( dmat2 ) => {std140::dmat2}; //{ DMat2 };
    ( dmat3 ) => {std140::dmat3}; //{ DMat3 };
    ( dmat4 ) => {std140::dmat4}; //{ DMat4 };
    ( $token: tt ) => {$token};
}

#[test]
fn test_into_for_mat4()
{
    let mat = nalgebra::Matrix4::<f32>::new(
        1.0, 2.0, 3.0, 4.0,
        1.0, 2.0, 3.0, 4.0,
        1.0, 2.0, 3.0, 4.0,
        1.0, 2.0, 3.0, 4.0,
    );

    let _std140_mat: std140::mat4x4 = mat.into();
    let nalgemra_mat: nalgebra::Matrix4<f32> = mat.into();
    dbg!(mat);
    dbg!(nalgemra_mat);
    assert_eq!(mat, nalgemra_mat)
}

#[test]
#[allow(dead_code)]
pub fn test_std_to_std140_macro_nested_struct()
{
    fast_impl_ssu!{
    layout(std140) struct GenericLightUniform {
        color: Vec3,
        power: f32,
        location: Vec3,
        distance: f32,
        z_near: f32,
        shadowmap_index: i32
    }}
    
    fast_impl_ssu!{
        layout(std140) struct NewSpotlightUniform {
            base: GenericLightUniform,
            direction: Vec3
        }
    }
    println!("Базовая структура {} {}", GenericLightUniform::glsl_type_name(), GenericLightUniform::structure());
    println!("Вложенная структура {} {}", NewSpotlightUniform::glsl_type_name(), NewSpotlightUniform::structure());
}