from siberian_ctypes import *

def __flight(obj):
    obj.translateLocal(Vector(-obj.attributes['ditance']/obj.attributes['time'],0.0,0.0) * sGetFrameTime())
    obj.attributes['timer'] -= sGetFrameTime()
    if obj.attributes['timer']<=0:
        obj.endObject()

def throw(scene):
    if scene.getObject('oflyingBall'):
        return
    obj = scene.addObject('oflyingBall')
    obj.attributes['time'] = 15.0
    obj.attributes['timer'] = obj.attributes['time']
    obj.attributes['ditance'] = 32.0
    obj.setBehaviour(__flight)
