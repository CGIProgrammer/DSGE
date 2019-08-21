# -*- coding: utf-8 -*-
from ctypes import *
import platform
import traceback
import sys
import os
from copy import copy

KEY_SPACE = 32
KEY_APOSTROPHE = 39  # '
KEY_COMMA = 44  # ,
KEY_MINUS = 45  # -
KEY_PERIOD = 46  # .
KEY_SLASH = 47  # /
KEY_0 = 48
KEY_1 = 49
KEY_2 = 50
KEY_3 = 51
KEY_4 = 52
KEY_5 = 53
KEY_6 = 54
KEY_7 = 55
KEY_8 = 56
KEY_9 = 57
KEY_SEMICOLON = 59  # ;
KEY_EQUAL = 61  # =
KEY_A = 65
KEY_B = 66
KEY_C = 67
KEY_D = 68
KEY_E = 69
KEY_F = 70
KEY_G = 71
KEY_H = 72
KEY_I = 73
KEY_J = 74
KEY_K = 75
KEY_L = 76
KEY_M = 77
KEY_N = 78
KEY_O = 79
KEY_P = 80
KEY_Q = 81
KEY_R = 82
KEY_S = 83
KEY_T = 84
KEY_U = 85
KEY_V = 86
KEY_W = 87
KEY_X = 88
KEY_Y = 89
KEY_Z = 90
KEY_LEFT_BRACKET = 91  # [
KEY_BACKSLASH = 92  # \
KEY_RIGHT_BRACKET = 93  # ]
KEY_GRAVE_ACCENT = 96  # `
KEY_WORLD_1 = 161  # non-US #1
KEY_WORLD_2 = 162  # non-US #2

# Function keys
KEY_ESCAPE = 256
KEY_ENTER = 257
KEY_TAB = 258
KEY_BACKSPACE = 259
KEY_INSERT = 260
KEY_DELETE = 261
KEY_RIGHT = 262
KEY_LEFT = 263
KEY_DOWN = 264
KEY_UP = 265
KEY_PAGE_UP = 266
KEY_PAGE_DOWN = 267
KEY_HOME = 268
KEY_END = 269
KEY_CAPS_LOCK = 280
KEY_SCROLL_LOCK = 281
KEY_NUM_LOCK = 282
KEY_PRINT_SCREEN = 283
KEY_PAUSE = 284
KEY_F1 = 290
KEY_F2 = 291
KEY_F3 = 292
KEY_F4 = 293
KEY_F5 = 294
KEY_F6 = 295
KEY_F7 = 296
KEY_F8 = 297
KEY_F9 = 298
KEY_F10 = 299
KEY_F11 = 300
KEY_F12 = 301
KEY_F13 = 302
KEY_F14 = 303
KEY_F15 = 304
KEY_F16 = 305
KEY_F17 = 306
KEY_F18 = 307
KEY_F19 = 308
KEY_F20 = 309
KEY_F21 = 310
KEY_F22 = 311
KEY_F23 = 312
KEY_F24 = 313
KEY_F25 = 314
KEY_KP_0 = 320
KEY_KP_1 = 321
KEY_KP_2 = 322
KEY_KP_3 = 323
KEY_KP_4 = 324
KEY_KP_5 = 325
KEY_KP_6 = 326
KEY_KP_7 = 327
KEY_KP_8 = 328
KEY_KP_9 = 329
KEY_KP_DECIMAL = 330
KEY_KP_DIVIDE = 331
KEY_KP_MULTIPLY = 332
KEY_KP_SUBTRACT = 333
KEY_KP_ADD = 334
KEY_KP_ENTER = 335
KEY_KP_EQUAL = 336
KEY_LEFT_SHIFT = 340
KEY_LEFT_CONTROL = 341
KEY_LEFT_ALT = 342
KEY_LEFT_SUPER = 343
KEY_RIGHT_SHIFT = 344
KEY_RIGHT_CONTROL = 345
KEY_RIGHT_ALT = 346
KEY_RIGHT_SUPER = 347
KEY_MENU = 348

if platform.system() == 'Linux':
    try:
        _siberian = cdll.LoadLibrary('./siberian.so')
    except:
        site_packs = [i for i in sys.path if i.endswith('site-packages')][0]
        _siberian = cdll.LoadLibrary(os.path.join(
            *(site_packs, 'siberian_ctypes', 'siberian.so')))
elif platform.system() == 'Windows':
    try:
        _siberian = cdll.LoadLibrary('./siberian.dll')
    except:
        site_packs = [i for i in sys.path if i.endswith('site-packages')][0]
        _siberian = cdll.LoadLibrary(os.path.join(
            *(site_packs, 'siberian_ctypes', 'siberian.dll')))
    # if platform.processor().lower().find('64')>-1:
        # try:
        #_siberian = CDLL('bin64/siberian.dll')
        # except:
        #_siberian = CDLL('bin32/siberian.dll')
    # elif platform.processor() == 'x86':
        #_siberian = CDLL('./bin32/siberian.dll')
    # else:
        #print("Unsopported processor",platform.processor())


sGetAllocatedMem = _siberian.sGetAllocatedMem
sGetAllocatedMem.argtypes = tuple()
sGetAllocatedMem.reatype = c_uint64

int8_t = c_byte
uint8_t = c_ubyte
int16_t = c_int16
uint16_t = c_uint16
int32_t = c_int32
uint32_t = c_uint32
int64_t = c_int64
uint64_t = c_uint64
char = c_char
double = c_double
void = c_void_p
index_t = uint32_t
dReal = double
#py_object_p = POINTER(py_object)
sRayDir = uint32_t
dJointID = dJointGroupID = dSpaceID = dBodyID = dWorldID = dGeomID = c_void_p
GLuint = uint32_t
GLint = GLsizei = c_int
sColour = c_float*4


def str2c_char_p(text):
    return c_char_p(text.encode())


if platform.system() == 'Linux':
    behaviour = CFUNCTYPE(None, c_void_p)
elif platform.system() == 'Windows':
    behaviour = CFUNCTYPE(None, c_void_p)

ACTION_STOP, ACTION_PLAY, ACTION_LOOP, ACTION_STOPED = list(range(0x00, 0x04))
ACTION_ADD_PLAY, ACTION_ADD_LOOP, ACTION_ADD_STOPED = list(range(0x11, 0x14))

rayX, rayY, rayZ, rayXn, rayYn, rayZn = [0, 1, 2, 5, 6, 7]
X_AXIS = 1
Y_AXIS = 2
Z_AXIS = 4


def clamp(x, a, b):
    return min(max(x, a), b)


def remap(x, a1, b1, a2, b2):
    x -= a1
    x /= b1-a1
    x *= b2-a2
    x += a2
    return x


attributes = {}
attributes['x'] = 0
attributes['y'] = 1
attributes['z'] = 2
attributes['w'] = 3
attributes_mat = {}
attributes_mat['ix'] = 0
attributes_mat['jx'] = 1
attributes_mat['kx'] = 2
attributes_mat['x'] = 3
attributes_mat['iy'] = 4
attributes_mat['jy'] = 5
attributes_mat['ky'] = 6
attributes_mat['y'] = 7
attributes_mat['iz'] = 8
attributes_mat['jz'] = 9
attributes_mat['kz'] = 10
attributes_mat['z'] = 11
attributes_mat['iw'] = 12
attributes_mat['jw'] = 13
attributes_mat['kw'] = 14
attributes_mat['w'] = 15


class laType(Structure):
    _fields_ = [("a", c_float*16),
                ("type", c_ubyte)]

    def __getattr__(self, name):
        if name in list(attributes.keys()) and _siberian.laTypeGetType(self) <= 4:
            return _siberian.laTypeGetItem(self, attributes[name])
        elif name in list(attributes_mat.keys()) and _siberian.laTypeGetType(self) <= 16:
            return _siberian.laTypeGetItem(self, attributes_mat[name])
        else:
            raise AttributeError(
                'laType{} has no attribute {}'.format(self.type, name))

    def __setattr__(self, name, val):
        if name in list(attributes.keys()) and _siberian.laTypeGetType(self) <= 4:
            return _siberian.laTypeSetItem(self, attributes[name], val)
        elif name in list(attributes_mat.keys()) and _siberian.laTypeGetType(self) <= 16:
            return _siberian.laTypeSetItem(self, attributes_mat[name], val)
        else:
            raise AttributeError(
                'laType{} has no attribute {}'.format(self.type, name))

    def __getitem__(self, index):
        if index >= 0 and index < 16:
            return _siberian.laTypeGetItem(self, index)
        else:
            raise IndexError("laType index out of range(16)")

    def __setitem__(self, index, val):
        if index >= 0 and index < 16:
            _siberian.laTypeSetItem(self, index, val)
        else:
            raise IndexError("laType index out of range(16)")

    def __add__(self, other):
        if type(other) == type(self):
            return _siberian.Add(self, other)
        else:
            return _siberian.Addf(self, c_float(other))

    def __sub__(self, other):
        if type(other) == type(self):
            return _siberian.Sub(self, other)
        else:
            return _siberian.Subf(self, c_float(other))

    def __mul__(self, other):
        if type(other) == type(self):
            return _siberian.Mul(other, self)
        else:
            return _siberian.Mulf(self, c_float(other))

    def __truediv__(self, other):
        if isinstance(other, (float, c_float)):
            return _siberian.Divf(self, c_float(other))
        elif isinstance(other, laType) and other.type == 16:
            return _siberian.Mul(_siberian.Inverted(other), self)

    def __repr__(self):
        if self.type == 3:
            return "Vector3D < %.4f, %.4f, %.4f >" % (self.a[0], self.a[1], self.a[2])
        elif self.type == 4:
            return "Vector4D < %.4f, %.4f, %.4f, %.4f >" % (self.a[0], self.a[1], self.a[2], self.a[3])
        elif self.type == 16:
            result = "Matrix4X4 < %.4f, %.4f, %.4f, %.4f,\n" % (
                self.a[0], self.a[1], self.a[2], self.a[3])
            result += "            %.4f, %.4f, %.4f, %.4f,\n" % (
                self.a[4], self.a[5], self.a[6], self.a[7])
            result += "            %.4f, %.4f, %.4f, %.4f,\n" % (
                self.a[8], self.a[9], self.a[10], self.a[11])
            result += "            %.4f, %.4f, %.4f, %.4f >" % (
                self.a[12], self.a[13], self.a[14], self.a[15])
            return result
        elif self.type == 9:
            result = "Matrix3X3 < %.4f, %.4f, %.4f,\n" % (
                self.a[0], self.a[1], self.a[2])
            result += "            %.4f, %.4f, %.4f,\n" % (
                self.a[3], self.a[4], self.a[5])
            result += "            %.4f, %.4f, %.4f," % (
                self.a[6], self.a[7], self.a[8])
            return result
        else:
            return 'undefined laType'

    __str__ = __repr__

    __radd__ = __add__
    __rsub__ = __sub__
    # def __rtruediv__(self, other):
    # if self.type==16:
    # return _siberian.Inverted(self) * other
    # elif self.type==4:
    # return Vector(other/self.x, other/self.y, other/self.z)

    def __neg__(self):
        return Vector(0.0, 0.0, 0.0) - self

    def __abs__(self):
        return _siberian.Length(self)

    length = __abs__

    def normalize(self):
        return _siberian.Normalize(pointer(self))

    def cross(self, other):
        return _siberian.Cross(self, other)

    def crossn(self, other):
        return _siberian.Crossn(self, other)

    def dot(self, other):
        return _siberian.Dot(self, other)

    def dotn(self, other):
        return _siberian.Dotn(self, other)

    def trackTo(self, other):
        _siberian.SetCameraDirection(self, other)

    def inverted(self):
        return _siberian.Inverted(self)

    def toEulerAngles(self):
        return _siberian.ToEuler(self)


xaxis, yaxis, zaxis = list(range(3))

_siberian.Vector.restype = laType
_siberian.Vector.argtypes = (c_float, c_float, c_float)


def Vector(x, y, z): return _siberian.Vector(
    c_float(x), c_float(y), c_float(z))


laLookAt = _siberian.LookAt
laLookAt.restype = laType
laLookAt.argtypes = (laType, laType, c_int, c_int)

_siberian.laTypeGetItem.restype = c_float
_siberian.laTypeGetItem.argtypes = (laType, c_int)
_siberian.laTypeSetItem.argtypes = (POINTER(laType), c_int, c_float)

_siberian.ToEuler.argtypes = laType,
_siberian.ToEuler.restype = laType

_siberian.Normalize.restype = None
_siberian.Normalize.argtypes = (POINTER(laType),)

_siberian.Inverted.argtypes = (laType,)
_siberian.Inverted.restype = laType

_siberian.Addf.restype = laType
_siberian.Add.restype = laType
_siberian.Subf.restype = laType
_siberian.Sub.restype = laType
_siberian.Mulf.restype = laType
_siberian.Mul.restype = laType
_siberian.Divf.restype = laType

_siberian.Cross.restype = laType
_siberian.Cross.argtypes = (laType, laType)
_siberian.Crossn.restype = laType
_siberian.Crossn.argtypes = (laType, laType)

_siberian.Dot.restype = c_float
_siberian.Dot.argtypes = (laType, laType)
_siberian.Dotn.restype = c_float
_siberian.Dotn.argtypes = (laType, laType)
_siberian.Length.restype = c_float

laIdentity = cast(_siberian.Identity, POINTER(laType)).contents

