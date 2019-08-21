bl_info = {
    'name': 'SGM Export',
    'category': 'All'
}

import bpy
from struct import *
import os
import sys
import platform
from subprocess import call
from mathutils import *
import random
from bpy_types import *
import threading
from imp import reload
#import siberian_ctypes as sgm
from time import *
import traceback

EXPORT_APPLY_MODIFIERS_RENDER = EXPORT_APPLY_MODIFIERS = True
MAX_BONES_BITS = (16)
MAX_BONES = (1<<MAX_BONES_BITS)
BONE_IW_MASK_SHIFT = (32-MAX_BONES_BITS)
BONE_IW_MASK = ((1<<BONE_IW_MASK_SHIFT)-1)

ACTION_PATH = 'mesh'
pjoin = os.path.join

def getTName(obj):
    types = {}
    types['MESH'] = 'o'
    types['LAMP'] = 'l'
    types['ARMATURE'] = 's'
    types['CAMERA'] = 'c'
    return types[obj.type] + obj.name


def getMeshModName(obj):
    if len(obj.modifiers)==1 and obj.modifiers[0].type=='ARMATURE':
        return obj.data.name
    if obj.modifiers:
        return obj.data.name + '_moded_by_' + obj.name
    else:
        return obj.data.name

ONLY_DEFORM_BONES = 2

def get_deform_parent(bone):
    if ONLY_DEFORM_BONES < 2:
        return bone.parent
    skel = bone['leke']
    if hasattr(bone, 'bone'):
        par = bone.parent
        while par:
            defname = 'DEF' + par.bone.name[3:]
            if defname in skel.pose.bones:
                if bone.name!=defname:
                    return skel.pose.bones[defname]
            elif par.bone.use_deform:
                return par
            par = par.parent
        return None
    else:
        par = bone.parent
        while par:
            defname = 'DEF' + par.name[3:]
            if defname in skel.data.bones:
                if bone.name!=defname:
                    return skel.data.bones[defname]
            elif par.use_deform:
                return par
            par = par.parent
        return None

def getBoneLocalPos(bone):
    parent = get_deform_parent(bone)
    if isinstance(bone, PoseBone):
        if parent:
            return parent.matrix.inverted() * bone.matrix
        else:
            return bone.matrix
    else:
        if parent:
            return parent.matrix_local.inverted() * bone.matrix_local
        else:
            return bone.matrix_local

def getDataBones(skel):
    bns = skel.data.bones[:]
    if not ONLY_DEFORM_BONES:
        return bns
    if ONLY_DEFORM_BONES==2:
        return [b for b in bns if b.use_deform]
    
    bones = []
    for b in bns:
        if b.use_deform:
            bones.append(b)
            continue
        for ch in b.children_recursive:
            if ch.use_deform:
                bones.append(b)
                break
    return bones

def getPoseBones(skel):
    bns = skel.data.bones[:]
    if not ONLY_DEFORM_BONES:
        return bns
    
    if ONLY_DEFORM_BONES==2:
        return [b for b in bns if b.use_deform]
    
    bones = []
    for b in bns:
        if b.bone.use_deform:
            bones.append(b)
            continue
        for ch in b.children_recursive:
            if ch.bone.use_deform:
                bones.append(b)
                break
    return bones

def matrix2bytes(mat):
    result = b''
    for i in mat:
        for num in i:
            result += pack('<f', num)
    return result

def mesh_triangulate(me):
    import bmesh
    bm = bmesh.new()
    bm.from_mesh(me)
    bmesh.ops.triangulate(bm, faces=bm.faces)
    bm.to_mesh(me)
    bm.free()

