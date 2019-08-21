from siberian_ctypes import sGetFrameTime,Vector,laLookAt,rayZ,Object, laIdentity
from random import choice

def bullet(obj):
  #print('bullet')
  #obj = Object(obj)
    if obj.name!='omaslina':
        return
    scene = obj.scene
    frame_time = sGetFrameTime()
    if not 'velocity' in obj.attributes:
        obj.attributes['velocity'] = Vector(0,0,100.0)
    if not 'timer' in obj.attributes:
        #obj.attributes['velocity'] /= abs(obj.attributes['velocity'])
        #obj.attributes['velocity'] *= 100.0
        
        obj.attributes['timer'] = 10.0
        initial_tr = obj.attributes['muzzle'].transform_global
        init_pos = Vector(initial_tr.x, initial_tr.y, initial_tr.z)
        initial_tr.x = initial_tr.y = initial_tr.z = 0.0
        
        obj.transform_global_previous = obj.transform_global = obj.transform = initial_tr
        
        obj.attributes['velocity'] *= initial_tr
        obj.ray.dir = 3
        obj.raySensorOn(abs(obj.attributes['velocity'])*frame_time*2.5)
        
        vel = obj.attributes['velocity']
        #print("Вектор скорости при выстреле", vel)
        #vel.z -= 9.8*frame_time
        obj.transform_global = obj.transform = laIdentity
        obj.translateGlobal(init_pos)
        obj.transform_global_previous = obj.transform_global = obj.transform = laLookAt(obj.transform,obj.transform - vel,1,2)
        return
  
    obj.attributes['timer'] -= frame_time
    if (obj.attributes['timer']<0.0):
        obj.endObject()
        return
    
    vel = obj.attributes['velocity']
    vel.z -= 9.8*frame_time
    obj.ray.dir = 3
    
    obj.transform_global = obj.transform = laLookAt(obj.transform,obj.transform - vel,1,2)
    obj.translateGlobal(vel*frame_time)
    
    nearest = None
    bullet_contact = None
    bullet_contact_n = None
    
    muzzle = obj.attributes['muzzle']
    muzzle_pos = Vector(muzzle.transform_global.x,muzzle.transform_global.y,muzzle.transform_global.z)
    contacts = sorted(obj.ray.contacts,key=lambda contact : abs(contact.hitPosition-muzzle_pos))
    contactsCount = len(contacts)

    contact = None
    for i in contacts:
        if str(i.hitObject.name) == 'oPlayer':
            continue
        else:
            contact = i
            break

    if contact is not None:
        nearest = contact.hitObject
        bullet_contact = contact.hitPosition
        bullet_contact_n = contact.hitNormal
        
        hole = scene.addObject('obullet_hole')
        
        hole.mesh.material.height_scale = 0.2
        
        hole.transform = obj.transform
        hole.transform.x = bullet_contact.x
        hole.transform.y = bullet_contact.y
        hole.transform.z = bullet_contact.z
        
        #hole.transform_global = hole.transform = laLookAt(hole.transform,hole.transform - bullet_contact_n,1,2)
        
        if not 'kill_mob' in nearest.attributes:
            nearest.applyHit(bullet_contact,obj.attributes['velocity']/frame_time,obj.attributes['mass'])
        hole.setParent(nearest)
        
        if nearest.mesh and nearest.mesh.material:
            mat_name = nearest.mesh.material.name
            if mat_name.lower().startswith('metal'):
                hole.attachSound(choice(['data/sounds/material/bullet_metal{}.wav'.format(i) for i in range(1,6)]))
            elif mat_name.lower().startswith('wood'):
                hole.attachSound(choice(['data/sounds/material/bullet_wood{}.wav'.format(i) for i in range(1,5)]))
            elif mat_name.lower().startswith('ground') or mat_name.lower().startswith('brick') or mat_name.lower().startswith('concrete'):
                hole.attachSound(choice(['data/sounds/material/bullet_ground{}.wav'.format(i) for i in range(1,5)]))
            elif mat_name.lower().startswith('sand'):
                hole.attachSound(choice(['data/sounds/material/bullet_sand{}.wav'.format(i) for i in range(1,4)]))

        if 'kill_mob' in nearest.attributes:
            nearest.attributes['kill_mob'](nearest)

        obj.endObject()
