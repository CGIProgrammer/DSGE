#coding: utf8
from random import random,choice
from math import pi,sin,cos
from .weapon import *
from .projectile import *
from siberian_ctypes import *#ACTION_PLAY,ACTION_PLAY,ACTION_LOOP
#from siberian_ctypes import sGetFrameTime,sKeyboardGetKeyState,sMouseGetKeyState,sObjectSetMeshPtr,sMeshNull,sPlayerSetImpact
#from siberian_ctypes import KEY_1,KEY_2,KEY_3,KEY_R,KEY_E
#from siberian_ctypes import sUI,sCamera
import sys
from ctypes import c_float,cast,POINTER,pointer

ITEM,USEFUL,AMMO,WEAPON,THROWABLE,BAT = list(range(6))

inName,iName,iType,iAmmoCount = list(range(4))
HUD_Showing,HUD_Hiding,HUD_Hidden,HUD_Idle,HUD_Shooting,HUD_Reloading = list(range(6))
HUD_Shown = HUD_Idle

items = {}
items['wpn_spas12'] = ('wpn_spas12','SPAS 12',WEAPON,8)
items['wpn_ak12'] = ('wpn_ak12','AK-12',WEAPON,30)
items['wpn_glow_stick'] = ('wpn_glow_stick','ХИС', WEAPON,0)
items['ammo_12g'] = ('ammo_12g','12 gage',AMMO,50)
items['ammo_5.45x39'] = ('ammo_5.45x39','5.45x39',AMMO,50)
items[None] = ('','','','')