def save_mesh(ob, scene, custom_name=None):
    directory = 'mesh/'
    mesh_name = getMeshModName(ob)
    checksum = hashMesh(ob.data)
    try:
        file = open(directory + mesh_name+'.mesh', 'rb')
        f_checksum = file.read(8)
        file.close()
        if f_checksum == pack('<d',checksum):
            return
    except:
        pass
    bones = None
    skeleton = None
    modifier = None
    for mod in ob.modifiers:
        if mod.type == 'ARMATURE' and mod.object:
            modifier = mod
            skeleton = mod.object
            pose_position = skeleton.data.pose_position
            #skeleton.data.pose_position = 'REST'
            bns = skeleton.data.bones
            bones = getDataBones(skeleton)
            groups = []
            for i in ob.vertex_groups:
                if i.name in bns and bns[i.name] in bones:
                    groups.append(bones.index(bns[i.name]) )
                else:
                    groups.append(-1)
            break
        
    if modifier:
        show_render = modifier.show_render
        modifier.show_render = False
    
    if custom_name is not None:
        mesh_name = custom_name
    
    me = ob.to_mesh(scene, EXPORT_APPLY_MODIFIERS, calc_tessface=False,
                                                settings='RENDER')
    
    mesh_triangulate(me)
    if not len(me.uv_textures):
        me.uv_textures.new()
    me.calc_tangents()

    skin = bool(skeleton)
    vertices = me.vertices[:]
    vert_indices = []
    vert_set = {}
    vert_count = 0
    vertices_data = []

    texture_uv = None
    lightmap_uv = None
    for i in me.uv_layers:
        if i.name != 'shadow' and texture_uv is None:
            texture_uv = i
        if i.name == 'shadow' and lightmap_uv is None:
            lightmap_uv = i
        if texture_uv is not None and lightmap_uv is not None:
            break

    pols = list(me.polygons)
    cnt = 0
    wm = bpy.context.window_manager
    wm.progress_begin(0, len(pols))
    
    for face in pols:
        if (cnt%1000) == 0:
            wm.progress_update(cnt)
        cnt+=1
        li = [me.loops[i] for i in face.loop_indices]
        for vert in li:
            vertex = vertices[vert.vertex_index]
            vert_data = tuple()
            weights = [0 for i in range(3)]
            if skin and len(vertex.groups):
                weights = [[groups[g.group], g.weight] for g in vertex.groups if groups[g.group]>-1]
                weights.sort(key = lambda val : val[1], reverse=True)
                while len(weights)<3:
                    weights.append([0,0])
                weights = weights[:3]
                s = sum([weights[0][1], weights[1][1], weights[2][1]])
                for i in range(3):
                    if s != 0.0:
                        weights[i][1] /= s
                    weights[i] = (weights[i][0]<<16) | int(weights[i][1]*0xFFFF)
            
            UV = texture_uv.data[vert.index].uv
            
            vert_data  = tuple(vertex.co)
            vert_data += tuple(vert.normal)
            vert_data += tuple(UV)
            vert_data += tuple(vert.bitangent)
            vert_data += tuple(vert.tangent)
            
            if skin:
                vert_data += tuple(weights[:3])
            if lightmap_uv:
                vert_data += tuple(lightmap_uv.data[vert.index].uv)

            if not vert_data in vert_set:
                vert_set[vert_data] = vert_count
                vert_indices.append(vert_count)
                vertices_data.append(vert_data)
                vert_count += 1
            else:
                vert_indices.append(vert_set[vert_data])
                
    wm.progress_end()
    if mesh_name.rfind('/')>-1:
        directory += mesh_name[:mesh_name.rfind('/')+1]
        if not os.path.isdir(directory):
            os.makedirs(directory)
        mesh_name = mesh_name[mesh_name.rfind('/')+1:]
        
    file = open(directory + mesh_name+'.mesh', 'wb')

    file.write(pack('<d', checksum))
    file.write(pack('<?', bool(skin)))
    file.write(pack('<?', bool(lightmap_uv)))
    file.write(pack('<I', len(vert_indices)))
    file.write(pack('<{}I'.format(len(vert_indices)), *vert_indices))
    file.write(pack('<I', vert_count))
    for i in vertices_data:
        format = '<3f3f2f3f3f'
        if skin:
            format += '3I'
        if lightmap_uv:
            format += '2f'
        data = pack(format, *i)
        file.write(data)
    
    if skin:
        print('Mesh with',len(bones), 'links')
        file.write(pack('<I', len(bones)))
        for b in bones:
            link_matrix = b.matrix_local
            file.write(matrix2bytes(link_matrix))
    if modifier:
        modifier.show_render = show_render
    file.close()
    try:
        bpy.data.meshes.remove(me)
    except:
        pass

