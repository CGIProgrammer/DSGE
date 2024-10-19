#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
pub enum GLSLVersion {
    V140 = 0,
    V150,
    V300ES,
    V310ES,
    V320ES,
    V330,
    V400,
    V410,
    V420,
    V430,
    V440,
    V450,
    V460,
}

impl GLSLVersion {
    pub fn stringify(&self) -> String {
        match self {
            Self::V140 => "#version 140".to_owned(),
            Self::V150 => "#version 150".to_owned(),
            Self::V330 => "#version 330".to_owned(),
            Self::V400 => "#version 400".to_owned(),
            Self::V410 => "#version 410".to_owned(),
            Self::V420 => "#version 420".to_owned(),
            Self::V430 => "#version 430".to_owned(),
            Self::V440 => "#version 440".to_owned(),
            Self::V450 => "#version 450".to_owned(),
            Self::V460 => "#version 460".to_owned(),
            Self::V300ES => "#version 300 es".to_owned(),
            Self::V310ES => "#version 310 es".to_owned(),
            Self::V320ES => "#version 320 es".to_owned(),
        }
    }

    pub fn have_explicit_attri_location(&self) -> bool {
        (*self as usize) >= (Self::V300ES as usize)
    }

    pub fn need_precision_qualifier(&self) -> bool {
        match self {
            Self::V300ES | Self::V310ES | Self::V320ES => true,
            _ => false,
        }
    }
}

#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
pub enum ShaderType {
    Vertex,
    TesselationControl,
    TesselationEval,
    Geometry,
    Fragment,
    Compute,
}

impl std::fmt::Display for ShaderType {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        let type_mark = match self {
            Self::Vertex => "vertex",
            Self::TesselationControl => "teselation cotrol",
            Self::TesselationEval => "teselation evaluation",
            Self::Geometry => "geometry",
            Self::Fragment => "fragmet",
            Self::Compute => "compute",
        };
        write!(formatter, "{}", type_mark)
    }
}

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub enum GLSLSampler {
    Sampler1D = 0x001,
    Sampler2D = 0x002,
    Sampler3D = 0x003,
    SamplerCube = 0x004,
}

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub enum GLSLType {
    Sampler1D = 0x001,
    Sampler2D = 0x002,
    Sampler3D = 0x003,
    SamplerCube = 0x004,
    //Sampler1DArray   = 0x011,
    //Sampler2DArray   = 0x012,
    //SamplerCubeArray = 0x014,
    Bool = 0x1110 | (0xF & (AttribNumType::BYTE as isize)),
    Int = 0x1140 | (0xF & (AttribNumType::INT as isize)),
    Uint = 0x1140 | (0xF & (AttribNumType::UNSIGNED_INT as isize)),
    Float = 0x1140 | (0xF & (AttribNumType::FLOAT as isize)),
    Double = 0x1180 | (0xF & (AttribNumType::DOUBLE as isize)),

    BVec2 = 0x1210 | (0xF & (AttribNumType::BYTE as isize)),
    IVec2 = 0x1240 | (0xF & (AttribNumType::INT as isize)),
    UVec2 = 0x1240 | (0xF & (AttribNumType::UNSIGNED_INT as isize)),
    Vec2 = 0x1240 | (0xF & (AttribNumType::FLOAT as isize)),
    DVec2 = 0x1280 | (0xF & (AttribNumType::DOUBLE as isize)),

    BVec3 = 0x1310 | (0xF & (AttribNumType::BYTE as isize)),
    IVec3 = 0x1340 | (0xF & (AttribNumType::INT as isize)),
    UVec3 = 0x1340 | (0xF & (AttribNumType::UNSIGNED_INT as isize)),
    Vec3 = 0x1340 | (0xF & (AttribNumType::FLOAT as isize)),
    DVec3 = 0x1380 | (0xF & (AttribNumType::DOUBLE as isize)),

    BVec4 = 0x1410 | (0xF & (AttribNumType::BYTE as isize)),
    IVec4 = 0x1440 | (0xF & (AttribNumType::INT as isize)),
    UVec4 = 0x1440 | (0xF & (AttribNumType::UNSIGNED_INT as isize)),
    Vec4 = 0x1440 | (0xF & (AttribNumType::FLOAT as isize)),
    DVec4 = 0x1480 | (0xF & (AttribNumType::DOUBLE as isize)),

    BMat2 = 0x2210 | (0xF & (AttribNumType::BYTE as isize)),
    IMat2 = 0x2240 | (0xF & (AttribNumType::INT as isize)),
    UMat2 = 0x2240 | (0xF & (AttribNumType::UNSIGNED_INT as isize)),
    Mat2 = 0x2240 | (0xF & (AttribNumType::FLOAT as isize)),
    DMat2 = 0x2280 | (0xF & (AttribNumType::DOUBLE as isize)),

    BMat3 = 0x3310 | (0xF & (AttribNumType::BYTE as isize)),
    IMat3 = 0x3340 | (0xF & (AttribNumType::INT as isize)),
    UMat3 = 0x3340 | (0xF & (AttribNumType::UNSIGNED_INT as isize)),
    Mat3 = 0x3340 | (0xF & (AttribNumType::FLOAT as isize)),
    DMat3 = 0x3380 | (0xF & (AttribNumType::DOUBLE as isize)),

    BMat4 = 0x4410 | (0xF & (AttribNumType::BYTE as isize)),
    IMat4 = 0x4440 | (0xF & (AttribNumType::INT as isize)),
    UMat4 = 0x4440 | (0xF & (AttribNumType::UNSIGNED_INT as isize)),
    Mat4 = 0x4440 | (0xF & (AttribNumType::FLOAT as isize)),
    DMat4 = 0x4480 | (0xF & (AttribNumType::DOUBLE as isize)),
}