_siberian.SetCameraDirection.argtypes = (POINTER(laType), laType)


def empty_f():
    return None


class sSoundSource(Structure):
    _fields_ = \
        [("loudness",  c_float),
         ('playing_mode',   uint8_t),
         ('_alSource', uint32_t),
         ('_alSampleSet', uint32_t),
            ('_destruction_timer', uint32_t)]


class sTexture(Structure):
    _fields_ = \
        [("__hash",  uint32_t),
         ('__name',  char*256),
            ('__width', uint16_t),
            ('__height', uint16_t),
            ('__data',  c_void_p),
            ('__type',  uint32_t),
            ('__ID',    uint32_t)]

    @staticmethod
    def loadCubemap(name):
        tex = sTexture()
        result = _siberian.sTextureLoadCubemap(tex, name.encode())
        if result == 1:
            raise FileNotFoundError("File {} not found".format(name))
        elif result == 2:
            raise TypeError("File {} is not a DDS file".format(name))
        return tex.contents

    @staticmethod
    def loadImage(name):
        tex = sTexture()
        result = _siberian.sTextureLoadDDS(tex, name.encode())
        if result == 1:
            raise FileNotFoundError("File {} not found".format(name))
        elif result == 2:
            raise TypeError("File {} is not a DDS file".format(name))
        return tex.contents

    @property
    def name(self):
        return self._name.decode()

    @property
    def size(self):
        return getattr(self, "__width"), getattr(self, "__height")

sTexture_p = POINTER(sTexture)

_siberian.sTextureLoadCubemap.argtypes = (sTexture_p, c_char_p)
_siberian.sTextureLoadCubemap.restype = c_int

_siberian.sTextureLoadDDS.argtypes = (sTexture_p, c_char_p)
_siberian.sTextureLoadDDS.restype = c_int

class sShader(Structure):
    _fields_ = \
        [('name',  char*256),
         ('_fragment_source', c_char_p),
            ('_vertex_source', c_char_p),
         ('_log_len', GLsizei),
            ('_frag_source_len', GLsizei),
            ('_vert_source_len', GLsizei),
            ('_success', GLint),
            ('_compute', GLint),
            ('_fragment', GLint),
            ('_vertex', GLint),
            ('_program', GLint),
            ('_log', GLint),

            ('_fp', c_void_p)]


class sMaterial(Structure):
    _fields_ = \
        [("_hash",  uint32_t),
         ('_name',  char*256),
            ('friction', double),
            ('transparency', c_float),
            ('glass', c_bool),
            ('height_scale', c_float),
            ('diffuse', c_float*4),
            ('specular', c_float*4),
            ('_diffuse_texture', POINTER(sTexture)),
            ('_specular_texture', POINTER(sTexture)),
            ('_height_texture', POINTER(sTexture)),
            ('_lightmap_texture', POINTER(sTexture)),
            ('_reflection_cubemap', POINTER(sTexture)),
            ('tdx', c_float),
            ('tdy', c_float),
            ('glow', c_float),
            ('wet', c_float),
            ('_shader', uint32_t)]

    @property
    def name(self):
        return self._name.decode()


class sMesh(Structure):
    _fields_ = \
        [("_hash",  uint32_t),
         ('name',  char*256),
            ('_link_matrix', POINTER(laType)),
            ('_vertices', POINTER(c_float*16)),
            ('_indices', POINTER(index_t)),
            ('_ind_count', uint32_t),
            ('_vert_count', uint32_t),
            ('_material', POINTER(sMaterial)),
            ('_VBO', uint32_t),
            ('_IBO', uint32_t),
            ('_uniforms', uint32_t*4),
            ('_transform', laType),
            ('_bounding_box', laType),
            ('_deformed', c_bool),
            ('_bones_indices', uint16_t*128),
            ('_uv2', c_bool),
            ('_owner', void)]

    @property
    def name(self):
        return self._name.decode()

    @property
    def material(self):
        if self._material:
            return self._material.contents
        elif not self._material:
            return None
        else:
            return None

    def setMaterial(self, scene, name):
        if isinstance(name, str):
            _siberian.sMeshSetMaterial(self, scene, name.encode())
        elif isinstance(name, bytes):
            _siberian.sMeshSetMaterial(self, scene, name)

    def getOwner(self):
        return _cast_to_sobject_p(self._owner).contents


class sPhysicsContact(Structure):
    _fields_ = \
        [('_object', c_void_p),
         ('_position', dReal*3),
            ('_normal', dReal*3)]

    @property
    def hitObject(self):
        return _cast_to_sobject_p(self._object).contents

    @property
    def hitPosition(self):
        return Vector(self._position[0], self._position[1], self._position[2])

    @property
    def hitNormal(self):
        return Vector(self._normal[0], self._normal[1], self._normal[2])


class sPhysicsCS(Structure):
    _fields_ = \
        [('__space', dSpaceID),
         ('_contacts', POINTER(sPhysicsContact)),
            ('_contactsCount', index_t),
         ('_contactsAllocated', index_t), ]

    _read_only = ['contacts', ]

    @property
    def contacts(self):
        return [self._contacts[i] for i in range(self._contactsCount)]


class sPhysicsRS(Structure):
    _fields_ = \
        [('__space', dSpaceID),
         ('_contacts', POINTER(sPhysicsContact)),
            ('_contactsCount', index_t),
            ('_contactsAllocated', index_t),
            ('_angle', dReal),
            ('_range', dReal),
            ('_dir', sRayDir),
            ('_radar_mesh', dGeomID)]

    _read_only = ['contacts', ]

    @property
    def angle(self):
        return self._angle

    @angle.setter
    def angle(self, val):
        val = c_float(val)
        _siberian.sPhysicsRadarSetAngle(self, val)

    @property
    def range(self):
        return self._range

    @range.setter
    def range(self, Range):
        _siberian.sPhysicsRSSetRange(self, Range)

    @property
    def contacts(self):
        return self._contacts[:self._contactsCount]
        # return [self._contacts[i] for i in range(self._contactsCount)]

    @property
    def contactsCount(self):
        return self._contactsCount

    @property
    def dir(self):
        return self._dir

    @dir.setter
    def dir(self, value):
        if value in range(8):
            self._dir = value
        else:
            raise ValueError('Invalid direction value')


class sObjectBase(Structure):
    _fields_ = \
        [("__hash",  uint32_t),
         ('_name',  char*256),
            ('_child_count', index_t),
            ('_parent', c_void_p),
            ('_children', POINTER(c_void_p)),
            ('_scene', c_void_p),
            ('_behaviour', behaviour),
            #('_pyBehaviour', py_object),
            #('_pyobj', py_object_p),
         ('_transform', laType),
         ('transform_global', laType),
         ('transform_global_previous', laType),
         ('_transformable', c_bool),
         ('inactive', c_bool),
         ('_hidden', c_bool),
         ('__data', c_void_p)]


