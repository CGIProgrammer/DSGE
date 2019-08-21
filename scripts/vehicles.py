from siberian_ctypes import sMeshNull,sVehicle

def carControl(veh):
    return veh

def armyJeepInit(scene):
    collider = scene.addObject('oveh_army_jeep_1_collider')
    flw = scene.addObject('oveh_army_jeep_1_flw')
    frw = scene.addObject('oveh_army_jeep_1_frw')
    blw = scene.addObject('oveh_army_jeep_1_blw')
    brw = scene.addObject('oveh_army_jeep_1_brw')
    fls = scene.addObject('oveh_army_jeep_1_fls')
    frs = scene.addObject('oveh_army_jeep_1_frs')
    bls = scene.addObject('oveh_army_jeep_1_bls')
    brs = scene.addObject('oveh_army_jeep_1_brs')
    if collider:
        collider.mesh = sMeshNull
        veh = sVehicle(scene,
                       collider,
                       flw,frw,blw,brw,
                       fls,frs,bls,brs)
        return veh

def mersInit(scene):
    collider = scene.addObject('oveh_mers_collider')
    flw = scene.addObject('omers_wheel_tire_fl')
    frw = scene.addObject('omers_wheel_tire_fr')
    blw = scene.addObject('omers_wheel_tire_bl')
    brw = scene.addObject('omers_wheel_tire_br')
    fls = scene.addObject('oveh_mers_fls')
    frs = scene.addObject('oveh_mers_frs')
    bls = scene.addObject('oveh_mers_bls')
    brs = scene.addObject('oveh_mers_brs')
    print("building merssd")
    flw.mesh.material.friction = 100000
    flw.mesh.material.friction = 100000
    brw.mesh.material.friction = 100000
    brw.mesh.material.friction = 100000
    print(collider.name)
    print(flw.name)
    print(frw.name)
    print(blw.name)
    print(brw.name)
    print(fls.name)
    print(frs.name)
    print(bls.name)
    print(brs.name)
    if collider:
        collider.mesh = sMeshNull
        veh = sVehicle(scene,
                       collider,
                       flw,frw,blw,brw,
                       fls,frs,bls,brs)
        return veh
