#coding: utf-8
from siberian_ctypes import *
from math import pi,sin,cos,radians
from random import choice
from struct import unpack
import sys

from scripts.itmes_handler import *
from scripts import flying_ball, vehicles

flashlight_trigger = False
flashlight_button = False
frames = 0;
update_timer = 0.0

def openDoor(obj):
    if not 'timer' in obj.attributes:
        return
    obj.attributes['timer'] += sGetFrameTime()
    if obj.attributes['timer'] <= 0.0:
        if obj.attributes['state']:
            obj.rotateGlobal(0.0, 0.0,-radians(45.0)*sGetFrameTime())
        else:
            obj.rotateGlobal(0.0, 0.0, radians(45.0)*sGetFrameTime())
    else:
        obj.attributes['state'] = not obj.attributes['state']
        del obj.attributes['timer']

kt = 0
marker = None
def placeBlock():
    global pl_ray,camera,kt,marker
    if not marker:
        return
    pos = Vector(pl_ray.transform_global.x, pl_ray.transform_global.y, pl_ray.transform_global.z)
    contacts = sorted(pl_ray.ray.contacts,key=lambda contact : abs(contact.hitPosition-pos))
    contactsCount = len(contacts)
    line = ''
    while contactsCount:
        contactsCount-=1
        if contacts[contactsCount].hitObject.name == 'oPlayer':
            contacts.remove(contacts[contactsCount])
            break
    
    for i in contacts:
        line += i.hitObject.name + '\n'
    #rayObjects.setText(line)
    marker.globalPosition = Vector(300.0, 300.0, 300.0)
    if contacts:
        cube = contacts[0].hitObject
        cubePos = Vector(cube.transform_global.x, cube.transform_global.y, cube.transform_global.z)
        contactPos = contacts[0].hitPosition
        side = contactPos-cubePos
        if abs(side.x)<0.49:
            side.x = 0
        if abs(side.y)<0.49:
            side.y = 0
        if abs(side.z)<0.49:
            side.z = 0
        marker.globalPosition = cubePos + side

        if sKeyboardGetKeyState(KEY_T)>kt:
            side *= 2.0
            newCube = cube.scene.addObject('oCube')
            newCubePos = Vector(newCube.transform_global.x, newCube.transform_global.y, newCube.transform_global.z)
            newCube.transform_global = laIdentity
            #ncp.x,ncp.y,ncp.z = contactPos.x, contactPos.y, contactPos.z
            newCube.globalPosition = cubePos + side
            rayObjects.setText(str(Vector(newCube.transform_global.x, newCube.transform_global.y, newCube.transform_global.z)))
    
    kt = sKeyboardGetKeyState(KEY_T)

def hud_script(sas):
    global frames,update_timer,pl_ray
    global HUD,hands,flashlight,player
    global flashlight_trigger, flashlight_button,rig
    
    sas.attributes['HUD'].process()
    if sKeyboardGetKeyState(KEY_F)==1:
        flashlight_trigger = not flashlight_trigger
        if flashlight_trigger:
            flashlight.color[3] = 0.0
        else:
            flashlight.color[3] = 2.0
  
    #sUI.print(*[i.name for i in pl_ray.collisionHitObjectsList()])
    
    if sKeyboardGetKeyState(KEY_G)==1:
        FluidInit(sas.scene, "obarrel_water_fluid", "barrel_water_fluid/shooting_range/barrel_water", 798)
    
    if sKeyboardGetKeyState(KEY_Y)==1:
        if not player['inveh']:
            for obj in pl_ray.collisionHitObjectsList():
                if obj.name.startswith('oveh'):
                    obj['controller'].setController()
                    sp = obj.getChildrenDict()['oveh_arm_jeep_1_sit_point']
                    player.suspendDynamics()
                    player.transform = laIdentity
                    player.rotateLocal(0.0,0.0,3.1415926535)
                    player.setParent(sp, 0)
                    player['inveh'] = 1
                    break
        else:
            player['inveh'] = 0
            player.transform = laIdentity
            player.rotateLocal(0.0,0.0,3.1415926535)
            player.removeParent()
            player.restoreDynamics()
  
    if sPlayerStep():
        try:
            if player.collider.contacts:
                mat_name = player.collider.contacts[0].hitObject.mesh.material.name
                step_sounds = []
                if mat_name.lower().startswith('brick') or mat_name.lower().startswith('concrete'):
                    step_sounds = ['data/sounds/foot_steps/concrete/step{}.wav'.format(i) for i in range(1,10)]
                if mat_name.lower().startswith('wood'):
                    step_sounds = ['data/sounds/foot_steps/wood/step{}.wav'.format(i) for i in range(1,5)]
                if mat_name.lower().startswith('gnd'):
                    step_sounds = ['data/sounds/foot_steps/ground/step_{}.wav'.format(i) for i in range(1,7)]
                if step_sounds:
                    player.attachSound(choice(step_sounds))
        except:# ValueError:
            pass
        #except Exception as e:
            #exc_info = sys.exc_info()
            #print("")
            #traceback.print_exception(*exc_info)
  
    if sKeyboardGetKeyState(KEY_Q):
        sPlayerMouseLookOff(sas.scene)
      
    if sKeyboardGetKeyState(KEY_E):
        sPlayerMouseLookOn(sas.scene)

def initLift(scene):
    lift = scene.getObject('olift')
    lift.collisionSensorOn()
    def beh(lift):
        li = lift.collisionHitObjectsList()
        if li:
            for obj in li:
                if obj.name=='oPlayer' and obj.transform_global.z<49:
                    lift.translateLocal(Vector(0.0,0.0,0.1))
    lift.setBehaviour(beh)
            
    