class sObject(Structure):
    _fields_ = sObjectBase._fields_[:]
    _fields_.extend([
        ('_skeleton', POINTER(c_void_p)),
        ('_mesh', POINTER(sMesh)),
        ('__body', dBodyID),
        ('__bodyIgnoring', dBodyID),
        ('_geom', dGeomID),
        ('_radar', sPhysicsRS),
        ('_ray', sPhysicsRS),
        ('_collider', sPhysicsCS),
        ('__ghost', c_int),
        ('__enabled', c_int),
        ('_physicsType', uint32_t),
        ('_physicsShape', uint32_t),
        ('_collisionGroups', uint32_t),
        ('_collideWithGroups', uint32_t),
        ('_physicsFriction', dReal),
        ('_physicsMass', dReal),
        ('_averangeVel', dReal)])

    _sensors = ('radar', 'ray', 'collider')
    _types = {'o': 'sObject', 'c': 'sCamera',
              'l': 'sLight', 'b': 'sBone', 's': 'sSkeleton'}
    _types[ord('o')] = 'sObject'
    _types[ord('c')] = 'sCamera'
    _types[ord('l')] = 'sLight'
    _types[ord('b')] = 'sBone'
    _types[ord('s')] = 'sSkeleton'
    _read_only = ['radar', 'ray', 'collider', 'mass']
    __properties = {}

    def __getitem__(self, key):
        return self.attributes[key]

    def __setitem__(self, key, value):
        self.attributes[key] = value

    def placeChildren(self):
        _siberian.sObjectPlaceChildren(self)

    @property
    def skeleton(self):
        if self._skeleton:
            return cast(self._skeleton, POINTER(sObject)).contents
        else:
            None

    def setIgnoredObject(self, obj):
        setattr(self, '__bodyIgnoring', getattr(obj, '__body'))
    # @skeleton.setter
    # def skeleton(self,obj):
        # if isinstance(obj,sObject) and (obj.name[0]=='s' or obj.name[0]==ord('s')):
        #obj = cast(pointer(obj),c_void_p)
        # else:
        #raise AttributeError('Invalid type for skeleton attribute')

    def actionProcess(self):
        if self._name[0] == ord('s') or self._name[0] == b's':
            _siberian.sActionProcess(self)
        else:
            raise AttributeError('{} has no attribute actionProcess'.format(
                sObject._types[self._name[0]]))

    @property
    def animatedFlag(self):
        return sBoneGetAnimatedFlag(self)

    @animatedFlag.setter
    def animatedFlag(self, flag):
        return sBoneSetAnimatedFlag(self, flag)

    @property
    def parent(self):
        if self._parent:
            return _cast_to_sobject_p(self._parent).contents
        else:
            None

    @property
    def attributes(self):
        props = self.scene.pydata
        if not addressof(self) in list(props.keys()):
            props[addressof(self)] = {}
        return props[addressof(self)]

    @attributes.setter
    def attributes(self, val):
        props = self.scene.pydata
        if type(val) == dict or hasattr(val, '__getitem__'):
            props[addressof(self)] = copy(val)
        else:
            raise AttributeError('Invalid type for attributes array')

    @property
    def mesh(self):
        if (self._name[0] == ord('o') or self._name[0] == b'o') and self._mesh:
            return self._mesh.contents
        elif not self._mesh:
            return None
        else:
            raise AttributeError('{} has no attribute mesh'.format(
                sObject._types[self._name[0]]))

    @mesh.setter
    def mesh(self, value):
        if isinstance(value, sMesh):
            self._mesh = pointer(value)
        elif isinstance(value, POINTER(sMesh)):
            self._mesh = value
        elif isinstance(value, str):
            sObjectSetMeshByName(self, value)
        else:
            raise TypeError(
                'Wrong type (%s). Expexted sMesh or string (name of mesh)' % (str(type(value))))

    @property
    def radar(self):
        if self._name[0] == ord('o') or self._name[0] == b'o':
            return self._radar
        else:
            raise AttributeError(
                '{} has no physics/sensor controllers'.format(sObject._types[self._name[0]]))

    @property
    def ray(self):
        if self._name[0] == ord('o') or self._name[0] == b'o':
            return self._ray
        else:
            raise AttributeError(
                '{} has no physics/sensor controllers'.format(sObject._types[self._name[0]]))

    @property
    def collider(self):
        if self._name[0] == ord('o') or self._name[0] == b'o':
            return self._collider
        else:
            raise AttributeError(
                '{} has no physics/sensor controllers'.format(sObject._types[self._name[0]]))

    @property
    def name(self):
        return self._name.decode()

    @property
    def mass(self):
        if self._name[0] == ord('o') or self._name[0] == b'o':
            return self._mass
        else:
            raise AttributeError(
                '{} has no physics/sensor controllers'.format(sObject._types[self._name[0]]))

    @property
    def suspendDynamics(self):
        if self._name[0] == ord('o') or self._name[0] == b'o':
            return lambda: _siberian.sPhysicsSuspend(self)
        else:
            raise AttributeError(
                '{} has no physics/sensor controllers'.format(sObject._types[self._name[0]]))

    @property
    def restoreDynamics(self):
        if self._name[0] == ord('o') or self._name[0] == b'o':
            return lambda: _siberian.sPhysicsResume(self)
        else:
            raise AttributeError(
                '{} has no physics/sensor controllers'.format(sObject._types[self._name[0]]))

    @property
    def autoDisablePhysics(self):
        return None

    @autoDisablePhysics.setter
    def autoDisablePhysics(self, value):
        _siberian.sPhysicsAutoDisable(self, value)

    def show(self):
        self._hidden = False

    def hide(self):
        self._hidden = True

    def getScene(self):
        return _siberian.sObjectGetScene(pointer(self)).contents

    def endObject(self):
        props = self.scene.pydata
        funcs = self.scene.pyfunctions
        if addressof(self) in props:
            del props[addressof(self)]
        if addressof(self) in funcs:
            del funcs[addressof(self)]
        _siberian.sObjectDelDuplicate(pointer(self))

    @property
    def uniqueNumber(self):
        return getattr(self, '__hash')

    @property
    def globalPosition(self):
        return sObjectGetPositionGlobal3fv(self)

    @globalPosition.setter
    def globalPosition(self, value):
        sObjectSetPositionGlobal3fv(self, value)

    @property
    def transform(self):
        return self._transform

    @transform.setter
    def transform(self, value):
        _siberian.sObjectSetLocalTransform(self, value)

    def rotateLocal(self, x, y, z):
        _siberian.sObjectRotateLocal3f(pointer(self), x, y, z)

    def rotateGlobal(self, x, y, z):
        _siberian.sObjectRotateGlobal3f(pointer(self), x, y, z)

    def translateGlobal(self, vector):
        _siberian.sObjectMoveGlobal3fv(pointer(self), vector)

    def translateLocal(self, vector):
        _siberian.sObjectMoveLocal3fv(pointer(self), vector)

    def snapTo(self, other):
        for i in range(16):
            self._transform[i] = other.transform_global[i]

    def setTransformToPhysics(self):
        _siberian.sObjectSetTransformToPhysics(pointer(self))

    def resetLocalPostion(self):
        for i in range(4):
            for j in range(4):
                self._transform[(i << 2) | j] = float(i == j)

    def setRotation(self, x, y, z):
        x, y, z = c_float(x), c_float(y), c_float(z)
        _siberian.sObjectSetRotation3f(pointer(self), x, y, z)

    def applyForceAtPoint(self, pos, vec):
        _siberian.sPhysicsApplyForceAtPointGlobal3fv(self, pos, vec)

    def applyImpulse(self, pos, vec):
        _siberian.sPhysicsApplyImpulseAtPointGlobal3fv(self, pos, vec)

    def applyHit(self, pos, vec, mass):
        _siberian.sPhysicsApplyHitAtPointGlobal3fv(self, pos, vec, mass)

    def getDistanceTo(self, other):
        try:
            return _siberian.sObjectGetDistanceTo(pointer(self), pointer(other))
        except BaseException:
            print(self, other)
            exit(1)

    def getVectorTo(self, other):
        return _siberian.sObjectGetVectorTo(pointer(self), pointer(other))

    def removeParent(self, at=1):
        if at:
            _siberian.sObjectRemoveParent(pointer(self))
        else:
            _siberian.sObjectDelParent(pointer(self))

    def getChildrenList(self):
        return [_siberian.sObjectGetChildren(pointer(self), i).contents for i in range(_siberian.sObjectGetChildCount(pointer(self)))]

    def getChildrenDict(self):
        result = {}
        for i in range(_siberian.sObjectGetChildCount(pointer(self))):
            child = _siberian.sObjectGetChildren(pointer(self), i).contents
            result[str(child.name)] = child
        return result

    def trackTo(self, target, axis, up):
        if isinstance(target, sObject):
            _siberian.sObjectTrackToOther(
                pointer(self), pointer(target), axis, up)
        elif type(target) == laType and target.type == 3:
            _siberian.sObjectTrackToPoint(pointer(self), target, axis, up)
        elif type(target) == laType and target.type == 16:
            _siberian.sObjectTrackToPoint(pointer(self), Vector(
                target.x, target.y, target.z), axis, up)
        else:
            raise TypeError(
                "trackTo: wrong type. Should be laType(Vector) or laType(Matrix) or sObject")

    def setBehaviour(self, func):
        def sast(ptr):
            func(cast(ptr, POINTER(sObject)).contents)
        beh = behaviour(sast)
        _siberian.sObjectSetBehaviour(self, beh)
        self.scene.pyfunctions[addressof(self)] = beh
        # return
        # if not addressof(self) in list(_functions.keys()):
        #    _functions[addressof(self)] = func

    def removeBehaviour(self):
        _siberian.sObjectSetBehaviour(self, cast(c_void_p(0), behaviour))
        # return
        # if addressof(self) in list(_functions.keys()):
        #    del _functions[addressof(self)]

    def setParent(self, targ, at=1):
        if isinstance(targ, sObject):
            _siberian.sObjectSetParent(pointer(self), pointer(targ), at)
        else:
            raise TypeError(
                'sObject parent should be object? not ' + str(type(targ)))

    def collisionSensorOn(self):
        _siberian.sPhysicsCSInit(pointer(self))

    def raySensorOn(self, radius=1.0):
        _siberian.sPhysicsRSInit(pointer(self), radius)

    def radarSensorOn(self, radius=1.0, angle=3.1415926535/2):
        _siberian.sPhysicsRadSInit(pointer(self), radius, angle)

    def radarHitObjectCount(self):
        return _siberian.sPhysicsRadSGetHitObjectCount(pointer(self))

    def rayHitObjectCount(self):
        return _siberian.sPhysicsRSGetHitObjectCount(pointer(self))

    def collisionHitObjectCount(self):
        return _siberian.sPhysicsCSGetHitObjectCount(pointer(self))

    def radarHitObjectsList(self):
        return [_siberian.sPhysicsRadSGetHitObject(pointer(self), i).contents for i in range(_siberian.sPhysicsRadSGetHitObjectCount(pointer(self)))]

    def radarHitObjectsDict(self):
        di = {}
        for i in range(_siberian.sPhysicsRadSGetHitObjectCount(pointer(self))):
            obj = _siberian.sPhysicsRadSGetHitObject(pointer(self), i).contents
            di[obj.name] = obj
        return di

    def rayHitObjectsList(self):
        return [_siberian.sPhysicsRSGetHitObject(pointer(self), i).contents for i in range(_siberian.sPhysicsRSGetHitObjectCount(pointer(self)))]

    def rayHitObjectsDict(self):
        di = {}
        for i in range(_siberian.sPhysicsRSGetHitObjectCount(pointer(self))):
            obj = _siberian.sPhysicsRSGetHitObject(pointer(self), i).contents
            di[obj.name] = obj
        return di

    def collisionHitObjectsList(self):
        return [_siberian.sPhysicsCSGetHitObject(pointer(self), i).contents for i in range(_siberian.sPhysicsCSGetHitObjectCount(pointer(self)))]

    def collisionHitObjectsDict(self):
        di = {}
        for i in range(_siberian.sPhysicsCSGetHitObjectCount(pointer(self))):
            obj = _siberian.sPhysicsCSGetHitObject(pointer(self), i).contents
            di[obj.name] = obj
        return di

    def setAngularVelocity(self, x, y, z):
        _siberian.sPhysicsSetAngularVelocity(self, x, y, z)

    def getLinearVelocityLocal(self):
        return _siberian.sPhysicsGetLinearVelocity(self)

    def getLinearVelocityGlobal(self):
        e = laIdentity*1.0
        e.w = 0.0
        orientation = self.transform_global * e
        return _siberian.sPhysicsGetLinearVelocity(self) * orientation

    def setLinearVelocityGlobal(self, vector, axes=0b111):
        _siberian.sPhysicsSetSpeedGlobal(self, vector, axes)

    def setSpeedGlobal(self, vector, axes=0b111):
        _siberian.sPhysicsSetSpeedGlobal(self, vector, axes)

    def setSpeedXLocal(self, val):
        _siberian.sPhysicsSetSpeedXLocal(self, val)

    def setSpeedYLocal(self, val):
        _siberian.sPhysicsSetSpeedYLocal(self, val)

    def setSpeedZLocal(self, val):
        _siberian.sPhysicsSetSpeedZLocal(self, val)

    def setLayerWeight(self, layer, weight):
        if self._name[0] == ord('s') or self._name[0] == b's':
            sSkeletonSetLayerWeight(self, layer, weight)
        else:
            raise AttributeError(
                sObject._types[chr(self._name[0])] + ' has no attribute setActionWeight')

    def setActionTime(self, layer, time):
        if self._name[0] == ord('s') or self._name[0] == b's':
            sSkeletonSetLayerTime(self, layer, time)
        else:
            raise AttributeError(sObject._types[chr(
                self._name[0])] + ' has no attribute setActionTime aka setActionPeriod')

    def setActionPeriod(self, layer, time):
        if self._name[0] == ord('s') or self._name[0] == b's':
            sSkeletonSetLayerTime(self, layer, time)
        else:
            raise AttributeError(sObject._types[chr(
                self._name[0])] + ' has no attribute setActionPeriod aka setActionTime')

    def getActionTime(self, layer):
        if self._name[0] == ord('s') or self._name[0] == b's':
            return sSkeletonSetLayerTime(self, layer)
        else:
            raise AttributeError(
                sObject._types[chr(self._name[0])] + ' has no attribute getActionTime')

    def setActionSpeed(self, layer, time):
        if self._name[0] == ord('s') or self._name[0] == b's':
            sSkeletonSetLayerSpeed(self, layer, time)
        else:
            raise AttributeError(
                sObject._types[chr(self._name[0])] + ' has no attribute setActionSpeed')

    def getActionSpeed(self, layer):
        if self._name[0] == ord('s') or self._name[0] == b's':
            return sSkeletonGetLayerSpeed(self, layer)
        else:
            raise AttributeError(
                sObject._types[chr(self._name[0])] + ' has no attribute getActionSpeed')

    def setActionFrame(self, layer, value):
        if self._name[0] == ord('s') or self._name[0] == b's':
            sSkeletonSetActionFrame(self, layer, value)
        else:
            raise AttributeError(
                sObject._types[chr(self._name[0])] + ' has no attribute setActionFrame')

    def getActionFrame(self, layer):
        if self._name[0] == ord('s') or self._name[0] == b's':
            return sSkeletonGetActionFrame(self, layer)
        else:
            raise AttributeError(
                sObject._types[chr(self._name[0])] + ' has no attribute getActionFrame')

    def setActionFrame2(self, layer, value):
        if self._name[0] == ord('s') or self._name[0] == b's':
            sSkeletonSetActionFrame2(self, layer, value)
        else:
            raise AttributeError(
                sObject._types[chr(self._name[0])] + ' has no attribute setActionFrame2')

    def getActionFrame2(self, layer):
        if self._name[0] == ord('s') or self._name[0] == b's':
            return sSkeletonGetActionFrame2(self, layer)
        else:
            raise AttributeError(
                sObject._types[chr(self._name[0])] + ' has no attribute getActionFrame2')

    def resetPose(self):
        if self._name[0] == ord('s') or self._name[0] == b's':
            sSkeletonResetPose(self)
        else:
            print(self._name)
            raise AttributeError(
                sObject._types[chr(self._name[0])] + ' has no attribute resetPose')

    def addPoseFromLayerToPose(self, layer, time, weight=1.0):
        if self._name[0] == ord('s') or self._name[0] == b's':
            sSkeletonAddPoseFromLayerToPose(self, layer, time, weight)
        else:
            raise AttributeError(sObject._types[chr(
                self._name[0])] + ' has no attribute addPoseFromLayerToPose')

    def mixPoseFromLayerWithPose(self, layer, time, weight=1.0):
        if self._name[0] == ord('s') or self._name[0] == b's':
            sSkeletonMixPoseFromLayerWithPose(self, layer, time, weight)
        else:
            raise AttributeError(sObject._types[chr(
                self._name[0])] + ' has no attribute addPoseFromLayerToPose')

    def addPoseFromActionToPose(self, name, keyframe=0, time=0.0, weight=1.0):
        if self._name[0] == ord('s') or self._name[0] == b's':
            name = str2c_char_p(name)
            sSkeletonAddPoseFromActionToPose(
                self, name, keyframe, time, weight)
        else:
            raise AttributeError(sObject._types[chr(
                self._name[0])] + ' has no attribute addPoseFromActionToPose')

    def mixPoseFromActionWithPose(self, name, keyframe=0, time=0.0, weight=1.0):
        if self._name[0] == ord('s') or self._name[0] == b's':
            name = str2c_char_p(name)
            sSkeletonMixPoseFromActionWithPose(
                self, name, keyframe, time, weight)
        else:
            raise AttributeError(sObject._types[chr(
                self._name[0])] + ' has no attribute mixPoseFromActionWithPose')

    @property
    def ghost(self):
        return getattr(self, '__ghost')

    @ghost.setter
    def ghost(self, val):
        setattr(self, '__ghost', val)

    @property
    def scene(self):
        return cast(self._scene, POINTER(sScene)).contents

    @property
    def color(self):
        return cast(pointer(self), POINTER(sLight)).contents.color

    @property
    def bones(self):
        if self._name[0] == ord('s') or self._name[0] == b's':
            res = {}
            for i in range(sSkeletonGetBoneCount(pointer(self))):
                bone = sSkeletonGetBoneByIndex(pointer(self), i).contents
                res[bone.name] = bone
            return res
        else:
            raise AttributeError(
                sObject._types[chr(self._name[0])] + ' has no attribute bones')

    @property
    def setActionInterval(self):
        if self._name[0] == ord('s') or self._name[0] == b's':
            return lambda *args: sSkeletonSetActionInterval(self, *args)
        else:
            raise AttributeError(
                sObject._types[chr(self._name[0])] + ' has no attribute setActionInterval')

    @property
    def setActionParam(self):
        if self._name[0] == ord('s') or self._name[0] == b's':
            return lambda *args: sActionSetParam(self, *args)
        else:
            raise AttributeError(
                sObject._types[chr(self._name[0])] + ' has no attribute setActionParam')

    @property
    def playAction(self):
        if self._name[0] == ord('s') or self._name[0] == b's':
            return lambda *args: sSkeletonSetPlayAction(self, *args)
        else:
            raise AttributeError(
                sObject._types[chr(self._name[0])] + ' has no attribute playAction')

    def playActionInTime(self, name, layer, act_type, start_frame, end_frame, act_time):
        dist = abs(end_frame-start_frame)
        speed = dist / act_time
        framerate = speed * 0.0333333
        self.playAction(name, layer, act_type,
                        start_frame, end_frame, framerate)

    def setActionParamInTime(self, layer, act_type, start_frame, end_frame, act_time):
        dist = abs(end_frame-start_frame)
        speed = float(dist) / act_time
        framerate = speed * 0.0333333
        self.setActionParam(layer, act_type, start_frame, end_frame, framerate)

    @property
    def setAction(self):
        if self._name[0] == ord('s') or self._name[0] == b's':
            return lambda name, layer: sSkeletonSetAction(pointer(self), layer, c_char_p(name.encode()))
        else:
            raise AttributeError(
                sObject._types[self._name[0]] + ' has no attribute setAction')

    @property
    def isPlayingAction(self):
        if self._name[0] == ord('s') or self._name[0] == b's':
            return lambda *args: sActionIsPlaying(self, *args)
        else:
            raise AttributeError(
                sObject._types[self._name[0]] + ' has no attribute isPLayingAction')

    @property
    def stopAction(self):
        if self._name[0] == ord('s') or self._name[0] == b's':
            return lambda layer: sActionStop(pointer(self), layer)
        else:
            raise AttributeError(
                sObject._types[self._name[0]] + ' has no attribute setAction')

    def attachSound(self, name):
        if name is None or name == '':
            return
        sSoundAttachToObject(pointer(self), name)