def meshesArray(scene):
    meshes = set()
    for i in scene.objects:
        if i.type == 'MESH':
            if [1 for p in scene['Ignore prefixes'] if i.name.startswith(p)]:
                continue
            mesh = getMeshModName(i).encode() + b'\n'
            if i.data.materials and i.data.materials[0]:
                mesh += i.data.materials[0].name.encode() + b'\n'
            else:
                mesh += b'default\n'
            meshes.add(mesh)
    return pack('<I', len(meshes)) + b''.join(meshes)

def texturesArray():
    textures = pack('<I', len(bpy.data.textures))
    for i in bpy.data.textures:
        img = i.image
        filepath = img.filepath[:img.filepath.rfind('.')].replace('//','') + '\n'
        textures += i.name.encode() + b'\n'
        textures += filepath.encode()
        fname = img.filepath.replace('//','')
        try:
            print(i.name, fname[:fname.rfind('.')] + '.dds')
            open(fname[:fname.rfind('.')] + '.dds', 'rb').close()
        except:
            call('nvcompress -bc1 ' + fname, shell=True)
            call('nvdxt -dxt1 -file ' + fname, shell=True)
    return textures

def materialsArray():
    materials = pack('<I', len(bpy.data.materials) + 1)
    materials+= b'default\n'  # Имя материала
    materials+= b'\n'         # Имя текстуры
    materials+= b'\n'         # Имя зеркальной текстуры
    materials+= b'\n'         # Имя карты рельефа
    materials+= b'\n'         # Имя текстуры освещения

    materials+= pack('<3f', 0.5,0.5,0.5)    # Diffuse reflection
    materials+= pack('<3f', 0.0,0.0,0.0)    # Specular reflection
    materials+= pack('<f',  0.0)      # Свечение
    materials+= pack('<f',  0.0)      # Прозрачность
    materials+= pack('<d',  1.0)      # Сила трения
    
    for mat in bpy.data.materials:
        print(mat.name)
        material = b''
        dtex  = b''
        stex  = b''
        ntex  = b''
        lmtex = b''
        diffuse_color = mat.diffuse_color * mat.diffuse_intensity
        specular_color = mat.specular_color * mat.specular_intensity
        for tex in mat.texture_slots:
            if tex is not None:
                if tex.use_map_color_diffuse:
                    dtex = tex.name.encode()
                if tex.use_map_normal:
                    ntex = tex.name.encode()
                if tex.use_map_color_spec:
                    stex = tex.name.encode()
                if tex.use_map_ambient:
                    lmtex = tex.name.encode()
        
        material += mat.name.encode() + b'\n'
        material += dtex + b'\n'
        material += stex + b'\n'
        material += ntex + b'\n'
        material += lmtex + b'\n'
        material += pack("<3f", *diffuse_color)
        material += pack("<3f", *specular_color)
        material += pack("<f",   mat.emit)
        material += pack("<f",   (1-mat.alpha) * mat.use_transparency)
        material += pack("<d",   mat.physics.friction)
        
        materials += material
    return materials

def lightsArray(scene):
    types = {}
    types['POINT']  = 0
    types['SUN']    = 1
    types['SPOT']   = 2
    types['HEMI']   = None
    types['SPAREA'] = None
    lights_count = 0
    lights = b''
    for i in scene.objects:
        if (i.type == 'LAMP' or i.type == 'LIGHT') and types[i.data.type] is not None:
            if [1 for p in scene['Ignore prefixes'] if i.name.startswith(p)]:
                continue
            print(i.name)
            l = i.data
            parent_name = '' if i.parent is None else getTName(i.parent)
            light  = getTName(i).encode() + b'\n'
            light += pack('?', i.hide_render)
            light += parent_name.encode() + b'\n'
            light += matrix2bytes(i.matrix_local)
            light += pack('<4f', *(l.color[:] + (l.energy,)))
            light += pack('b', types[l.type])
            light += pack('?', l.use_shadow)
            if hasattr(l,'spot_size'):
                light += pack('<f', l.spot_size)
                light += pack('<f', l.spot_size * l.spot_blend)
            else:
                light += pack('<f', 0.0)
                light += pack('<f', 0.0)
            light += pack('<f', l.shadow_buffer_clip_start)
            light += pack('<f', l.shadow_buffer_clip_end)
            
            lights += light
            lights_count += 1
    return pack('<I', lights_count) + lights

