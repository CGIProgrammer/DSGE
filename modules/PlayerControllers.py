from siberian_ctypes import *
from math import *

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
        self.__iteractFunc = None
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
        self.__actionStack = []
        self.__activePose = CharacterController.__STANDING
    
    @property
    def collider(self):
        return self.__collider
    
    def pushAction(self, action):
        self.__actionStack.append(action)
        
    def popAction(self):
        if not self.__skeleton.isPlayingAction(4) and self.__actionStack:
            return self.__actionStack.pop(0)
        return None
    
    def setIteractFunc(self, func):
        self.__iteractFunc = func
    
    def die(self, targ=None):
        if isinstance(targ, sObject):
            self.__camera['target'] = targ
            self.__camera.setBehaviour(self.__3rdpw)
        else:
            del self.__camera['target']
            self.__camera.removeBehaviour()
        self.__skeleton.removeParent()
        self.__collider.endObject()
        
    def suspend(self):
        self.__skeleton.removeParent()
        self.__collider.suspendDynamics()
        self.__collider.setParent(self.__skeleton)
        self.__collider.removeBehaviour()
        
    def restore(self):
        self.__collider.removeParent()
        self.__collider.restoreDynamics()
        self.__skeleton.setParent(self.__collider)
        self.__collider.setBehaviour(CharacterController.__handling)
	
    def stopAction(self):
        if not self.__skeleton.isPlayingAction(4): self.__skeleton.setLayerWeight(4,0.0)
    
    def playAction(self,name,start,end,speed):
        self.__skeleton.playAction(name, 4, ACTION_PLAY, start, end, speed)
        self.__skeleton.setLayerWeight(4,1.0)
    
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
        cam.globalPosition = player.globalPosition + (vect - side*0.2)*dist + Vector(0.0, 0.0, 0.75)
    
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
            skel.setActionFrame2(self.__MOVING_LAYER_1, self.__loopTimer)
            skel.setActionFrame2(self.__ACTION_LAYER_1, self.__loopTimer)
            skel.setActionFrame2(self.__MOVING_LAYER_2, self.__loopTimer)
            skel.setActionFrame2(self.__ACTION_LAYER_2, self.__loopTimer)
    
    @staticmethod
    def __handling(player):
        #print(player.scene._gobjects_count)
        controller = player.attributes['controller']
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
        
        ##############
        controller.__animation()
        CharacterController.__3rdpw(cam)
        if controller.__iteractFunc and sKeyboardGetKeyState(KEY_F)==1:
            controller.__iteractFunc()
        controller.stopAction()
        act = controller.popAction()
        if act: act(controller)
        ##############
    
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