def _cast_to_sobject_p(obj):
    return cast(obj, POINTER(sObject))

def Object(obj):
    return cast(pointer(obj), POINTER(sObject)).contents

def sSkeletonCast(obj):
    return cast(pointer(obj), POINTER(sSkeleton)).contents

def convert_python_to_c(func):
    def Foo(obj):
        var = _cast_to_sobject_p(obj)
        func(var.contents)
        return None
    return behaviour(Foo)
    # return behaviour(lambda obj : func(_cast_to_sobject_p(obj).contents))


_functions = {}


def executeAll(arg):
    INTP = POINTER(sObject)
    keys = list(_functions.keys())
    i = 0
    while i < len(list(_functions.keys())):
        key = list(_functions.keys())[i]
        try:
            _functions[key](cast(key, INTP).contents)
        except Exception as e:
            exc_info = sys.exc_info()
            print("Python exception:")
            print("".join(traceback.format_exception(*exc_info)))
        i += 1
    return 0


def sEngineClearScripts():
    global _functions
    _functions = {}

# def sSkeletonCast(obj):


sMeshNull = cast(0, POINTER(sMesh))

_siberian.sObjectPlaceChildren.argtypes = (POINTER(sObject),)
_siberian.sObjectPlaceChildren.restype = None

_siberian.sObjectSetBehaviour.argtypes = (POINTER(sObject), behaviour)

sObjectSetMeshPtr = _siberian.sObjectSetMesh
sObjectSetMeshPtr.argtypes = (POINTER(sObject), POINTER(sMesh))

_siberian.sObjectSetMeshByName.argtypes = (POINTER(sObject), c_char_p)

def sObjectSetMeshByName(ptr, name): return _siberian.sObjectSetMeshByName(
    ptr, name.encode() if type(name) == str else name)

sObjectEnd = _siberian.sObjectDelDuplicate
sObjectEnd.argtypes = (POINTER(sObject),)
sObjectEnd.restype = None

sObjectSetPositionGlobal3fv = _siberian.sObjectSetPositionGlobal3fv
sObjectSetPositionGlobal3fv.argtypes = (POINTER(sObject), laType)
sObjectSetPositionGlobal3fv.restype = None

_siberian.sObjectSetLocalTransform.argtypes = (POINTER(sObject), laType)
_siberian.sObjectSetLocalTransform.restype = None

sObjectGetPositionGlobal3fv = _siberian.sObjectGetPositionGlobal3fv
sObjectGetPositionGlobal3fv.argtypes = (POINTER(sObject),)
sObjectGetPositionGlobal3fv.restype = laType

sObjectRotateLocal3f = _siberian.sObjectRotateLocal3f
sObjectRotateLocal3f.argtypes = (POINTER(sObject), c_float, c_float, c_float)

sObjectRotateGlobal3f = _siberian.sObjectRotateGlobal3f
sObjectRotateGlobal3f.argtypes = (POINTER(sObject), c_float, c_float, c_float)

sObjectMoveGlobal3fv = _siberian.sObjectMoveGlobal3fv
sObjectMoveGlobal3fv.argtypes = (POINTER(sObject), laType)

sObjectMoveLocal3fv = _siberian.sObjectMoveLocal3fv
sObjectMoveLocal3fv.argtypes = (POINTER(sObject), laType)

sObjectSetRotation3f = _siberian.sObjectSetRotation3f
sObjectSetRotation3f.argtypes = (POINTER(sObject), c_float, c_float, c_float)
sObjectSetRotation3f.restype = None

sObjectGetParent = _siberian.sObjectGetParent
sObjectGetParent.restype = POINTER(sObject)

sObjectGetDistanceTo = _siberian.sObjectGetDistanceTo
sObjectGetDistanceTo.argtypes = (POINTER(sObject), POINTER(sObject))
sObjectGetDistanceTo.restype = c_float

sObjectGetVectorTo = _siberian.sObjectGetVectorTo
sObjectGetVectorTo.argtypes = (POINTER(sObject), POINTER(sObject))
sObjectGetVectorTo.restype = laType

_siberian.sObjectRemoveParent
_siberian.sObjectRemoveParent.argtypes = (POINTER(sObject),)
_siberian.sObjectRemoveParent.restype = None

sObjectDelParent = _siberian.sObjectDelParent
sObjectDelParent.argtypes = (POINTER(sObject),)
sObjectDelParent.restype = None

sObjectGetChildren = _siberian.sObjectGetChildren
sObjectGetChildren.argtypes = (POINTER(sObject), uint32_t)
sObjectGetChildren.restype = POINTER(sObject)

sObjectTrackToOther = _siberian.sObjectTrackToOther
sObjectTrackToOther.argtypes = (
    POINTER(sObject), POINTER(sObject), uint8_t, uint8_t)

sObjectTrackToPoint = _siberian.sObjectTrackToPoint
sObjectTrackToPoint.argtypes = (POINTER(sObject), laType, uint8_t, uint8_t)

sObjectSetParent = _siberian.sObjectSetParent
sObjectSetParent.argtypes = (POINTER(sObject), POINTER(sObject), c_bool)

sPhysicsApplyForceAtPointGlobal3fv = _siberian.sPhysicsApplyForceAtPointGlobal3fv
sPhysicsApplyForceAtPointGlobal3fv.argtypes = (
    POINTER(sObject), laType, laType)

sPhysicsApplyImpulseAtPointGlobal3fv = _siberian.sPhysicsApplyImpulseAtPointGlobal3fv
sPhysicsApplyImpulseAtPointGlobal3fv.argtypes = (
    POINTER(sObject), laType, laType)

sPhysicsApplyHitAtPointGlobal3fv = _siberian.sPhysicsApplyHitAtPointGlobal3fv
sPhysicsApplyHitAtPointGlobal3fv.argtypes = (
    POINTER(sObject), laType, laType, c_float)

sPhysicsCSInit = _siberian.sPhysicsCSInit
sPhysicsCSInit.argtypes = (POINTER(sObject),)

sPhysicsRSInit = _siberian.sPhysicsRSInit
sPhysicsRSInit.argtypes = (POINTER(sObject), c_float)

sPhysicsRadSInit = _siberian.sPhysicsRadSInit
sPhysicsRadSInit.argtypes = (POINTER(sObject), c_float, c_float)

sPhysicsRadSGetHitObject = _siberian.sPhysicsRadSGetHitObject
sPhysicsRadSGetHitObject.argtypes = (POINTER(sObject), uint32_t)
sPhysicsRadSGetHitObject.restype = POINTER(sObject)

sPhysicsRadSGetHitObjectCount = _siberian.sPhysicsRadSGetHitObjectCount
sPhysicsRadSGetHitObjectCount.argtypes = (POINTER(sObject),)
sPhysicsRadSGetHitObjectCount.restype = uint32_t

sPhysicsRSGetHitObject = _siberian.sPhysicsRSGetHitObject
sPhysicsRSGetHitObject.argtypes = (POINTER(sObject), uint32_t)
sPhysicsRSGetHitObject.restype = POINTER(sObject)

sPhysicsRSGetHitObjectCount = _siberian.sPhysicsRSGetHitObjectCount
sPhysicsRSGetHitObjectCount.argtypes = (POINTER(sObject),)
sPhysicsRSGetHitObjectCount.restype = uint32_t

sPhysicsCSGetHitObject = _siberian.sPhysicsCSGetHitObject
sPhysicsCSGetHitObject.argtypes = (POINTER(sObject), uint32_t)
sPhysicsCSGetHitObject.restype = POINTER(sObject)

sPhysicsCSGetHitObjectCount = _siberian.sPhysicsCSGetHitObjectCount
sPhysicsCSGetHitObjectCount.argtypes = (POINTER(sObject),)
sPhysicsCSGetHitObjectCount.restype = uint32_t

sPhysicsRSSetRange = _siberian.sPhysicsRSSetRange
_siberian.sPhysicsRSSetRange.argtypes = (POINTER(sPhysicsRS), c_float)
_siberian.sPhysicsRSSetRange.restype = None

sPhysicsRadarSetAngle = _siberian.sPhysicsRadarSetAngle
sPhysicsRadarSetAngle.argtypes = (POINTER(sPhysicsRS), c_float)
sPhysicsRadarSetAngle.restype = None

sPhysicsAutoDisable = _siberian.sPhysicsAutoDisable
sPhysicsAutoDisable.argtypes = (POINTER(sObject), c_bool)

_siberian.sPhysicsSetAngularVelocity.argtypes = (
    POINTER(sObject), c_double, c_double, c_double)
_siberian.sPhysicsSetAngularVelocity.restype = None

sPhysicsSetSpeedGlobal = _siberian.sPhysicsSetSpeedGlobal
sPhysicsSetSpeedGlobal.argtypes = (POINTER(sObject), laType, uint8_t)
sPhysicsSetSpeedGlobal.restype = None

sPhysicsSetSpeedXLocal = _siberian.sPhysicsSetSpeedXLocal
sPhysicsSetSpeedXLocal.argtypes = (POINTER(sObject), c_float)
sPhysicsSetSpeedXLocal.restype = None

sPhysicsSetSpeedYLocal = _siberian.sPhysicsSetSpeedYLocal
sPhysicsSetSpeedYLocal.argtypes = (POINTER(sObject), c_float)
sPhysicsSetSpeedYLocal.restype = None

sPhysicsSetSpeedZLocal = _siberian.sPhysicsSetSpeedZLocal
sPhysicsSetSpeedZLocal.argtypes = (POINTER(sObject), c_float)
sPhysicsSetSpeedZLocal.restype = None

_siberian.sPhysicsGetLinearVelocity.argtypes = (POINTER(sObject),)
_siberian.sPhysicsGetLinearVelocity.restype = laType

_siberian.sPhysicsSuspend.argtypes = POINTER(sObject),
_siberian.sPhysicsSuspend.restype = None

_siberian.sPhysicsResume.argtypes = POINTER(sObject),
_siberian.sPhysicsResume.restype = None

# Specific object functions


def sSkeletonSetPlayAction(skeleton, name, layer, act_type, start_frame, stop_frame, speed): return _siberian.sSkeletonSetPlayAction(
    skeleton, name.encode(), layer, act_type, start_frame, stop_frame, speed)


_siberian.sSkeletonSetPlayAction.argtypes = (
    POINTER(sObject), c_char_p, uint8_t, uint32_t, c_float, c_float, c_float)
_siberian.sSkeletonSetPlayAction.restype = None

sActionSetParam = _siberian.sActionSetParam
sActionSetParam.argtypes = POINTER(
    sObject), uint8_t, uint32_t, c_float, c_float, c_float
sActionSetParam.restype = None

_siberian.sActionProcess.argtypes = (POINTER(sObject),)
_siberian.sActionProcess.restype = None

sSkeletonSetActionInterval = _siberian.sSkeletonSetActionInterval
sSkeletonSetActionInterval.argtypes = POINTER(
    sObject), uint8_t, c_float, c_float
sSkeletonSetActionInterval.restype = None

sBoneGetSkeleton = _siberian.sBoneGetSkeleton
sBoneGetSkeleton.argtypes = (POINTER(sObject),)
sBoneGetSkeleton.restype = POINTER(sObject)

sBoneGetAnimatedFlag = _siberian.sBoneGetAnimatedFlag
sBoneGetAnimatedFlag.argtypes = (POINTER(sObject),)
sBoneGetAnimatedFlag.restype = c_int

sBoneSetAnimatedFlag = _siberian.sBoneSetAnimatedFlag
sBoneSetAnimatedFlag.argtypes = (POINTER(sObject), c_int)
sBoneSetAnimatedFlag.restype = None

sSkeletonGetActionFrame = _siberian.sSkeletonGetActionFrame
sSkeletonGetActionFrame.argtypes = (POINTER(sObject), c_int)
sSkeletonGetActionFrame.restype = c_float

sSkeletonSetActionFrame = _siberian.sSkeletonSetActionFrame
sSkeletonSetActionFrame.argtypes = POINTER(sObject), uint8_t, c_float
sSkeletonSetActionFrame.restype = None

sSkeletonGetActionFrame2 = _siberian.sSkeletonGetActionFrame2
sSkeletonGetActionFrame2.argtypes = (POINTER(sObject), c_int)
sSkeletonGetActionFrame2.restype = c_float

sSkeletonSetActionFrame2 = _siberian.sSkeletonSetActionFrame2
sSkeletonSetActionFrame2.argtypes = POINTER(sObject), c_int, c_float
sSkeletonSetActionFrame2.restype = None