def objectsArray(scene, save_meshes):
    meshes_set = set()
    
    colliderTypes = {}
    colliderTypes['CAPSULE'] = 0
    colliderTypes['BOX'] = 1
    colliderTypes['SPHERE'] = 2
    colliderTypes['CYLINDER'] = 3
    colliderTypes['TRIANGLE_MESH'] = 4
    colliderTypes['CONVEX_HULL'] = 5
    colliderTypes['CONE'] = 6
    
    dynamicTypes = {}
    dynamicTypes['NO_COLLISION'] = 0
    dynamicTypes['NAVMESH'] = 0
    dynamicTypes['SENSOR'] = 0
    dynamicTypes['OCCLUDER'] = 0
    dynamicTypes['STATIC'] = 1
    dynamicTypes['RIGID_BODY'] = 2
    dynamicTypes['SOFT_BODY'] = 2
    dynamicTypes['DYNAMIC'] = 3
    dynamicTypes['CHARACTER'] = 3
    objects = b''
    obj_count = 0
    for i in scene.objects:
        if i.type == 'MESH':
            if [1 for p in scene['Ignore prefixes'] if i.name.startswith(p)]:
                continue
            print(i.name)
            parent_name = '' if i.parent is None else getTName(i.parent)
            mesh_name = getMeshModName(i)
            if mesh_name not in meshes_set:
                meshes_set.add(mesh_name)
                if save_meshes:
                    if not 'fluid' in i:
                        save_mesh(i, scene)
                    else:
                        old_frame = scene.frame_current
                        for fr in range(i['fluid']):
                            scene.frame_set(fr)
                            save_mesh(i, scene, i.name+'/frame_%03d'%(fr,))
                        scene.frame_current = old_frame
            
            obj = getTName(i).encode() + b'\n'
            obj += pack('?', i.hide_render)
            obj += parent_name.encode() + b'\n'
            obj += mesh_name.encode() + b'\n'
            obj += matrix2bytes(i.matrix_local)
            obj += pack('<I', dynamicTypes[i.game.physics_type])
            obj += pack('<I', colliderTypes[i.game.collision_bounds_type] if i.game.use_collision_bounds else 4)
            obj += pack('<f', i.game.mass)
            for mod in i.modifiers:
                if mod.type == 'ARMATURE' and mod.object:
                   obj += getTName(mod.object).encode() + b'\n'
                   break
            obj_count += 1
            objects += obj
            
    return pack("<I", obj_count) + objects

def setSkeletonDeformFlags(skel):
    groups = set()
    for obj in skel.children:
        if obj.type == 'MESH':
            for vg in obj.vertex_groups:
                groups.add(vg.name)
    for b in skel.data.bones:
        b.use_deform = (b.name in groups)# or (not b.name.startswith('ORG') and not b.name.startswith('MCH') and not b.name.startswith('DEF'))