#[allow(dead_code)]
impl GLSLType {
    fn get_gl_const(self) -> u32 {
        return (self as u32) & 0xF | 0x1400;
    }

    pub fn get_glsl_name(self) -> String {
        let code = self as usize;
        let rows = ((code >> 12) & 0xF) as isize;
        let columns = ((code >> 8) & 0xF) as isize;
        let gl_type_code = AttribNumType::from_i32(self.get_gl_const() as i32);
        let (type_marker, num_type_name) = match gl_type_code {
            AttribNumType::FLOAT | AttribNumType::HALF_FLOAT | AttribNumType::FIXED => {
                ("", "float")
            }
            AttribNumType::DOUBLE => ("d", "double"),
            AttribNumType::BYTE | AttribNumType::UNSIGNED_BYTE => ("b", "bool"),
            AttribNumType::SHORT | AttribNumType::INT => ("i", "int"),
            AttribNumType::UNSIGNED_SHORT | AttribNumType::UNSIGNED_INT => ("u", "uint"),
        };
        match self {
            Self::Sampler1D => return String::from("sampler1D"),
            Self::Sampler2D => return String::from("sampler2D"),
            Self::Sampler3D => return String::from("sampler3D"),
            Self::SamplerCube => return String::from("samplerCube"),
            _ => (),
        };
        match (rows, columns) {
            (1, 1) => format!("{}", num_type_name),
            (1, 2..=4) => format!("{}vec{}", type_marker, columns),
            (2..=4, 2..=4) => {
                if rows == columns {
                    format!("{}mat{}", type_marker, columns)
                } else {
                    format!("{}mat{}x{}", type_marker, columns, rows)
                }
            }
            _ => String::new(),
        }
    }
}

#[allow(dead_code)]
#[allow(non_camel_case_types)]
pub enum AttribNumType {
    BYTE = 0,
    UNSIGNED_BYTE = 1,
    SHORT = 2,
    UNSIGNED_SHORT = 3,
    INT = 4,
    UNSIGNED_INT = 5,
    HALF_FLOAT = 6,
    FLOAT = 7,
    DOUBLE = 8,
    FIXED = 9,
}