sSkeletonResetPose = _siberian.sSkeletonResetPose
sSkeletonResetPose.argtypes = POINTER(sObject),
sSkeletonResetPose.restype = None

sSkeletonAddPoseFromLayerToPose = _siberian.sSkeletonAddPoseFromLayerToPose
sSkeletonAddPoseFromLayerToPose.argtypes = POINTER(
    sObject), uint8_t, c_float, c_float
sSkeletonAddPoseFromLayerToPose.restype = None

sSkeletonMixPoseFromLayerWithPose = _siberian.sSkeletonMixPoseFromLayerWithPose
sSkeletonMixPoseFromLayerWithPose.argtypes = POINTER(
    sObject), uint8_t, c_float, c_float
sSkeletonMixPoseFromLayerWithPose.restype = None

sSkeletonAddPoseFromActionToPose = _siberian.sSkeletonAddPoseFromActionToPose
sSkeletonAddPoseFromActionToPose.argtypes = POINTER(
    sObject), c_char_p, uint32_t, c_float, c_float
sSkeletonAddPoseFromActionToPose.restype = None

sSkeletonMixPoseFromActionWithPose = _siberian.sSkeletonMixPoseFromActionWithPose
sSkeletonMixPoseFromActionWithPose.argtypes = POINTER(
    sObject), c_char_p, uint32_t, c_float, c_float
sSkeletonMixPoseFromActionWithPose.restype = None

sActionIsPlaying = _siberian.sActionIsPlaying
sActionIsPlaying.argtypes = (POINTER(sObject), uint8_t)
sActionIsPlaying.restype = uint8_t

sActionStop = _siberian.sActionStop
sActionStop.argtypes = (POINTER(sObject), uint8_t)

sSkeletonSetAction = _siberian.sSkeletonSetAction
sSkeletonSetAction.argtypes = (POINTER(sObject), uint8_t, c_char_p)

sSkeletonGetBone = _siberian.sSkeletonGetBone
sSkeletonGetBone.argtypes = (POINTER(sObject), c_char_p)
sSkeletonGetBone.restype = POINTER(sObject)

sSkeletonGetBoneByIndex = _siberian.sSkeletonGetBoneByIndex
sSkeletonGetBoneByIndex.argtypes = (POINTER(sObject), uint16_t)
sSkeletonGetBoneByIndex.restype = POINTER(sObject)

sSkeletonGetBoneCount = _siberian.sSkeletonGetBoneCount
sSkeletonGetBoneCount.argtypes = (POINTER(sObject),)
sSkeletonGetBoneCount.restype = uint16_t

sSkeletonSetLayerWeight = _siberian.sSkeletonSetLayerWeight
sSkeletonSetLayerWeight.argtypes = POINTER(sObject), uint8_t, c_float
sSkeletonSetLayerWeight.restype = None

sSkeletonSetLayerTime = _siberian.sSkeletonSetLayerTime
sSkeletonSetLayerTime.argtypes = POINTER(sObject), uint8_t, c_float
sSkeletonSetLayerTime.restype = None

sSkeletonGetLayerTime = _siberian.sSkeletonGetLayerTime
sSkeletonGetLayerTime.argtypes = POINTER(sObject), uint8_t
sSkeletonGetLayerTime.restype = c_float

sSkeletonSetLayerSpeed = _siberian.sSkeletonSetLayerSpeed
sSkeletonSetLayerSpeed.argtypes = POINTER(sObject), uint8_t, c_float
sSkeletonSetLayerSpeed.restype = None

sSkeletonGetLayerSpeed = _siberian.sSkeletonGetLayerSpeed
sSkeletonGetLayerSpeed.argtypes = POINTER(sObject), uint8_t
sSkeletonGetLayerSpeed.restype = c_float


class sCamera(Structure):
    _fields_ = sObjectBase._fields_[:]
    _fields_.extend([
        ('__projection', laType),
        ('__viewProjection', laType),
        ('__noise', GLuint),
        ('__render_texture', GLuint),
        ('__render_result', GLuint),
        ('__render_texture1', GLuint),
        ('__render_texture2', GLuint),
        ('__render_normal', GLuint),
        # ('__render_distance',GLuint),
        ('__render_distance_glass', GLuint),
        ('__render_specular', GLuint),
        ('__render_ambient', GLuint),
        ('__render_vectors', GLuint),
        ('__render_depth', GLuint),
        ('__render_buffer', GLuint),
        ('__render_fb', GLuint),

        ('__render_normals_texture', GLuint),
        ('__render_normals_depth', GLuint),
        ('__render_normals_buffer', GLuint),
        ('__render_normals_fb', GLuint),

        ('__render_plane', sMesh),
        ('__filters', POINTER(sShader)*8),
        ('__mipmap_layers', uint32_t),
        # ('skybox',sShader),
        ('zNear', c_float),
        ('zFar', c_float),
        ('FOV', c_float),
        ('_width', uint16_t),
        ('_height', uint16_t),
        ('_view_point', POINTER(sObject))])

    @property
    def height(self):
        return self._height

    @property
    def width(self):
        return self._width

    @property
    def view_point(self):
        if self._view_point:
            return self._view_point.contents
        else:
            None

    @view_point.setter
    def view_point(self, obj):
        if isinstance(obj, POINTER(sObject)):
            self._view_point = obj
        elif isinstance(obj, sObject):
            self._view_point = pointer(obj)


class sLight(Structure):
    _fields_ = sCamera._fields_[:]
    _fields_.extend([
        ('type', uint8_t),
        ('color', sColour),
        ('inner', c_float),
        ('outer', c_float),
        ('shadow', c_bool)])


class sScene(Structure):
    _fields_ = [
        ('_behaviour', behaviour),
        ('_camera', sCamera),
        ('_meshes', POINTER(sMesh)),
        ('_materials', POINTER(sMaterial)),
        ('_textures', POINTER(sTexture)),
        ('_cubemap', POINTER(sTexture)),
        ('_lights', POINTER(POINTER(sLight))),
        ('_objects', POINTER(POINTER(sObject))),
        ('_skelets', POINTER(c_void_p)),  # sSkeleton
        ('_lights_inactive', POINTER(sLight)),
        ('_objects_inactive', POINTER(sObject)),
        ('_skelets_inactive', c_void_p),
        ('_actions', c_void_p),

        ('_gobjects', POINTER(POINTER(c_void_p))),
        ('_gobjects_count', index_t),
        ('_gobjects_counter', index_t),

        ('_shader_list', POINTER(sShader)*8),

        ('_mesh_count', index_t),
        ('_material_count', index_t),
        ('_texture_count', index_t),
        ('_lights_count', index_t),
        ('_objects_count', index_t),
        ('_skelets_count', index_t),
        ('_lights_inactive_count', uint32_t),
        ('_objects_inactive_count', index_t),
        ('_skelets_inactive_count', index_t),
        ('_actions_count', index_t),
        ('_world', dWorldID),
        ('_space', dSpaceID),
        ('_contactgroup', dJointGroupID),
        ('__joints', dJointGroupID)]

    __scenes = {}

    def __init__(self, *args, **kwargs):
        Structure.__init__(self)
        _siberian.sSceneLoad(self, kwargs['filename'].encode())
        print("Adding scene to static list")
        sScene.__scenes[addressof(self)] = {
            'scene': self, 'pyfunctions': {}, 'pydata': {}}
        self.__pyfunctions = sScene.__scenes[addressof(self)]['pyfunctions']
        self.__pydata = sScene.__scenes[addressof(self)]['pydata']

    @property
    def pyfunctions(self):
        return sScene.__scenes[addressof(self)]['pyfunctions']

    @property
    def pydata(self):
        return sScene.__scenes[addressof(self)]['pydata']

    @property
    def camera(self):
        return self._camera

    def getObject(self, name):
        ptr = _siberian.sSceneGetObject(pointer(self), c_char_p(name.encode()))
        if ptr:
            return ptr.contents
        else:
            None

    def getMaterial(self, name):
        return _siberian.sSceneGetMaterial(self, name).contents

    def addObject(self, name):
        ptr = _siberian.sSceneAddObject(pointer(self), c_char_p(name.encode()))
        if ptr:
            return ptr.contents
        else:
            return None

    def loadAction(self, path, name):
        sActionLoad(pointer(self), path.encode(), name.encode())

    def setScript(self, func):
        def f(scene):
            func(cast(scene, POINTER(sScene)).contents)
        script = behaviour(f)
        sScene.__scenes[addressof(self)]['script'] = script
        _siberian.sSceneSetScript(self, script)

    def setSkyTexture(self, texture):
        _siberian.sSceneSetSkyTexture(self, texture)

    def loadMesh(self, name):
        if isinstance(name, bytes):
            return _siberian.sSceneAddMesh(self, name).contents
        elif isinstance(name, str):
            return _siberian.sSceneAddMesh(self, name.encode()).contents
        else:
            raise TypeError(
                "loadMesh's \"name\" argument must be str or bytes")

    def removeMesh(self, name):
        if isinstance(name, bytes):
            _siberian.sSceneRemoveMesh(self, name)
        elif isinstance(name, str):
            _siberian.sSceneRemoveMesh(self, name.encode())
        else:
            raise TypeError(
                "loadMesh's \"name\" argument must be str or bytes")

    def destroy(self):
        _siberian.sSceneFree(self)
        del sScene.__scenes[addressof(self)]


"""
    sSound functions
"""
SOUND_NO_DEVICE = -2
SOUND_NO_CONTEXT = -1
SOUND_OK = 0
SOUND_OAL_ERROR = 1
SOUND_FILE_NOT_FOUND = 2
SOUND_DOES_NOT_EXISTS = 3

ALC_INVALID_DEVICE = 0xA001
ALC_INVALID_CONTEXT = 0xA002
ALC_INVALID_ENUM = 0xA003
ALC_INVALID_VALUE = 0xA004
ALC_OUT_OF_MEMORY = 0xA005

SOUND_ERRORS = {}
SOUND_ERRORS[-2] = 'sSound: Failed to initialize sound device', RuntimeError
SOUND_ERRORS[-1] = 'sSound: Failed to initialize context', RuntimeError
SOUND_ERRORS[0] = 'sSound: OK', None
SOUND_ERRORS[1] = 'sSound: Failed to initialize OpenAL. Have you install it?', RuntimeError
SOUND_ERRORS[2] = 'sSound: File not found', FileNotFoundError
SOUND_ERRORS[3] = 'sSound: Sound not found in loaded sounds', KeyError

_siberian.sSoundInit.restype = c_int
_siberian.sSoundInit.argtypes = ()

_siberian.sSoundLoad.argtypes = (c_char_p,)
_siberian.sSoundLoad.restype = c_int

_siberian.sSoundAttachToObject.argtypes = (POINTER(sObject), c_char_p)
_siberian.sSoundAttachToObject.restype = c_int


def sSoundInit():
    print(SOUND_ERRORS[_siberian.sSoundInit()][0])


def sSoundLoad():
    result = _siberian.sSoundInit()
    if result != 0:
        raise SOUND_ERRORS[result][1](SOUND_ERRORS[result][0])


def sSoundAttachToObject(name):
    if isinstance(name, str):
        name = name.encode()
    result = _siberian.sSoundAttachToObject(name)
    if result != 0:
        raise SOUND_ERRORS[result][1](SOUND_ERRORS[result][0])


sSoundCloseDevice = _siberian.sSoundCloseDevice

# Interface
def c_string(val, coding = 'cp1251'):
    if isinstance(value, str):
        value = value.encode(coding)
    elif isinstance(value, bytes):
        pass
    else:
        value = str(value).encode(coding)
    return value

class fElement(Structure):
    _fields_ = [('__data', uint8_t*224)]

    @property
    def lockRotation(self):
        return _siberian.fElementGetLockRotationBit(self)
    @lockRotation.setter
    def lockRotation(self, val):
        _siberian.fElementSetLockRotationBit(self, val)

    @property
    def visible(self):
        return _siberian.fElementGetVisibleBit(self)
    @visible.setter
    def visible(self, val):
        _siberian.fElementSetVisibleBit(self, val)

    @property
    def height(self):
        return _siberian.fElementGetHeight(self)
    @height.setter
    def height(self, size):
        _siberian.fElementSetHeight(self, size)

    @property
    def width(self):
        return _siberian.fElementGetWidth(self)
    @width.setter
    def width(self, size):
        _siberian.fElementSetWidth(self, size)

    @property
    def text(self):
        return _siberian.fElementGetTextPtr(self).decode('cp1251')

    @text.setter
    def text(self, value):
        _siberian.fElementSetText(self, c_string(value))

    @property
    def planeColor(self):
        var = (c_float*4)(0,0,0,0)
        _siberian.fElementGetPlaneColor4fv(self, var)
        return var

    @planeColor.setter
    def planeColor(self, val):
        _siberian.fElementGetPlaneColor4fv(self, (c_float*4)(*val))

    @property
    def textColor(self):
        var = (c_float*4)()
        _siberian.fElementGetTextColor4fv(self, var)
        return var

    @textColor.setter
    def textColor(self, val):
        _siberian.fElementGetTextColor4fv(self, (c_float*4)(*val))

    def moveToTopLayer(self):
        _siberian.fElementSetTopLayer(self)

    def moveToBottomLayer(self):
        _siberian.fElementSetBottomLayer(self)

    def moveLayerUp(self):
        _siberian.fElementMoveLayerUp(self)

    def moveLayerDown(self):
        _siberian.fElementMoveLayerDown(self)

fElement_p = POINTER(fElement)