def skeletonsArray(scene):
    skel_count = 0
    skeletons = b''
    for i in scene.objects:
        if i.type != 'ARMATURE': continue
        if [1 for p in scene['Ignore prefixes'] if i.name.startswith(p)]:
            continue
        skel = i
        setSkeletonDeformFlags(skel)
        parent_name = '' if i.parent is None else getTName(i.parent)
        pose_position = i.data.pose_position
        i.data.pose_position = 'REST'
        bones = getDataBones(i) #[b for b in i.data.bones if (b.use_deform or not ONLY_DEFORM_BONES)]
        
        print(i.name,'with',len(bones),'bones')
        
        skeleton  = getTName(i).encode() + b'\n'
        skeleton += pack('?', i.hide_render)
        skeleton += parent_name.encode() + b'\n'
        skeleton += matrix2bytes(i.matrix_local)
        skeleton += pack('<I', len(bones))
        bone_dict = {}
        
        for b in range(len(bones)):
            bone_dict[bones[b].name] = b
        num = 0
        for b in bones:
            bone = b'b' + b.name.encode() + b'\n'
            parent = get_deform_parent(b)
            if parent:
                pnum = bone_dict[parent.name]
                bone += pack('<I', bone_dict[parent.name])
                bone += pack('<f', 1.0)
                bone += matrix2bytes(getBoneLocalPos(b))
            else:
                pnum = -1
                bone += pack('<I', 0xFFFFFFFF)
                bone += pack('<f', 1.0)
                bone += matrix2bytes(getBoneLocalPos(b))
            skeleton += bone
            num += 1
            
        for b in skel.data.bones:
            del b['leke']
        for b in skel.pose.bones:
            del b['leke']
        
        skel_count += 1
        skeletons += skeleton
        i.data.pose_position = pose_position
    return pack('<I', skel_count) + skeletons

def camera(scene):
    aspect = scene.render.resolution_y / scene.render.resolution_x
    cam = scene.camera
    cname  = b'c' + cam.name.encode() + b'\n'
    pname  = (('' if cam.parent is None else getTName(cam.parent)) + '\n').encode()
    cmat   = matrix2bytes(cam.matrix_local)
    cangle = pack('<f', cam.data.angle*180/3.1415926535 * aspect)
    
    print(cam.name, cam.data.angle*180/3.1415926535 * aspect)
    
    return cname + pname + cmat + cangle

def save_action(scene, skel, act_name, prefix=""):
    print('Action',act_name,'for',skel.name)
    hash = hashAction(bpy.data.actions[act_name])
    hash = pack('<d', hash)
    
    try:
        file = open(pjoin(ACTION_PATH, prefix + act_name + '.anim'), 'rb')
        h = file.read(8) 
        file.close()
        if h==hash:
            #print(act_name,'has not changed')
            return
    except:
        pass
    file = open(pjoin(ACTION_PATH, prefix + act_name + '.anim'), 'wb')
    pose = skel.data.pose_position
    skel.data.pose_position = "POSE"
            
    act = bpy.data.actions[act_name]
    
    if not skel.animation_data:
        skel.animation_data_create.create()
    
    old_act = skel.animation_data.action
    old_frame = scene.frame_current
    
    skel.animation_data.action = act
    start, end = act.frame_range
    
    bbones = getDataBones(skel)
    bones  = [skel.pose.bones[i.name] for i in bbones]
    
    data  = hash
    data += pack('<I', len(bones))
    data += pack('<I', round(end - start))
    
    bones_frames = {}
    for b in bones:
        bones_frames[b.name] = b''
    
    for frame in range(int(start), int(end)):
        scene.frame_set(frame)
        for b in bones:
            bones_frames[b.name] += matrix2bytes(getBoneLocalPos(b))
    
    for b in bones:
        data += b'b' + b.name.encode() + b'\n'
        data += bones_frames[b.name]
    skel.data.pose_position = pose
    skel.animation_data.action = old_act
    scene.frame_current = old_frame
    file.write(data)
    file.close()

def actionsArray(scene):
    actions = b''
    count = 0
    for skel in scene.objects:
        if skel.type == 'ARMATURE':
            for b in skel.data.bones:
                b['leke'] = skel
            for b in skel.pose.bones:
                b['leke'] = skel
            
            bonesNames = {i.name for i in getDataBones(skel)}
            allBonesNames = {i.name for i in skel.data.bones}
            for act in bpy.data.actions:
                actBonesNames = {i.name for i in act.groups}
                crossing = actBonesNames&bonesNames
                if actBonesNames&allBonesNames:
                    #print(skel.name,act.name,len(crossing))
                    if actBonesNames&allBonesNames==actBonesNames:
                        prefix = ""
                    else:
                        prefix = skel.name + "_"
                    actions += (prefix + act.name).encode() + b'\n'
                    save_action(scene, skel, act.name, prefix)
                    count += 1
                
    return pack('<I', count) + actions

