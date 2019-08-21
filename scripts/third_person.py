from siberian_ctypes import *
from math import *
from . import vehicles

SITING, STANDING, WALKING, RUNNING, SPRINTING = POSES = 'sit', 'stand', 'walk', 'run', 'sprint'

class CharacterController:
    __SITING, __STANDING, __WALKING, __RUNNING, __SPRINTING = __POSES = 'sit', 'stand', 'walk', 'run', 'sprint'
    __ACTION_LAYER_1 = 0
    __MOVING_LAYER_1 = 1
    __ACTION_LAYER_2 = 2
    __MOVING_LAYER_2 = 3
    def __init__(self,skel,name,camera=None):
        __ACTION_LAYER_1 = 0
        __MOVING_LAYER_1 = 1
        __ACTION_LAYER_2 = 2
        __MOVING_LAYER_2 = 3
        self.__camera = camera
        self.__skeleton = skel
        self.__scene = skel.scene
        
        self.__rotationX = 0.0
        self.__rotationY = 0.0
        
        self.__walkSpeed = Vector(0,0,0)
        self.__collider = sCharacterInit(self.__scene, skel, 'o' + str(name))
        
        self.__collider.attributes = skel.attributes
        self.__actionParams = skel.attributes
        
        self.__loopTimer = 0.0
        
        skel.resetPose()
        skel.mixPoseFromActionWithPose(self.__actionParams['hands_action'], 0, 0.0, 1.0)
        skel.playAction(self.__actionParams['hands_action'], __ACTION_LAYER_1, ACTION_ADD_LOOP,0.0,1.0,0.0)
        skel.playAction(self.__actionParams['hands_action'], __ACTION_LAYER_2, ACTION_ADD_LOOP,0.0,1.0,0.0)
        skel.playAction(self.__actionParams['legs_action'],  __MOVING_LAYER_1, ACTION_ADD_LOOP,0.0,1.0,0.0)
        skel.playAction(self.__actionParams['legs_action'],  __MOVING_LAYER_2, ACTION_ADD_LOOP,0.0,1.0,0.0)
        
        c = self.__actionParams['head_body_rot_prop']
        
        self.__activePose = CharacterController.__STANDING
    
    @staticmethod
    def __3rdpw(cam):
        # Получение скорости курсора мыши
        cpos  = sMouseGetPosition()
        delta = cpos - Vector(320, 240, 0)
        cam['rotInert'] += delta*-0.0005
        cam['rot'] += cam['rotInert']
        
        if cam['rot'].y < 0.5:
            cam['rot'].y = 0.5
        elif cam['rot'].y > pi-0.7:
            cam['rot'].y = pi-0.7
        
        cam['rotInert'] *= pow(0.75, sGetFrameTime()*60.0)
        sMouseSetPosition(320, 240)
        
        # Управление расстоянием между камерой и целью
        cam['distance'] += sMouseGetVerticalScroll()
        if cam['distance']>10:
            cam['distance'] = 10
        elif cam['distance']<-10:
            cam['distance'] = -10
        
        # Перемещение и поворот камеры
        cam.setRotation(cam['rot'].y, 0.0, 0.0)
        cam.rotateGlobal(0.0, 0.0, cam['rot'].x)
        
        player = cam['target']
            
        dist = pow(1.1, cam['distance'])
        vect = Vector(cam.transform_global.kx, cam.transform_global.ky, cam.transform_global.kz)
        side = Vector(cam.transform_global.ix, cam.transform_global.iy, cam.transform_global.iz)
        cam.globalPosition = player.globalPosition + (vect + side*0.4)*dist + Vector(0.0, 0.0, 0.75)
    
    def __animation(self):
        speed = abs(self.__walkSpeed)
        skel = self.__skeleton
        # Поворот персонажа
        angle_x = self.__rotationX
        angle_y = self.__rotationY
        c = skel['head_body_rot_prop']
        
        skel.mixPoseFromLayerWithPose(self.__MOVING_LAYER_1, 0.0, 1.0)
        
        frame = remap(angle_x, 0.5*pi, -0.5*pi, skel['body_rot_left'], skel['body_rot_right'])
        skel.addPoseFromLayerToPose(self.__MOVING_LAYER_1, time=frame, weight=c)
        
        frame = remap(angle_x, 0.5*pi, -0.5*pi, skel['head_rot_left'], skel['head_rot_right'])
        skel.addPoseFromLayerToPose(self.__MOVING_LAYER_1, time=frame, weight=1.0-c)
        
        frame = remap(angle_y, -0.5*pi, 0.5*pi, skel['body_rot_up'], skel['body_rot_down'])
        skel.addPoseFromLayerToPose(self.__MOVING_LAYER_1, time=frame, weight=c)
        
        frame = remap(angle_y, -0.5*pi, 0.5*pi, skel['head_rot_up'], skel['head_rot_down'])
        skel.addPoseFromLayerToPose(self.__MOVING_LAYER_1, time=frame, weight=1.0-c)
        
        # Анимация хотьбы/бега
        x0 = skel['walk_speed']; y0 = 1.0 / skel['legs_walking_period']
        x1 = skel['run_speed'];  y1 = 1.0 / skel['legs_running_period']
        k  = (y1 - y0) / (x1 - x0)
        b  = -k*x0 + y0
        period = 1.0 / (speed*k + b)
        self.__loopTimer = (self.__loopTimer + (speed*k + b) * sGetFrameTime())%1.0
        
        stand_walk = speed>0 and speed<skel['walk_speed']
        walk_run   = speed>=skel['walk_speed']
        if stand_walk:
            weight = speed / skel['walk_speed']
            skel.setActionInterval(self.__ACTION_LAYER_1, skel['hands_standing_start'], skel['hands_standing_end'])
            skel.setActionInterval(self.__MOVING_LAYER_1, 0.0, 0.0001)
            skel.setActionInterval(self.__ACTION_LAYER_2, skel['hands_walking_start'], skel['hands_walking_end'])
            skel.setActionInterval(self.__MOVING_LAYER_2, skel['legs_walking_start'], skel['legs_walking_end'])
            
            skel.setLayerWeight(self.__MOVING_LAYER_1, 1.0-weight)
            skel.setLayerWeight(self.__ACTION_LAYER_1, 1.0-weight)
            skel.setLayerWeight(self.__MOVING_LAYER_2, weight)
            skel.setLayerWeight(self.__ACTION_LAYER_2, weight)
            
            skel.setActionSpeed(self.__MOVING_LAYER_1, 0.0)
            skel.setActionSpeed(self.__ACTION_LAYER_1, 0.03)
            
            skel.setActionFrame2(self.__MOVING_LAYER_2, self.__loopTimer)
            skel.setActionFrame2(self.__ACTION_LAYER_2, self.__loopTimer)
            
        if walk_run:
            weight = (speed - skel['walk_speed']) / (skel['run_speed'] - skel['walk_speed'])
            #sUI.print("weight",weight)
            skel.setActionInterval(self.__ACTION_LAYER_1, skel['hands_walking_start'], skel['hands_walking_end'])
            skel.setActionInterval(self.__MOVING_LAYER_1, skel['legs_walking_start'], skel['legs_walking_end'])
            skel.setActionInterval(self.__ACTION_LAYER_2, skel['hands_running_start'], skel['hands_running_end'])
            skel.setActionInterval(self.__MOVING_LAYER_2, skel['legs_running_start'], skel['legs_running_end'])
            skel.setLayerWeight(self.__MOVING_LAYER_1, 1.0-weight)
            skel.setLayerWeight(self.__ACTION_LAYER_1, 1.0-weight)
            skel.setLayerWeight(self.__MOVING_LAYER_2, weight)
            skel.setLayerWeight(self.__ACTION_LAYER_2, weight)
            ltp = (self.__loopTimer + 0.5)%1.0
            skel.setActionFrame2(self.__MOVING_LAYER_1, self.__loopTimer)
            skel.setActionFrame2(self.__ACTION_LAYER_1, self.__loopTimer)
            skel.setActionFrame2(self.__MOVING_LAYER_2, ltp)
            skel.setActionFrame2(self.__ACTION_LAYER_2, ltp)
    
    @staticmethod
    def __handling(player):
        controller = player['controller']
        skel = controller.__skeleton
        cam = controller.__camera
        walkVector = Vector(0,0,0)
        walkVector -= Vector(cam.transform_global.kx, cam.transform_global.ky, cam.transform_global.kz) * sKeyboardGetKeyState(KEY_W)
        walkVector += Vector(cam.transform_global.kx, cam.transform_global.ky, cam.transform_global.kz) * sKeyboardGetKeyState(KEY_S)
        
        walkVector -= Vector(cam.transform_global.ix, cam.transform_global.iy, cam.transform_global.iz) * sKeyboardGetKeyState(KEY_A)
        walkVector += Vector(cam.transform_global.ix, cam.transform_global.iy, cam.transform_global.iz) * sKeyboardGetKeyState(KEY_D)
        walkVector.z = 0.0
        walkVector.normalize()
        
        if abs(walkVector):
            if sKeyboardGetKeyState(KEY_LEFT_SHIFT):
                max_speed = skel['run_speed']
                controller.__state = 'RUNNING'
            else:
                max_speed = skel['walk_speed']
                controller.__state = 'WALKING'
        else:
            max_speed = 0
            controller.__state = 'STANDING'
        
        walkVector *= max_speed
        
        ft = sGetFrameTime()
        accel = 5.0
        
        dv = walkVector - controller.__walkSpeed
        l = abs(dv)
        dv.normalize()
        dv *= sGetFrameTime() * accel
        dv *= (1.0 - exp(-(l**2.0)))**0.5
        controller.__walkSpeed += dv
        
        pAngle = pi-atan2(controller.__walkSpeed.x, controller.__walkSpeed.y)
        cAngle = -atan2(cam.transform_global.kx, cam.transform_global.ky)
        
        player.setRotation(0.0, 0.0, pAngle)
        player.setSpeedGlobal(controller.__walkSpeed, X_AXIS|Y_AXIS)
        
        targetBodyAngle = clamp((cAngle-pAngle+pi)%(pi*2.0)-pi, -0.5*pi, 0.5*pi)
        cam['controller'].rotationX += (targetBodyAngle - cam['controller'].rotationX)*0.2
        cam['controller'].rotationY  = atan2(cam.transform_global.kz, cam.transform_global.jz)
        controller.__animation()
        CharacterController.__3rdpw(cam)
    
    @property
    def camera(self):
        return self.__camera
    
    @camera.setter
    def camera(self, cam):
        if isinstance(cam,sObject):
            self.__camera = cam
            cam['rot']  = Vector(0,0,0)
            cam['mppos'] = sMouseGetPosition()
            cam['target'] = self.__collider
            cam['rotInert'] = Vector(0,0,0)
            cam['distance'] = 1.0
            cam['controller'] = self
            #cam.setBehaviour(CharacterController.__3rdpw)
            self.__collider['rotation'] = 0.0
            self.__collider['velocity'] = Vector(0.0, 0.0, 0.0)
            self.__collider['controller'] = self
            self.__collider.setBehaviour(CharacterController.__handling)
    
    @property
    def rotationX(self):
        return self.__rotationX
    @rotationX.setter
    def rotationX(self, angle):
        self.__rotationX = angle
    
    @property
    def rotationY(self):
        return self.__rotationY
    @rotationY.setter
    def rotationY(self, angle):
        self.__rotationY = angle
    
    def setPose(self, pose):
        print(self.__actions['name'], pose)