_siberian.fElementSetVisibleBit.argtypes = (fElement_p, c_bool)
_siberian.fElementSetVisibleBit.restype  = None

_siberian.fElementGetVisibleBit.argtypes = (fElement_p,)
_siberian.fElementGetVisibleBit.restype  = c_bool

_siberian.fElementSetLockRotationBit.argtypes = (fElement_p, c_bool)
_siberian.fElementSetLockRotationBit.restype  = None

_siberian.fElementGetLockRotationBit.argtypes = (fElement_p,)
_siberian.fElementGetLockRotationBit.restype  = c_bool

_siberian.fElementSetText.argtypes = (fElement_p, c_char_p)
_siberian.fElementSetText.restype  = None

_siberian.fElementGetTextLength.argtypes = (fElement_p,)
_siberian.fElementGetTextLength.restype  = c_int

_siberian.fElementGetText.argtypes = (fElement_p, c_char_p, c_int)
_siberian.fElementGetText.restype  = None

_siberian.fElementGetTextPtr.argtypes = (fElement_p,)
_siberian.fElementGetTextPtr.restype  = c_char_p

_siberian.fElementSetTopLayer.argtypes = (fElement_p,)
_siberian.fElementSetTopLayer.restype  = None

_siberian.fElementSetBottomLayer.argtypes = (fElement_p,)
_siberian.fElementSetBottomLayer.restype  = None

_siberian.fElementMoveLayerDown.argtypes = (fElement_p,)
_siberian.fElementMoveLayerDown.restype  = None

_siberian.fElementMoveLayerUp.argtypes = (fElement_p,)
_siberian.fElementMoveLayerUp.restype  = None

_siberian.fElementGetWidth.argtypes = (fElement_p,)
_siberian.fElementGetWidth.restype  = c_float

_siberian.fElementSetWidth.argtypes = (fElement_p,c_float)
_siberian.fElementSetWidth.restype  = None

_siberian.fElementGetHeight.argtypes = (fElement_p,)
_siberian.fElementGetHeight.restype  = c_float

_siberian.fElementSetHeight.argtypes = (fElement_p,c_float)
_siberian.fElementSetHeight.restype  = None

_siberian.fElementSetFont.argtypes = (fElement_p, sTexture_p)
_siberian.fElementSetFont.restype  = None
#########
_siberian.fElementSetLocalPosition.argtypes = (fElement_p, c_float, c_float)
_siberian.fElementSetLocalPosition.restype  = None

_siberian.fElementSetGlobalPosition.argtypes = (fElement_p, c_float, c_float)
_siberian.fElementSetGlobalPosition.restype  = None

_siberian.fElementGetLocalPosition.argtypes = (fElement_p, POINTER(c_float), POINTER(c_float))
_siberian.fElementGetLocalPosition.restype  = None

_siberian.fElementGetGlobalPosition.argtypes = (fElement_p, POINTER(c_float), POINTER(c_float))
_siberian.fElementGetGlobalPosition.restype  = None

_siberian.fElementTranslateLocal.argtypes = (fElement_p, c_float, c_float)
_siberian.fElementTranslateLocal.restype  = None

_siberian.fElementTranslateGlobal.argtypes = (fElement_p, c_float, c_float)
_siberian.fElementTranslateGlobal.restype  = None
#########

_siberian.fElementSetPlaneColor4fv.argtypes = (fElement_p, POINTER(c_float))
_siberian.fElementSetPlaneColor4fv.restype  = None

_siberian.fElementGetPlaneColor4fv.argtypes = (fElement_p, POINTER(c_float))
_siberian.fElementGetPlaneColor4fv.restype  = None

_siberian.fElementSetTextColor4fv.argtypes = (fElement_p, POINTER(c_float))
_siberian.fElementSetTextColor4fv.restype  = None

_siberian.fElementGetTextColor4fv.argtypes = (fElement_p, POINTER(c_float))
_siberian.fElementGetTextColor4fv.restype  = None

_siberian.fElementRotate.argtypes = (fElement_p,c_float)
_siberian.fElementRotate.restype = None

_siberian.fElementGetLocalRotation.argtypes = (fElement_p,)
_siberian.fElementGetLocalRotation.restype  = c_float

_siberian.fElementGetGlobalRotation.argtypes = (fElement_p,)
_siberian.fElementGetGlobalRotation.restype  = c_float

_siberian.fElementSetLocalRotation.argtypes = (fElement_p, c_float)
_siberian.fElementSetLocalRotation.restype  = None

_siberian.fElementSetGlobalRotation.argtypes = (fElement_p, c_float)
_siberian.fElementSetGlobalRotation.restype  = None

class fForm(Structure):
    _fields_ = [('__data', uint8_t*224)]

    __instances__ = {}

    def __init__(self):
        Structure.__init__(self)
        struct = fForm.__instances__[addressof(self)] = {}
        struct["attribs"] = {}
        struct['instance'] = self
        struct['attributes'] = {}
        _siberian.fFormConstructor(self)

    @property
    def height(self):
        return _siberian.fFormGetHeight(self)
    @height.setter
    def height(self, size):
        _siberian.fFormSetHeight(self, size)

    @property
    def width(self):
        return _siberian.fFormGetWidth(self)
    @width.setter
    def width(self, size):
        _siberian.fFormSetWidth(self, size)

    @property
    def localRotation(self):
        return _siberian.fFormGetLocalRotation(self)

    @localRotation.setter
    def localRotation(self, angle):
        _siberian.fFormSetLocalRotation(self, angle)

    @property
    def globalRotation(self):
        return _siberian.fFormGetGlobalRotation(self)

    @globalRotation.setter
    def globalRotation(self, angle):
        _siberian.fFormSetGlobalRotation(self, angle)

    @property
    def verticalScrollValue(self):
        return _siberian.fFormGetVerticalScrolling(self)

    @verticalScrollValue.setter
    def verticalScrollValue(self, val):
        _siberian.fFormSetVerticalScrolling(self, val)

    @property
    def horizontalScrollValue(self):
        return _siberian.fFormGetHorizontalScrolling(self)
        
    @horizontalScrollValue.setter
    def horizontalScrollValue(self, val):
        _siberian.fFormSetHorizontalScrolling(self, val)

    def removeParent(self):
        _siberian.fFormRemoveParent(self)
    
    def delete(self):
        _siberian.fFormDelete(self)

    def addForm(self, form):
        _siberian.fFormAddForm(self, form)

    def addElement(self, width, height, text=None, font_size=8):
        if isinstance(text, str):
            text = text.encode('1251')
        elif isinstance(text, bytes) or text is None:
            pass
        else:
            text = str(text).encode('1251')
        return _siberian.fFormAddElement(self, text, font_size, width, height)

    def setIdle(self, callback):
        fForm.__instances__[addressof(self)]['idleCallback'] = {}
        fForm.__instances__[addressof(self)]['idleCallback']['py'] = callback
        c_callback = fForm.__instances__[addressof(self)]['idleCallback']['c'] = __fFormCallback(callback)
        _siberian.fFormSetIdle(self, c_callback)

    def setLMB(self, callback):
        fForm.__instances__[addressof(self)]['lmbCallback'] = {}
        fForm.__instances__[addressof(self)]['lmbCallback']['py'] = callback
        c_callback = fForm.__instances__[addressof(self)]['lmbCallback']['c'] = __fFormCallback(callback)
        _siberian.fFormSetLMB(self, c_callback)

    def setRMB(self, callback):
        fForm.__instances__[addressof(self)]['rmbCallback'] = {}
        fForm.__instances__[addressof(self)]['rmbCallback']['py'] = callback
        c_callback = fForm.__instances__[addressof(self)]['rmbCallback']['c'] = __fFormCallback(callback)
        _siberian.fFormSetRMB(self, c_callback)

    def setScroll(self, callback):
        fForm.__instances__[addressof(self)]['scrollCallback'] = {}
        fForm.__instances__[addressof(self)]['scrollCallback']['py'] = callback
        c_callback = fForm.__instances__[addressof(self)]['scrollCallback']['c'] = __fFormScrollCallback(callback)
        _siberian.fFormSetScroll(self, c_callback)

    def setCursorHover(self, callback):
        fForm.__instances__[addressof(self)]['hoverCallback'] = {}
        fForm.__instances__[addressof(self)]['hoverCallback']['py'] = callback
        c_callback = fForm.__instances__[addressof(self)]['hoverCallback']['c'] = __fFormCallback(callback)
        _siberian.fFormSetCursorHover(self, c_callback)

    def setCursorRelease(self, callback):
        fForm.__instances__[addressof(self)]['releaseCallback'] = {}
        fForm.__instances__[addressof(self)]['releaseCallback']['py'] = callback
        c_callback = fForm.__instances__[addressof(self)]['releaseCallback']['c'] = __fFormCallback(callback)
        _siberian.fFormSetCursorHover(self, c_callback)

    def translateGlobal(self, x, y):
        _siberian.fFormTranslateGlobal(self, x, y)

    def translateLocal(self, x, y):
        _siberian.fFormTranslateLocal(self, x, y)

    def rotate(self, angle):
        _siberian.fFormRotate(self, angle)

fForm_p = POINTER(fForm)

__fFormCallback = CFUNCTYPE(None, fForm_p)
__fFormScrollCallback = CFUNCTYPE(None, fForm_p, c_int)

_siberian.fFormCreate.argtypes = tuple()
_siberian.fFormCreate.restype = fForm_p

_siberian.fFormConstructor.argtypes = (fForm_p,)
_siberian.fFormConstructor.restype = None

_siberian.fFormRemoveParent.argtypes = (fForm_p,)
_siberian.fFormRemoveParent.restype = None

_siberian.fFormMarkDelete.argtypes = (fForm_p,)
_siberian.fFormMarkDelete.restype = None

_siberian.fFormAddForm.argtypes = (fForm_p, fForm_p)
_siberian.fFormAddForm.restype = None

_siberian.fFormAddElement.argtypes = (fForm_p, c_char_p, c_int, c_float, c_float)
_siberian.fFormAddElement.restype = fElement_p

_siberian.fFormSetIdle.argtypes = (fForm_p, __fFormCallback)
_siberian.fFormSetIdle.restype = None

_siberian.fFormSetLMB.argtypes = (fForm_p, __fFormCallback)
_siberian.fFormSetLMB.restype = None

_siberian.fFormSetRMB.argtypes = (fForm_p, __fFormCallback)
_siberian.fFormSetRMB.restype = None

_siberian.fFormSetScroll.argtypes = (fForm_p, __fFormScrollCallback)
_siberian.fFormSetScroll.restype = None

_siberian.fFormSetCursorHover.argtypes = (fForm_p, __fFormCallback)
_siberian.fFormSetCursorHover.restype = None

_siberian.fFormSetCursorLeave.argtypes = (fForm_p, __fFormCallback)
_siberian.fFormSetCursorLeave.restype = None

_siberian.fFormSetTopLayer.argtypes = (fForm_p,)
_siberian.fFormSetTopLayer.restype = None

#_siberian.fFormSetBottomLayer.argtypes = (fForm_p,)
#_siberian.fFormSetBottomLayer.restype = None

#_siberian.fFormSetLayerUp.argtypes = (fForm_p,)
#_siberian.fFormSetLayerUp.restype = None

#_siberian.fFormSetLayerDown.argtypes = (fForm_p,)
#_siberian.fFormSetLayerDown.restype = None

_siberian.fFormSetLocalPosition.argtypes = (fForm_p, c_float, c_float)
_siberian.fFormSetLocalPosition.restype  = None

_siberian.fFormSetGlobalPosition.argtypes = (fForm_p, c_float, c_float)
_siberian.fFormSetGlobalPosition.restype  = None

_siberian.fFormTranslateLocal.argtypes = (fForm_p, c_float, c_float)
_siberian.fFormTranslateLocal.restype  = None

_siberian.fFormTranslateGlobal.argtypes = (fForm_p, c_float, c_float)
_siberian.fFormTranslateGlobal.restype  = None

_siberian.fFormScrollVertical.argtypes = (fForm_p, c_float)
_siberian.fFormScrollVertical.restype = None

_siberian.fFormScrollHorizontal.argtypes = (fForm_p, c_float)
_siberian.fFormScrollHorizontal.restype = None

_siberian.fFormSetVerticalScrolling.argtypes = (fForm_p, c_float)
_siberian.fFormSetVerticalScrolling.restype  = None

_siberian.fFormSetHorizontalScrolling.argtypes = (fForm_p, c_float)
_siberian.fFormSetHorizontalScrolling.restype  = None

_siberian.fFormGetVerticalScrolling.argtypes = (fForm_p,)
_siberian.fFormGetVerticalScrolling.restype  = c_float

_siberian.fFormGetHorizontalScrolling.argtypes = (fForm_p,)
_siberian.fFormGetHorizontalScrolling.restype  = c_float

_siberian.fFormRotate.argtypes = (fForm_p,c_float)
_siberian.fFormRotate.restype = None

_siberian.fFormSetRotationLocal.argtypes = (fForm_p, c_float)
_siberian.fFormSetRotationLocal.restype  = None

_siberian.fFormSetRotationGlobal.argtypes = (fForm_p, c_float)
_siberian.fFormSetRotationGlobal.restype  = None

_siberian.fFormGetLocalRotation.argtypes = (fForm_p,)
_siberian.fFormGetLocalRotation.restype  = c_float