def export():
    DATA_PATH = bpy.path.abspath('//')
    os.chdir(DATA_PATH)

    scene = bpy.context.scene
    join_path = os.path.join

    dir = ''
    t1 = time()
    print('Export scene')
    file = open(join_path(dir, 'scenes', scene.name + '.bin'), 'wb')

    print('=== Camera ======')
    file.write(camera(scene))
    print('=================\n')

    print('=== Textures ====')
    file.write(texturesArray())
    print('=================\n')

    print('=== Materials ===')
    file.write(materialsArray())
    print('=================\n')

    #print('=== Meshes ======')
    file.write(meshesArray(scene))

    print('=== Actions =====')
    ta = time()
    file.write(actionsArray(scene))
    print(time() - ta,'s.')
    print('=================\n')

    print('=== Lights ======')
    file.write(lightsArray(scene))
    print('=================\n')

    print('=== Skeletons ===')
    file.write(skeletonsArray(scene))
    print('=================\n')

    print('=== Objects =====')
    file.write(objectsArray(scene, 1))
    print('=================\n')
    file.close()
    
    print(time() - t1, 'seconds')
    
    os.chdir(r'../')

def run_sgm(scene):
    global script_loader
    width = scene.game_settings.resolution_x
    htight = scene.game_settings.resolution_y
    scene_name = scene.name
    
    sgm.sEngineCreateWindow(width, htight, 0)
    sgm.sEngineSetSwapInterval(1)
    sgm.sSoundInit()

    sgm.sRender.MotionBlur = 0
    sgm.sRender.SSGI = 1
    sgm.sRender.MotionBlur = 1
    sgm.sRender.HDR = 1
    sgm.sRender.Bloom = 1

    sgm.sEngineStartOpenGL()

    sgm.sUI.init(640, 480)

    skybox = sgm.sTexture()
    skybox.loadCubemap("data/textures/cubemap/field.dds")

    scene = sgm.sScene(filename = scene_name)
    sgm.sEngineSetActiveScene(scene)
    scene.setSkyTexture(skybox)
    scene.setScript(sgm.executeAll)
    
    #ProfilingTimers.show()
    
    sgm.sEngineShowFPS()
    
    try:
        import script_loader
        reload(script_loader)
        script_loader.main(scene)
    except Exception as e:
        exc_info = sys.exc_info()
        print("")
        print("".join(traceback.format_exception(*exc_info)))
    
    sgm.sEngineStartLoop()
    sgm.sSoundCloseDevice()
    sgm.sEngineClearScripts()
    exit(0)

def run():
    scene = bpy.context.scene
    export()
    if platform.system()=='Windows':
        call('python SGM.py', shell=True)
    else:
        call('./Game -hdr -ssgi -ssr', shell=True)
    #reload(sgm)
    
    #t = threading.Thread(target=run_sgm, args=(scene,))
    
    #t.start()
    #t.join()
    
class SGM_Export(bpy.types.Operator):
    bl_idname = 'export_scene.sgm_export'
    bl_label = 'Export to SGM'
    bl_options = {"REGISTER", "UNDO"}
 
    def execute(self, context):
        run()
        return {"FINISHED"}
 
def register() :
    bpy.utils.register_class(SGM_Export)
 
def unregister() :
    bpy.utils.unregister_class(SGM_Export)

def hashAction(act):
    result = 0
    for fc in act.fcurves:
        for kp in fc.keyframe_points:
            result += kp.period - kp.back - kp.amplitude - kp.co.magnitude
    return result

def hashMesh(mesh):
    result = 0
    for i in mesh.vertices:
        vec = i.co
        nor = i.normal
        result += vec.x - vec.y + vec.z
        result += nor.x - nor.y + nor.z
    if len(mesh.uv_layers):
        for i in mesh.uv_layers[0].data:
            result += i.uv.x - i.uv.y
    if len(mesh.uv_layers)>1:
        for i in mesh.uv_layers[1].data:
            result += i.uv.x - i.uv.y
    return result

def ph(obj):
    print('\n'.join(dir(obj)))

if __name__ == '__main__':
    run()