def cam_fly(camera):
    sMouseShow()
    global __skeleton
    camRes = camera.scene.camera.width, camera.scene.camera.height
    if not 'prev_pos' in camera.attributes:
        camera['prev_pos'] = sMouseGetPosition()
        camera['inertia'] = Vector(0,0,0)
        camera['rotation'] = Vector(0,0,0)
        camera['distance'] = 2.0
        camera['target'] = Vector(0.0, 0.0, 0.0)
        
    delta = camera['prev_pos']-sMouseGetPosition()
    
    if sMouseGetVerticalScroll()>0:
        camera['distance'] /= 1.1 ** (1.0/(60*sGetFrameTime()))
        
    if sMouseGetVerticalScroll()<0:
        camera['distance'] *= 1.1 ** (1.0/(60*sGetFrameTime()))
        
    if sMouseGetKeyState(2):
        camera['target'] += Vector(camera.transform_global.ix, camera.transform_global.iy, camera.transform_global.iz) *sGetFrameTime()*delta.x * 0.2
        camera['target'] -= Vector(camera.transform_global.jx, camera.transform_global.jy, camera.transform_global.jz) *sGetFrameTime()*delta.y * 0.2
    
    if sMouseGetKeyState(0):
        camera['inertia'] -= sGetFrameTime()*delta * 0.1
    
    if sMouseGetKeyState(0) or sMouseGetKeyState(2):
        mpos = sMouseGetPosition()
        mpos.x %= camRes[0]
        mpos.y %= camRes[1]
        sMouseSetPosition(mpos.x, mpos.y)
    
    camera.resetLocalPostion()
    
    pos = Vector(
        -camera['distance'] * sin(camera['rotation'].x) * cos(camera['rotation'].y),
        -camera['distance'] * cos(camera['rotation'].x) * cos(camera['rotation'].y),
         camera['distance'] * sin(camera['rotation'].y) + 1.5
    ) + camera['target']
    
    camera.rotateGlobal(0.0,0,-camera['rotation'].x)
    camera.rotateLocal(-camera['rotation'].y + 3.1415926535/2,0,0)
    camera.globalPosition = pos
    
    camera['rotation']+= camera['inertia']
    
    camera['inertia'] *= 0.6
    camera['prev_pos'] = sMouseGetPosition()