def FluidStep(obj):
    scene = obj.scene
    if obj['fluidFrame']>=obj['fluidFrames']:
        obj.removeBehaviour()
    obj['fluidFrame']+=1
    scene.removeMesh(obj.mesh.name)
    mesh = scene.loadMesh("%s.%03d"%(obj['fluidMeshName'], obj['fluidFrame']))
    obj.mesh = mesh
    obj.mesh.setMaterial(scene, "water")

def FluidInit(scene, objectName, meshName, frames):
    obj = scene.getObject(objectName)
    obj['material'] = obj.mesh._material
    mesh = scene.loadMesh("%s.%03d"%(meshName,1))
    obj.mesh = mesh
    obj.mesh.setMaterial(scene, "water")
    obj['fluidFrames'] = frames
    obj['fluidFrame'] = 1
    obj['fluidMeshName'] = meshName
    obj.setBehaviour(FluidStep)
    
def init(scene):
    #initLift(scene)
    global HUD,hands,flashlight,player,rig,pl_ray,camera,rayObjects,marker
    camera = scene.camera
    marker = scene.addObject('oblock_marker')
    
    hands = scene.getObject('smetarig')
    
    sPlayerInit(scene,hands)
    sPlayerMouseLookOn(scene)
    sMouseHide()
    return
    player = scene.getObject('oPlayer')
    player['inveh'] = 0
    
    player.radarSensorOn(1.0,3.1415926535/8)
    player.radar.dir = 0b010
    pl_ray = player.scene.getObject('oPlayerRay')
    pl_ray.setParent(scene.getObject('cCamera'), 1)
    pl_ray.ghost = 1
    pl_ray.collisionSensorOn()
    pl_ray.hide()
    #pl_ray.ray.dir = rayYn
    
    wpn = scene.getObject('owpn_ak12_hud')
    
    walls = scene.getObject('oinvisible_walls')
    if (hands and shootingPoint):
        scene.loadAction('data/mesh/ak12_shoot.anim','ak12_shoot')
        scene.loadAction('data/mesh/ak12_aim.anim','ak12_aim')
        scene.loadAction('data/mesh/ak12_aim_shot.anim','ak12_aim_shot')
        scene.loadAction('data/mesh/ak12_reload.anim','ak12_reload')
        scene.loadAction('data/mesh/SPAS12_All.anim','SPAS12_All')
        scene.loadAction('data/mesh/GlowStick.anim','GlowStick')
        scene.loadAction('data/mesh/ak12_hide.anim','ak12_hide')
        scene.loadAction('data/mesh/ak12_idle.anim','ak12_idle')
        
        HUD = HandsHUD(hands,shootingPoint,wpn)
        HUD.addItem('wpn_glow_stick')
        HUD.addItem('wpn_spas12')
        HUD.addItem('wpn_ak12')
        HUD.addItem('ammo_12g')
        HUD.addItem('ammo_12g')
        HUD.addItem('ammo_12g')
        HUD.addItem('ammo_12g')
        HUD.addItem('ammo_12g')
        HUD.addItem('ammo_12g')
        for i in range(10):
            HUD.addItem('ammo_5.45x39')
        hands.attributes['HUD'] = HUD
        hands.setBehaviour(hud_script)
        HUD.showItems()
        
    if walls:
        walls.mesh = sMeshNull
    flashlight = scene.getObject('lFlashlight')
    if flashlight:
        flashlight.setParent(scene.getObject("cCamera"))
        flashlight.transform = laIdentity
        flashlight.translateLocal(Vector(0.2,0.2,0.0))
        flashlight.color[3] = 0.0
        
    sSoundLoad('data/sounds/machinegunes/ak12_shot1.wav')
    sSoundLoad('data/sounds/machinegunes/ak12_shot2.wav')
    sSoundLoad('data/sounds/machinegunes/ak12_shot3.wav')
    sSoundLoad('data/sounds/machinegunes/ak12_reload.wav')
    sSoundLoad('data/sounds/machinegunes/ak12_show.wav')
    sSoundLoad('data/sounds/machinegunes/ak12_hide.wav')
    sSoundLoad('data/sounds/machinegunes/shoot_echo1.wav')
    for i in range(1,7):
        sSoundLoad('data/sounds/foot_steps/ground/step_{}.wav'.format(i))
        
    for i in range(1,10):
        sSoundLoad('data/sounds/foot_steps/concrete/step{}.wav'.format(i))
        
    for i in range(1,5):
        sSoundLoad('data/sounds/foot_steps/wood/step{}.wav'.format(i))
    for i in range(8):
        sSoundLoad('data/sounds/machinegunes/shoot_echo{}.wav'.format(i))
        
    for i in range(1,6):
        sSoundLoad('data/sounds/material/bullet_metal{}.wav'.format(i))
    for i in range(1,5):
        sSoundLoad('data/sounds/material/bullet_wood{}.wav'.format(i))
    
    for i in range(1,4):
        sSoundLoad('data/sounds/material/bullet_sand{}.wav'.format(i))
    for i in range(1,5):
        sSoundLoad('data/sounds/material/bullet_ground{}.wav'.format(i))
    
    # Сборка физической модели машины
    if (scene.getObject('oveh_volga_collider')):
        scene.getObject('oveh_volga_collider').mesh = sMeshNull
        sVehicle(scene,
                scene.getObject('oveh_volga_collider'),
                scene.getObject('ovolga_flw'),
                scene.getObject('ovolga_frw'),
                scene.getObject('ovolga_blw'),
                scene.getObject('ovolga_brw'),
                scene.getObject('ovolga_fls'),
                scene.getObject('ovolga_frs'),
                scene.getObject('ovolga_bls'),
                scene.getObject('ovolga_brs'))
    sPlayerMouseLookOn(scene)
    sMouseHide()
    return player