_siberian.fFormGetGlobalRotation.argtypes = (fForm_p,)
_siberian.fFormGetGlobalRotation.restype  = c_float

_siberian.fFormGetWidth.argtypes = (fForm_p,)
_siberian.fFormGetWidth.restype  = c_float

_siberian.fFormSetWidth.argtypes = (fForm_p,c_float)
_siberian.fFormSetWidth.restype  = None

_siberian.fFormGetHeight.argtypes = (fForm_p,)
_siberian.fFormGetHeight.restype  = c_float

_siberian.fFormSetHeight.argtypes = (fForm_p,c_float)
_siberian.fFormSetHeight.restype  = None

class fButton(Structure):
    _fields_ = [('__data', uint8_t*224)]
    __instances__ = {}
    
    def __init__(self, text, x, y, width, height, callback=None):
        text = c_string(text)
        struct = fButton.__instances__[addressof(self)] = {}
        struct['attribs'] = {}
        if callback is None:
            _siberian.fButtonConstructor(pointer(self), text, x, y, width, height, None)
        else:
            struct['callback'] = {}
            struct['callback']['py'] = callback
            struct['callback']['c'] = buttonCallback(callback)
            _siberian.fButtonConstructor(pointer(self), text, x, y, width, height, struct['callback']['c'])

    def setCallback(self, callback):
        struct = fButton.__instances__[addressof(self)]
        struct['callback'] = {}
        struct['callback']['py'] = callback
        struct['callback']['c'] = buttonCallback(callback)
        _siberian.fButtonSetCallback(self, struct['callback']['c'])

    def setText(self, text):
        text = c_string(text)
        _siberian.fButtonSetText(self, text)

    def delete(self):
        del fButton.__instances__[addressof(self)]
        _siberian.fButtonDelete(self)

buttonCallback = CFUNCTYPE(None, POINTER(fButton))

_siberian.fButtonConstructor.argtypes = (POINTER(fButton), c_char_p, c_float, c_float, c_int, c_int)
_siberian.fButtonConstructor.restype = None

_siberian.fButtonSetCallback.argtypes = (POINTER(fButton), buttonCallback)
_siberian.fButtonSetCallback.restype = None

_siberian.fButtonSetText.argtypes = (POINTER(fButton), c_char_p)
_siberian.fButtonSetText.restype = None

_siberian.fButtonDelete.argtypes = (POINTER(fButton),)
_siberian.fButtonDelete.restype = None

class fList(Structure):
    _fields_ = [('__data', uint8_t*224)]
    __instances__ = {}
    
    def __init__(self, x, y, width, height, callback):
        struct = fButton.__instances__[addressof(self)] = {}
        struct['callback'] = {}
        struct['callback']['py'] = callback
        struct['callback']['c'] = listCallback(callback)
        _siberian.fListConstructor(pointer(self), x, y, width, height, struct['callback']['c'])

    def addItem(self, text):
        if isinstance(text, bytes):
            pass
        elif isinstance(text, str):
            text = text.encode('cp1251')
        else:
            text = str(text).encode('cp1251')
        _siberian.fListAddItem(self, text)

    def removeItem(self, num):
        if isinstance(num, (int, float)):
            _siberian.fListRemoveItem(self, int(num))
        else:
            raise AttributeError("fList.removeItem argument must be numeric")

    def delete(self):
        _siberian.fListDelete(self)

listCallback = CFUNCTYPE(None, POINTER(fList), c_int)

_siberian.fListConstructor.argtypes = (POINTER(fList), c_float, c_float, c_float, c_float, listCallback)
_siberian.fListConstructor.restype = None

_siberian.fListAddItem.argtypes = (POINTER(fList),c_char_p)
_siberian.fListAddItem.restype  = None

_siberian.fListRemoveItem.argtypes = (POINTER(fList),c_int)
_siberian.fListRemoveItem.restype  = None

_siberian.fListDelete.argtypes = (POINTER(fList),)
_siberian.fListDelete.restype  = None
# """

sActionLoad = _siberian.sActionLoad
sActionLoad.argtypes = (POINTER(sScene), c_char_p, c_char_p)

sObjectGetScene = _siberian.sObjectGetScene
_siberian.sObjectGetScene.restype = POINTER(sScene)
_siberian.sObjectGetScene.argtypes = (POINTER(sObject),)

sSceneSetScript = _siberian.sSceneSetScript
sSceneSetScript.restype = None
sSceneSetScript.argtypes = (POINTER(sScene), behaviour)

sSceneGetObject = _siberian.sSceneGetObject
_siberian.sSceneGetObject.restype = POINTER(sObject)
_siberian.sSceneGetObject.argtypes = (POINTER(sScene), c_char_p)

_siberian.sSceneGetMaterial.argtypes = POINTER(sScene), c_char_p
_siberian.sSceneGetMaterial.restype = POINTER(sMaterial)

sSceneAddObject = _siberian.sSceneAddObject
_siberian.sSceneAddObject.restype = POINTER(sObject)
_siberian.sSceneAddObject.argtypes = (POINTER(sScene), c_char_p)

_siberian.sSceneLoad.argtypes = POINTER(sScene), c_char_p
_siberian.sSceneLoad.restype = None

_siberian.sSceneSetSkyTexture.argtypes = POINTER(sScene), POINTER(sTexture)
_siberian.sSceneSetSkyTexture.restype = None

_siberian.sSceneAddMesh.argtypes = POINTER(sScene), c_char_p
_siberian.sSceneAddMesh.restype = POINTER(sMesh)

_siberian.sSceneRemoveMesh.argtypes = POINTER(sScene), c_char_p
_siberian.sSceneRemoveMesh.restype = None

_siberian.sSceneFree.argtypes = POINTER(sScene),
_siberian.sSceneFree.restype = None

sEngineSetSwapInterval = _siberian.sEngineSetSwapInterval
sEngineSetSwapInterval.argtypes = (uint32_t,)
sEngineSetSwapInterval.restype = None

sEngineSetActiveScene = _siberian.sEngineSetActiveScene
sEngineSetActiveScene.argtypes = POINTER(sScene),
sEngineSetActiveScene.restype = None

sGetFrameTime = _siberian.sGetFrameTime
sGetFrameTime.restype = double

sGetProfilingString = _siberian.sGetProfilingString
sGetProfilingString.restype = c_char_p

sEngineCreateWindow = _siberian.sEngineCreateWindow
sEngineCreateWindow.argtypes = (uint16_t, uint16_t, uint16_t)

sMouseGetKeyState = _siberian.sMouseGetKeyState
sMouseGetKeyState.argtypes = (c_int,)
sMouseGetKeyState.restype = c_int

sMouseShow = _siberian.sMouseShow
sMouseShow.restype = None

sMouseHide = _siberian.sMouseHide
sMouseHide.restype = None

sMouseGetVerticalScroll = _siberian.sMouseGetVerticalScroll
sMouseGetVerticalScroll.restype = c_float

sMouseSetPosition = _siberian.sSetMousePosition
sMouseSetPosition.argtypes = (c_float, c_float)

sGetMouseDelta = _siberian.sGetMouseDelta
sGetMouseDelta.restype = c_float*2

_siberian.sGetMousePosition.restype = None
_siberian.sGetMousePosition.argtypes = (POINTER(laType),)


def sMouseGetPosition():
    vec = Vector(0, 0, 0)
    _siberian.sGetMousePosition(pointer(vec))
    return vec


sKeyboardGetKeyState = _siberian.sKeyboardGetKeyState
sKeyboardGetKeyState.argtypes = (c_int,)
sKeyboardGetKeyState.restype = c_int

# game_functions
#sScreenshot = _siberian.screenshot
_siberian.sCharacterInit.argtypes = (
    POINTER(sScene), POINTER(sObject), c_char_p)
_siberian.sCharacterInit.restype = POINTER(sObject)


def sCharacterInit(scene, obj, name):
    return _siberian.sCharacterInit(scene, obj, c_char_p(name.encode() if isinstance(name, str) else name)).contents


_siberian.sMobInit.argtypes = (
    POINTER(sScene), POINTER(sObject), c_char_p, laType)
_siberian.sMobInit.restype = POINTER(sObject)


def sMobInit(scene, obj, name, bbox): return _siberian.sMobInit(
    scene, obj, name.encode(), bbox).contents


class sVehicle4Wheel(Structure):
    _fields_ = [("_body", POINTER(sObject)),
                ("_flw", POINTER(sObject)),
                ("_frw", POINTER(sObject)),
                ("_blw", POINTER(sObject)),
                ("_brw", POINTER(sObject)),
                ("_fls", POINTER(sObject)),
                ("_frs", POINTER(sObject)),
                ("_bls", POINTER(sObject)),
                ("_brs", POINTER(sObject)),
                ("_max_speed", c_float),
                ("_max_torque", c_float),
                ("_max_speed", c_void_p*8),
                ("_jointGroup", c_void_p),
                ("_drive_wheels", uint8_t),
                ("_spring_damping", c_float),
                ("_spring_force", c_float),
                ("_rpm", c_float),
                ("_acceleration", c_float),
                ("_transmission", c_char),
                ("_power", c_float),
                ("_breaks", c_int),
                ("_gas", c_float)]

    def control(self):
        _siberian.sVehicleController(self, cast(0, POINTER(sObject)))

    def camControl(self, camera):
        _siberian.sVehicleController(self, camera)

    def setController(self):
        body = getattr(self, "_body").contents
        body['controller'] = self
        body.setBehaviour(lambda obj: obj['controller'].control())

    def setCamController(self, camera):
        body = getattr(self, "_body").contents
        body['controller'] = self
        body['camera'] = camera
        body.setBehaviour(
            lambda obj: obj['controller'].camControl(obj['camera']))

    def steer(self, amount, wheels):
        _siberian.sVehicleTurn(self, amount, wheels)

    def setTireFriction(self, friction):
        _siberian.sVehicleSetTireFriction(self, friction)

    @property
    def collider(self):
        return getattr(self, "_body").contents


_siberian.sVehicleInit.argtypes = POINTER(
    sScene), POINTER(sVehicle4Wheel), c_char_p
_siberian.sVehicleInit.restype = None

_siberian.sVehicleController.argtypes = POINTER(
    sVehicle4Wheel), POINTER(sObject)
_siberian.sVehicleController.restype = None

_siberian.sVehicleSetTireFriction.argtypes = POINTER(sVehicle4Wheel), c_float
_siberian.sVehicleSetTireFriction.restype = None

_siberian.sVehicleSetMaxSpeedKPH.argtypes = POINTER(sVehicle4Wheel), c_float
_siberian.sVehicleSetMaxSpeedKPH.restype = None

_siberian.sVehicleTurn.argtypes = POINTER(sVehicle4Wheel), c_float, uint8_t
_siberian.sVehicleTurn.restype = None


class sPhysicsJoint(c_void_p):
    def __init__(self):
        c_void_p.__init__(self)
        self.__angle = 0.0
        self.__force = 0.0
        self.__angle2 = 0.0
        self.__force2 = 0.0

    def setRateAndForce(self, rate, force):
        if not self:
            raise Exception('sPhysicsJoint: this joint is not initialized')
        self.__angle, self.__force = rate, force
        _siberian.sPhysicsJointSetAngle1Rate(self, rate, force)

    def setRateAndForce2(self, rate, force):
        if not self:
            raise Exception('sPhysicsJoint: this joint is not initialized')
        self.__angle2, self.__force2 = rate, force
        _siberian.sPhysicsJointSetAngle2Rate(self, rate, force)

    @property
    def axisCount(self):
        return _siberian.sPhysicsJointGetAxisCount(self)

    @property
    def angle(self):
        if not self:
            raise Exception('sPhysicsJoint: this joint is not initialized')
        return _siberian.sPhysicsJointGetAngle1(self)

    @property
    def angle2(self):
        if not self:
            raise Exception('sPhysicsJoint: this joint is not initialized')
        return _siberian.sPhysicsJointGetAngle2(self)

    @property
    def angleRate(self):
        if not self:
            raise Exception('sPhysicsJoint: this joint is not initialized')
        return _siberian.sPhysicsJointGetAngle1Rate(self)

    @angleRate.setter
    def angleRate(self, angle):
        if not self:
            raise Exception('sPhysicsJoint: this joint is not initialized')
        self.__angle = angle
        if not hasattr(self, '__force'):
            self.__force = 0.0
        _siberian.sPhysicsJointSetAngle1Rate(self, self.__angle, self.__force)

    @property
    def angle2Rate(self):
        if not self:
            raise Exception('sPhysicsJoint: this joint is not initialized')
        return _siberian.sPhysicsJointGetAngle2Rate(self)

    @angle2Rate.setter
    def angle2Rate(self, angle):
        if not self:
            raise Exception('sPhysicsJoint: this joint is not initialized')
        self.__angle2 = angle
        if not hasattr(self, '__force2'):
            self.__force2 = 0.0
        _siberian.sPhysicsJointSetAngle2Rate(
            self, self.__angle2, self.__force2)

    @property
    def angleForce(self):
        if not self:
            raise Exception('sPhysicsJoint: this joint is not initialized')
        return self.__force

    @angleForce.setter
    def angleForce(self, force):
        if not self:
            raise Exception('sPhysicsJoint: this joint is not initialized')
        self.__force = force
        if not hasattr(self, '__angle'):
            self.__angle = 0.0
        _siberian.sPhysicsJointSetAngle1Rate(self, self.__angle, self.__force)

    @property
    def angle2Force(self):
        if not self:
            raise Exception('sPhysicsJoint: this joint is not initialized')
        return self.__force2

    @angle2Force.setter
    def angle2Force(self, force):
        if not self:
            raise Exception('sPhysicsJoint: this joint is not initialized')
        if not hasattr(self, '__angle2'):
            self.__angle2 = 0.0
        self.__force2 = force
        _siberian.sPhysicsJointSetAngle2Rate(
            self, self.__angle2, self.__force2)