def init(scene):
    global skel,char,__camera
    __camera = scene.getObject('cCamera')
    #__camera.setBehaviour(cam_fly)
    scene.loadAction('data/mesh/MHCharacterActions.anim', 'MHCharacterActions')
    scene.loadAction('data/mesh/MHCharacterMoving.anim',  'MHCharacterMoving')
    skel = scene.getObject('srig')
    skel['lfoot'] = skel.bones["bDEF-toe.L"]
    skel['rfoot'] = skel.bones["bDEF-toe.R"]
    skel['spine'] = skel.bones["bDEF-spine"]
    skel['walk_speed'] = 1.2
    skel['run_speed'] = 3.0
    
    skel['hands_action'] = 'MHCharacterActions'
    
    skel['hands_standing_start'] = 2.0
    skel['hands_standing_end'] = 8.0
    skel['hands_standing_speed'] = 0.01
    
    skel['hands_walking_start']  = 10.0
    skel['hands_walking_end']    = 20.0
    skel['hands_walking_period'] = 1.25
    
    skel['hands_running_start']  = 30.0
    skel['hands_running_end']    = 50.0
    skel['hands_running_period'] = 0.75
    
    skel['legs_action'] = 'MHCharacterMoving'
    
    skel['legs_standing_start'] = 0.0
    skel['legs_standing_end'] = 0.0
    skel['legs_standing_period'] = 0.00
    
    skel['legs_walking_start'] =  5.0
    skel['legs_walking_end']   = 34.0
    skel['legs_walking_period'] =  1.25
    
    skel['legs_running_start'] = 45.0
    skel['legs_running_end']   = 85.0
    skel['legs_running_period'] =  0.75
    
    skel['head_body_rot_prop'] = 1.0
    
    skel['head_rot_up']    = 145.0
    skel['head_rot_down']  = 137.0
    skel['head_rot_left']  = 133.0
    skel['head_rot_right'] = 124.0
    
    skel['body_rot_up']    = 109.0
    skel['body_rot_down']  = 101.0
    skel['body_rot_left']  = 113.0
    skel['body_rot_right'] = 121.0
    
    controller = CharacterController(skel, 'smems')
    controller.camera = __camera
    sMouseShow()
    sUI.hideConsole()