impl AttribNumType {
    pub fn from_i32(num: i32) -> Self {
        match num as u32 % 10 {
            0 => Self::BYTE,
            1 => Self::UNSIGNED_BYTE,
            2 => Self::SHORT,
            3 => Self::UNSIGNED_SHORT,
            4 => Self::INT,
            5 => Self::UNSIGNED_INT,
            6 => Self::HALF_FLOAT,
            7 => Self::FLOAT,
            8 => Self::DOUBLE,
            9 => Self::FIXED,
            _ => panic!("Что за хрень происходит?!"),
        }
    }
}

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub enum AttribType {
    Byte = 0x1110 | (0xF & (AttribNumType::BYTE as isize)),
    UByte = 0x1110 | (0xF & (AttribNumType::UNSIGNED_BYTE as isize)),
    Short = 0x1120 | (0xF & (AttribNumType::SHORT as isize)),
    UShort = 0x1120 | (0xF & (AttribNumType::UNSIGNED_SHORT as isize)),
    Int = 0x1140 | (0xF & (AttribNumType::INT as isize)),
    UInt = 0x1140 | (0xF & (AttribNumType::UNSIGNED_INT as isize)),
    HFloat = 0x1120 | (0xF & (AttribNumType::HALF_FLOAT as isize)),
    Float = 0x1140 | (0xF & (AttribNumType::FLOAT as isize)),
    Double = 0x1180 | (0xF & (AttribNumType::DOUBLE as isize)),
    Fixed = 0x1140 | (0xF & (AttribNumType::FIXED as isize)),

    BVec2 = 0x1210 | AttribNumType::BYTE as isize,
    IVec2 = 0x1240 | AttribNumType::INT as isize,
    UVec2 = 0x1240 | AttribNumType::UNSIGNED_INT as isize,
    FVec2 = 0x1240 | AttribNumType::FLOAT as isize,
    DVec2 = 0x1280 | AttribNumType::DOUBLE as isize,
    BVec3 = 0x1310 | AttribNumType::BYTE as isize,
    IVec3 = 0x1340 | AttribNumType::INT as isize,
    UVec3 = 0x1340 | AttribNumType::UNSIGNED_INT as isize,
    FVec3 = 0x1340 | AttribNumType::FLOAT as isize,
    DVec3 = 0x1380 | AttribNumType::DOUBLE as isize,
    BVec4 = 0x1410 | AttribNumType::BYTE as isize,
    IVec4 = 0x1440 | AttribNumType::INT as isize,
    UVec4 = 0x1440 | AttribNumType::UNSIGNED_INT as isize,
    FVec4 = 0x1440 | AttribNumType::FLOAT as isize,
    DVec4 = 0x1480 | AttribNumType::DOUBLE as isize,
    BMat2 = 0x2210 | AttribNumType::BYTE as isize,
    IMat2 = 0x2240 | AttribNumType::INT as isize,
    UMat2 = 0x2240 | AttribNumType::UNSIGNED_INT as isize,
    FMat2 = 0x2240 | AttribNumType::FLOAT as isize,
    DMat2 = 0x2280 | AttribNumType::DOUBLE as isize,
    BMat3 = 0x3310 | AttribNumType::BYTE as isize,
    IMat3 = 0x3340 | AttribNumType::INT as isize,
    UMat3 = 0x3340 | AttribNumType::UNSIGNED_INT as isize,
    FMat3 = 0x3340 | AttribNumType::FLOAT as isize,
    DMat3 = 0x3380 | AttribNumType::DOUBLE as isize,
    BMat4 = 0x4410 | AttribNumType::BYTE as isize,
    IMat4 = 0x4440 | AttribNumType::INT as isize,
    UMat4 = 0x4440 | AttribNumType::UNSIGNED_INT as isize,
    FMat4 = 0x4440 | AttribNumType::FLOAT as isize,
    DMat4 = 0x4480 | AttribNumType::DOUBLE as isize,
}

impl std::fmt::Display for AttribType {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        write!(formatter, "{}", self.get_glsl_name())
    }
}

#[allow(dead_code)]
impl AttribType {
    pub fn get_gl_const(self) -> AttribNumType {
        return AttribNumType::from_i32(self as i32 & 0xF);
    }

    pub fn get_byte_size(self) -> usize {
        let code = self as usize;
        let s = ((code >> 4) & 0xF) as usize;
        return s * self.get_cells_count();
    }

    pub fn get_cells_count(self) -> usize {
        let code = self as usize;
        let n = ((code >> 12) & 0xF) as usize;
        let m = ((code >> 8) & 0xF) as usize;
        m * n
    }

    pub fn is_int(self) -> bool {
        let num_type = AttribNumType::from_i32((self as i32) & 0xF);
        match num_type {
            AttribNumType::HALF_FLOAT
            | AttribNumType::FLOAT
            | AttribNumType::DOUBLE
            | AttribNumType::FIXED => false,
            _ => true,
        }
    }

    pub fn is_float(self) -> bool {
        let num_type = AttribNumType::from_i32((self as i32) & 0xF);
        match num_type {
            AttribNumType::HALF_FLOAT | AttribNumType::FLOAT | AttribNumType::DOUBLE => true,
            _ => false,
        }
    }

    pub fn get_glsl_name(self) -> String {
        let code = self as usize;
        let rows = ((code >> 12) & 0xF) as usize;
        let columns = ((code >> 8) & 0xF) as usize;
        let gl_type_code = self.get_gl_const();
        let (type_marker, num_type_name) = match gl_type_code {
            AttribNumType::FLOAT | AttribNumType::HALF_FLOAT | AttribNumType::FIXED => {
                ("", "float")
            }
            AttribNumType::DOUBLE => ("d", "double"),
            AttribNumType::BYTE | AttribNumType::UNSIGNED_BYTE => ("b", "bool"),
            AttribNumType::SHORT | AttribNumType::INT => ("i", "int"),
            AttribNumType::UNSIGNED_SHORT | AttribNumType::UNSIGNED_INT => ("u", "uint"),
        };
        match (rows, columns) {
            (1, 1) => format!("{}", num_type_name),
            (1, 2..=4) => format!("{}vec{}", type_marker, columns),
            (2..=4, 2..=4) => {
                if rows == columns {
                    format!("{}mat{}", type_marker, columns)
                } else {
                    format!("{}mat{}x{}", type_marker, columns, rows)
                }
            }
            _ => String::new(),
        }
    }
}