_siberian.sPhysicsCreateAnchor.argtypes = (
    POINTER(sObject), POINTER(sObject), dReal, dReal, laType, laType, c_bool)
_siberian.sPhysicsCreateAnchor.restype = sPhysicsJoint
sPhysicsCreateAnchor = lambda *args: _siberian.sPhysicsCreateAnchor(0, *args)

_siberian.sPhysicsJointGetAngle1.argtypes = (sPhysicsJoint,)
_siberian.sPhysicsJointGetAngle1.restype = c_double

_siberian.sPhysicsJointGetAngle2.argtypes = (sPhysicsJoint,)
_siberian.sPhysicsJointGetAngle2.restype = c_double

_siberian.sPhysicsJointGetAngle1Rate.argtypes = (sPhysicsJoint,)
_siberian.sPhysicsJointGetAngle1Rate.restype = c_double

_siberian.sPhysicsJointSetAngle1Rate.argtypes = (
    sPhysicsJoint, c_double, c_double)
_siberian.sPhysicsJointSetAngle1Rate.restype = None

_siberian.sPhysicsJointGetAngle2Rate.argtypes = (sPhysicsJoint,)
_siberian.sPhysicsJointGetAngle2Rate.restype = c_double

_siberian.sPhysicsJointSetAngle2Rate.argtypes = (
    sPhysicsJoint, c_double, c_double)
_siberian.sPhysicsJointSetAngle2Rate.restype = None

_siberian.sPhysicsJointGetAxisCount.argtypes = (sPhysicsJoint,)
_siberian.sPhysicsJointGetAxisCount.restype = c_int

_siberian.sBicycleAssemble.argtypes = (
    POINTER(sObject), POINTER(sObject), POINTER(sObject), POINTER(sObject))
_siberian.sBicycleAssemble.restype = None
sBicycleAssemble = _siberian.sBicycleAssemble


def sVehicle(scene, body,
             flw, frw, blw, brw,
             fls, frs, bls, brs, prefix=''):
    veh = sVehicle4Wheel()
    veh._body = pointer(body)
    veh._flw = pointer(flw)  # Переднее левое колесо
    veh._frw = pointer(frw)  # Переднее правое колесо
    veh._blw = pointer(blw)  # Переднее правое колесо
    veh._brw = pointer(brw)  # Заднее правое колесо
    veh._fls = pointer(fls)  # Передняя левая пружина
    veh._frs = pointer(frs)  # Передняя правая пружина
    veh._bls = pointer(bls)  # Задняя левая пружина
    veh._brs = pointer(brs)  # Задняя правая пружина

    _siberian.sVehicleInit(scene, veh, prefix.encode())
    return veh


class sRagdoll(Structure):
    _fields_ = [
        ('__head', POINTER(sObject)),
        ('__spine1', POINTER(sObject)),
        ('__spine2', POINTER(sObject)),
        ('__spine3', POINTER(sObject)),
        ('__lShoulder', POINTER(sObject)),
        ('__lForearm', POINTER(sObject)),
        ('__rShoulder', POINTER(sObject)),
        ('__rForearm', POINTER(sObject)),
        ('__lLeg', POINTER(sObject)),
        ('__lKnee', POINTER(sObject)),
        ('__rLeg', POINTER(sObject)),
        ('__rKnee', POINTER(sObject)),
        ('__lFoot', POINTER(sObject)),
        ('__rFoot', POINTER(sObject)),
        ('joints', sPhysicsJoint*16),
        ('__group', dJointGroupID)]

    def __init__(self, scene=None):
        Structure.__init__(self)
        self.__scene = scene

    @property
    def spine1_spine2_joint(self):
        return self.joints[0]

    @property
    def spine2_spine3_joint(self):
        return self.joints[1]

    @property
    def lforearm_lshoulder_joint(self):
        return self.joints[2]

    @property
    def rforearm_rshoulder_joint(self):
        return self.joints[3]

    @property
    def spine1_lshoulder_joint(self):
        return self.joints[4]

    @property
    def spine1_rshoulder_joint(self):
        return self.joints[5]

    @property
    def spine1_head_joint(self):
        return self.joints[6]

    @property
    def lknee_lleg_joint(self):
        return self.joints[7]

    @property
    def rknee_rleg_joint(self):
        return self.joints[8]

    @property
    def spine3_lleg_joint(self):
        return self.joints[9]

    @property
    def spine3_rleg_joint(self):
        return self.joints[10]

    @property
    def lfoot_lknee_joint(self):
        return self.joints[11]

    @property
    def rfoot_rknee_joint(self):
        return self.joints[12]

    @property
    def head(self):
        return getattr(self, '__head').contents if getattr(self, '__head') else None

    @head.setter
    def head(self, value):
        return setattr(self, '__head', pointer(value))

    @property
    def spine1(self):
        return getattr(self, '__spine1').contents if getattr(self, '__spine1') else None

    @spine1.setter
    def spine1(self, value):
        return setattr(self, '__spine1', pointer(value))

    @property
    def spine2(self):
        return getattr(self, '__spine2').contents if getattr(self, '__spine2') else None

    @spine2.setter
    def spine2(self, value):
        return setattr(self, '__spine2', pointer(value))

    @property
    def spine3(self):
        return getattr(self, '__spine3').contents if getattr(self, '__spine3') else None

    @spine3.setter
    def spine3(self, value):
        return setattr(self, '__spine3', pointer(value))

    @property
    def lShoulder(self):
        return getattr(self, '__lShoulder').contents if getattr(self, '__lShoulder') else None

    @lShoulder.setter
    def lShoulder(self, value):
        return setattr(self, '__lShoulder', pointer(value))

    @property
    def rShoulder(self):
        return getattr(self, '__rShoulder').contents if getattr(self, '__rShoulder') else None

    @rShoulder.setter
    def rShoulder(self, value):
        return setattr(self, '__rShoulder', pointer(value))

    @property
    def lForearm(self):
        return getattr(self, '__lForearm').contents if getattr(self, '__lForearm') else None

    @lForearm.setter
    def lForearm(self, value):
        return setattr(self, '__lForearm', pointer(value))

    @property
    def rForearm(self):
        return getattr(self, '__rForearm').contents if getattr(self, '__rForearm') else None

    @rForearm.setter
    def rForearm(self, value):
        return setattr(self, '__rForearm', pointer(value))

    @property
    def lLeg(self):
        return getattr(self, '__lLeg').contents if getattr(self, '__lLeg') else None

    @lLeg.setter
    def lLeg(self, value):
        return setattr(self, '__lLeg', pointer(value))

    @property
    def rLeg(self):
        return getattr(self, '__rLeg').contents if getattr(self, '__rLeg') else None

    @rLeg.setter
    def rLeg(self, value):
        return setattr(self, '__rLeg', pointer(value))

    @property
    def lKnee(self):
        return getattr(self, '__lKnee').contents if getattr(self, '__lKnee') else None

    @lKnee.setter
    def lKnee(self, value):
        return setattr(self, '__lKnee', pointer(value))

    @property
    def rKnee(self):
        return getattr(self, '__rKnee').contents if getattr(self, '__rKnee') else None

    @rKnee.setter
    def rKnee(self, value):
        return setattr(self, '__rKnee', pointer(value))

    @property
    def lFoot(self):
        return getattr(self, '__lFoot').contents if getattr(self, '__lFoot') else None

    @lFoot.setter
    def lFoot(self, value):
        return setattr(self, '__lFoot', pointer(value))

    @property
    def rFoot(self):
        return getattr(self, '__rFoot').contents if getattr(self, '__rFoot') else None

    @rFoot.setter
    def rFoot(self, value):
        return setattr(self, '__rFoot', pointer(value))

    def joinBodyParts(self, autodetect=0, prefix=""):
        if isinstance(prefix, str):
            prefix = prefix.encode()
        if (_siberian.sRagdollInit(self.__scene, self, autodetect, prefix)):
            raise Exception('sRagdoll: one of the spine parts is not set')


_siberian.sRagdollInit.argtypes = (
    POINTER(sScene), POINTER(sRagdoll), c_bool, c_char_p)
_siberian.sRagdollInit.restype = c_int

sPlayerInit = _siberian.sPlayerInit
sPlayerInit.argtypes = (POINTER(sScene), POINTER(sObject))
sPlayerSetImpact = _siberian.sPlayerSetImpact
sPlayerSetImpact.argtypes = (c_float, c_float, c_float)

sPlayerMouseLookOn = _siberian.sPlayerMouseLookOn
sPlayerMouseLookOn.argtypes = (POINTER(sScene),)
sPlayerMouseLookOn.restype = None

sPlayerMouseLookOff = _siberian.sPlayerMouseLookOff
sPlayerMouseLookOff.argtypes = (POINTER(sScene),)
sPlayerMouseLookOff.restype = None

sEngineStartOpenGL = _siberian.sEngineStartOpenGL
sEngineStartLoop = _siberian.sEngineStartLoop

#sStreamMJPEGOpen = _siberian.open_videostream
#sStreamMJPEGWrite = _siberian.stream_write_frame
#sStreamMJPEGClose = _siberian.close_videostream


def sPlayerSpeed(): return cast(
    _siberian.walk_speed_vector, POINTER(laType)).contents


def sPlayerStep(): return cast(_siberian.walk_step, POINTER(c_bool)).contents


class __sRender:
    def toggle(self, param):
        if param == 'Bloom':
            sRender.Bloom = 1 - sRender.Bloom

        if param == 'HDR':
            sRender.HDR = 1 - sRender.HDR

        if param == 'Reflections':
            sRender.Reflections = 1 - sRender.Reflections

        if param == 'SSGI':
            sRender.SSGI = 1 - sRender.SSGI

        if param == 'MotionBlur':
            sRender.MotionBlur = 1 - sRender.MotionBlur

    def setGLSLversion(self, version):
        if not isinstance(version, bytes):
            version = str(version).encode()
        if len(version) < 16:
            _siberian.sShaderSetVersion(version)

    @property
    def Bloom(self):
        return _siberian.sRenderGetBloom()

    @Bloom.setter
    def Bloom(self, value):
        _siberian.sRenderSetBloom(value)

    @property
    def HDR(self):
        return _siberian.sRenderGetHDR()

    @HDR.setter
    def HDR(self, value):
        _siberian.sRenderSetHDR(value)

    @property
    def Reflections(self):
        return _siberian.sRenderGetReflections()

    @Reflections.setter
    def Reflections(self, value):
        _siberian.sRenderSetReflections(value)

    @property
    def SSGI(self):
        return _siberian.sRenderGetSSGI()

    @SSGI.setter
    def SSGI(self, value):
        _siberian.sRenderSetSSGI(value)

    @property
    def MotionBlur(self):
        return _siberian.sRenderGetMotionBlur()

    @MotionBlur.setter
    def MotionBlur(self, value):
        _siberian.sRenderSetMotionBlur(value)

    def SwapPPShaders(self):
        _siberian.sRenderSwapPPShaders()


sRender = __sRender()

_siberian.sShaderSetVersion.argtypes = (c_void_p,)
_siberian.sShaderSetVersion.restype = None

_siberian.sRenderGetBloom.argtypes = ()
_siberian.sRenderGetBloom.restype = c_int

_siberian.sRenderSetBloom.argtypes = (c_int,)
_siberian.sRenderSetBloom.restype = None

_siberian.sRenderGetReflections.argtypes = ()
_siberian.sRenderGetReflections.restype = c_int

_siberian.sRenderSetReflections.argtypes = (c_int,)
_siberian.sRenderSetReflections.restype = None

_siberian.sRenderGetSSGI.argtypes = ()
_siberian.sRenderGetSSGI.restype = c_int

_siberian.sRenderSetSSGI.argtypes = (c_int,)
_siberian.sRenderSetSSGI.restype = None

_siberian.sRenderGetMotionBlur.argtypes = ()
_siberian.sRenderGetMotionBlur.restype = c_int

_siberian.sRenderSetMotionBlur.argtypes = (c_int,)
_siberian.sRenderSetMotionBlur.restype = None

_siberian.sRenderGetHDR.argtypes = ()
_siberian.sRenderGetHDR.restype = c_int

_siberian.sRenderSetHDR.argtypes = (c_int,)
_siberian.sRenderSetHDR.restype = None

_siberian.sRenderSwapPPShaders.argtypes = ()
_siberian.sRenderSwapPPShaders.restype = None

# sRenderDeferred(1)

if __name__ == '__main__':
    print((sObject, sizeof(sObject)))
    print((sCamera, sizeof(sCamera)))
    print((sShader, sizeof(sShader)))
    print((sTexture, sizeof(sTexture)))
    print((sMaterial, sizeof(sMaterial)))
    print((sMesh, sizeof(sMesh)))
    print((sPhysicsContact, sizeof(sPhysicsContact)))
    print((sPhysicsCS, sizeof(sPhysicsCS)))
    print((sPhysicsRS, sizeof(sPhysicsRS)))
    print((sScene, sizeof(sScene)))
    print((fElement, sizeof(fElement)))
    print((fForm, sizeof(fForm)))
