#from . import first_person
#from . import items_handler
#from . import projectile
#from . import weapon

from ctypes import cast,pointer,POINTER
import os
from . import third_person
from . import first_person
from . import vehicles

ACTION_STOP,ACTION_PLAY,ACTION_LOOP,ACTION_STOPED = list(range(0x00, 0x04))
ACTION_ADD_PLAY,ACTION_ADD_LOOP,ACTION_ADD_STOPED = list(range(0x11, 0x14))

def init(scene,sn):
    if sn=='model_view':
        camera = scene.getObject('cCamera')
        mers = vehicles.mersInit(scene)
        mers.setCamController(camera)
        mers.setTireFriction(1000.0)
    elif sn=='Scene':
        third_person.init(scene)
    else:
        first_person.init(scene)
        pass
    surf = scene.getObject('oasphalt')
    if surf:
        surf.mesh.material.wet = 0.75