class HandsHUD:
    def __init__(self,object,hand_bone,item_model):
        self.__camera = cast(pointer(object.parent),POINTER(sCamera)).contents
        self.__hands = object
        self.__item_model = item_model
        self.__scene = object.getScene()
        self.__hand_bone = hand_bone
        self.__items = []
        self.__slots = [None,None,None]
        self.__active = 0
        self.__status = HUD_Hidden
        self.__actions_stack = []
        self.__shoot_timer = 0.0
        self.__rocking_timer = 0.0
        self.__displacement = 0.0
        self.__aim = 0
        
        self.rocking_amp = 0.0
    
    @property
    def status(self):
        return self.__status
    
    @property
    def wpnName(self):
        if self.__slots[self.__active] is None or self.__active is None or self.__status==HUD_Hidden:
            return ''
        return self.__slots[self.__active][iName]
    
    @property
    def mag_shells(self):
        gun = self.__slots[self.__active]
        return gun[iAmmoCount]
    
    @property
    def shells_name(self):
        gun = self.__slots[self.__active]
        return items[weaps[gun[inName]]['cal']][iName]
    @property
    def shells_inname(self):
        gun = self.__slots[self.__active]
        return weaps[gun[inName]]['cal']
        
    @property
    def all_shells(self):
        result = 0
        gun = self.__slots[self.__active]
        cal = weaps[gun[inName]]['cal']
        for i in self.__items:
            if i[inName]==cal:
                result+=i[iAmmoCount]
        return result
    
    @property
    def wpn_params(self):
        return weaps[self.__slots[self.__active][inName]]
    
    @property
    def wpn_anims(self):
        return hands_animations[self.__slots[self.__active][inName]]
    
    def __set_status(self,status):
        self.__status = status
        
    def __set_active(self,num):
        self.__active = num
        
    def __set_item_model(self,object):
        if object.name[0]=='o':
            self.__item_model = object
        else:
            raise TypeError
    
    def show(self):
        if self.__slots[self.__active] is None or self.__active is None or self.__status!=HUD_Hidden: return
        wpn_name = self.__slots[self.__active][inName]
        ani_params = self.wpn_anims[SHOW]
        
        self.__status = HUD_Showing
        #self.__item_model.mesh = 'mshooing_range/{}_hud'.format(wpn_name)
        self.__hands.show()
        self.__hands.bones['bRifle'].attachSound(hands_animations[wpn_name][SHOW_SOUND])
        self.__hands.playAction(self.wpn_anims[IDLE][0],0,ACTION_LOOP,self.wpn_anims[IDLE][1],self.wpn_anims[IDLE][2],0.025)
        self.__item_model.mesh = 'weapon/{}'.format(self.__slots[self.__active][inName])
        self.__actions_stack.append(lambda : self.__hands.playAction(ani_params[0],1,ACTION_PLAY,ani_params[1],ani_params[2],1.0))
        self.__actions_stack.append(lambda : self.__set_status(HUD_Idle))
        
    def hide(self):
        if self.__slots[self.__active] is None or self.__active is None or self.__status!=HUD_Shown: return
    
        if self.__aim:
            self.aim()
            
        wpn_name = self.__slots[self.__active][inName]
        ani_params = self.wpn_anims[SHOW]
        self.__status = HUD_Hiding
        self.__hands.stopAction(0)
        self.__hands.bones['bRifle'].attachSound(hands_animations[wpn_name][HIDE_SOUND])
        self.__actions_stack.append(lambda : self.__hands.playAction(ani_params[0],1,ACTION_PLAY,ani_params[2],ani_params[1],1.0))
        self.__actions_stack.append(lambda : self.__hands.hide())
        self.__actions_stack.append(lambda : self.__set_status(HUD_Hidden))
    
    def shoot(self):
        if self.__slots[self.__active] is None or self.__active is None or self.__status!=HUD_Shown: return
        gun = self.__slots[self.__active]
        wpn_name = gun[inName]
        wpn_params = self.wpn_params
        
        if not (gun[iAmmoCount] and gun[iType]!=BAT) or self.__shoot_timer<wpn_params['shot_time']: return
            
        if gun[iType]==BAT:
            ani_params = hands_animations[wpn_name][ATTACK]
            self.__hands.playAction(ani_params[0],1,ACTION_ADD_PLAY,ani_params[1],ani_params[2],1.0)
        elif gun[iType]==THROWABLE:
            pass
        
        else:
            gun[iAmmoCount]-=1
            self.__shoot_timer = 0
            if (self.__aim):
                ani_params = hands_animations[wpn_name][AIM_SHOT]
            else:
                ani_params = hands_animations[wpn_name][ATTACK]
            self.__hands.playAction(ani_params[0],1,ACTION_ADD_PLAY,ani_params[1],ani_params[2],1.0)
            self.__hands.bones['bRifle'].attachSound(choice(hands_animations[wpn_name][SHOT_SOUND]))
            #for i in range(8):
            #  self.__hands.scene.getObject('oecho_point.%03d' % i).attachSound('data/sounds/machinegunes/shoot_echo%d.wav' % i)
            
            sPlayerSetImpact((random()-0.5)*0.0025,0.05,0.0)
            maslina = self.__scene.addObject('omaslina')
            if not maslina is None:
                maslina.attributes['muzzle'] = self.__hands.bones['bSilencer']
                ammoType = self.shells_inname
                disp = Vector(((random()-0.5)*2.0),((random()-0.5)*2.0),0.0)
                disp = disp/abs(disp) * (random()**3) * ammos[ammoType][SPEED] * wpn_params['accuracy']
                maslina.attributes['velocity'] = Vector(disp.x,disp.y,ammos[ammoType][SPEED])
                maslina.attributes['mass'] = ammos[ammoType][MASS]
                maslina.setBehaviour(bullet)
    
    def aim(self):
      if self.__slots[self.__active] is None or self.__active is None or self.__status!=HUD_Shown or self.__hands.isPlayingAction(1): return
      gun = self.__slots[self.__active]
      ani_params = hands_animations[gun[inName]][AIM]
      ani_idle = hands_animations[gun[inName]][IDLE]
      if self.__aim:
        self.__hands.playAction(ani_idle[0],0,ACTION_LOOP,ani_idle[1],ani_idle[2],0.025)
        self.__actions_stack.append(lambda : self.__hands.playAction(ani_params[0],1,ACTION_PLAY,ani_params[2],ani_params[1],1.0))
      else:
        self.__hands.playAction(ani_params[0],0,ACTION_LOOP,ani_params[2],ani_params[2],1.0)
        self.__actions_stack.append(lambda : self.__hands.playAction(ani_params[0],1,ACTION_PLAY,ani_params[1],ani_params[2],1.0))
      self.__aim = not self.__aim
    
    def __load_shell(self,count):
        gun = self.__slots[self.__active]
        cal = weaps[gun[inName]]['cal']
        ammo = None
        index = 0
        for i in self.__items:
            if i[inName]==cal:
                ammo = i
                break
            index+=1
        
        if ammo is None: return 0
        count = min(count,ammo[iAmmoCount])
        gun[iAmmoCount]+=count
        ammo[iAmmoCount]-=count
        if not ammo[iAmmoCount]:
            del self.__items[index]
        return 1
    
    def reload(self):
        if self.__slots[self.__active] is None or self.__active is None or self.__status!=HUD_Shown: return
    
        if self.__aim:
            self.aim()
    
        gun = self.__slots[self.__active]
        name = gun[inName]
        mag_shells = self.mag_shells
        mag_size = weaps[name]['mag_size']
        all_shells = self.all_shells
        if mag_shells==mag_size or not all_shells or self.__shoot_timer<weaps[name]['shot_time']: return
        cal = self.shells_name
        reload_type = weaps[name]['reload_type']
        
        self.__set_status(HUD_Reloading)
        ani_params = hands_animations[name][RELOAD]
        
        ani_idle = hands_animations[gun[inName]][IDLE]
        self.__hands.stopAction(0)
        
        if reload_type==PUMP:
            available_shells = min(mag_size-mag_shells,all_shells) - (not mag_shells)
            self.__hands.playAction(ani_params[0],1,ACTION_PLAY,ani_params[1],ani_params[2],1.0)
            for i in range(available_shells):
                #self.__actions_stack.append(lambda : self.__scene.getObject('oshot_point').attachSound("data/sounds/shotgun/shell.wav"))
                self.__actions_stack.append(lambda : self.__hands.playAction(ani_params[0],1,ACTION_PLAY,ani_params[2],ani_params[3],1.0))
                self.__actions_stack.append(lambda : self.__load_shell(1))
            self.__actions_stack.append(lambda : self.__hands.playAction(ani_params[0],1,ACTION_PLAY,ani_params[3],ani_params[4],1.0))
            if len(ani_params)>5 and not mag_shells:
                #self.__actions_stack.append(lambda : self.__scene.getObject('oshot_point').attachSound("data/sounds/shotgun/pump1.wav"))
                self.__actions_stack.append(lambda : self.__hands.playAction(ani_params[0],1,ACTION_PLAY,ani_params[4],ani_params[5],1.0))
        elif reload_type==MAG:
          available_shells = min(mag_size-mag_shells,all_shells) - (not mag_shells)
          self.__hands.bones['bRifle'].attachSound(hands_animations[name][RELOAD_SOUND])
          self.__actions_stack.append(lambda : self.__hands.playAction(ani_params[0],1,ACTION_PLAY,ani_params[1],ani_params[2],1.0))
          self.__actions_stack.append(lambda : self.__load_shell(available_shells))
          #self.__actions_stack.append(lambda : sys.stdout.write('{}/{}\n'.format(self.mag_shells,self.all_shells)))
        self.__actions_stack.append(lambda : self.__set_status(HUD_Idle))
        self.__actions_stack.append(lambda : self.__hands.playAction(ani_idle[0],0,ACTION_LOOP,ani_idle[1],ani_idle[2],0.025))
    
    def __rocking(self):
        if self.__status==HUD_Hidden: return
        sig = lambda x : 1.0/(1.0+2.7182818**(8.0*(0.5-x)))
        
        self.__rocking_timer += pi*0.03*sGetFrameTime()/0.01666
        if self.__rocking_timer>2*pi:
            self.__rocking_timer = 0.0
    
    def process(self):
      self.__rocking()
      keys = [sKeyboardGetKeyState(KEY_1), sKeyboardGetKeyState(KEY_2), sKeyboardGetKeyState(KEY_3)]
      if sum(keys):
        if keys[self.__active]:
            if self.status==2:
                self.show()
            if self.status==3:
                self.hide()
        elif self.__status==HUD_Idle:
            if self.status==3:
                self.hide()
            self.__actions_stack.append(lambda : self.__set_active(keys.index(1)))
            self.__actions_stack.append(self.show)
      
      if sKeyboardGetKeyState(KEY_R):
        self.reload()
      
      if sMouseGetKeyState(1):
        self.aim()
        
      if sMouseGetKeyState(0):
        self.shoot()
      
      self.__shoot_timer += sGetFrameTime()
      if self.__hands.isPlayingAction(1) or not len(self.__actions_stack): return
      self.__actions_stack.pop(0)()
    
    def addItem(self,name):
        item = list(items[name])
        self.__items.append(item)
        for i in range(len(self.__slots)):
            if self.__slots[i] is None:
                self.__slots[i] = item
                break
    
    def drop(self,name):
        return
    
    def pick(self,object):
        return
