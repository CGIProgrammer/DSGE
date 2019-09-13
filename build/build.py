import subprocess
import threading
import platform
import time
import sys
import os

progname = 'siberian'
src_dir = 'src'
inc_dir = 'inc'
gcc_dir = 'gcc'
obj_dir = 'build'

#path = os.getcwd()

sources = [
    '2D_renderer/2D_renderer.c',
    '2D_renderer/fForm.c',
    '2D_renderer/fText.c',
    '2D_renderer/forms.c',
    'game_functions.c',
    'animation.c',
    'renderer.c',
    'textures.c',
    'objects.c',
    'physics.c',
    'engine.c',
    'linalg.c',
    'shader.c',
    'sound.c',
    'scene.c',
    'glew.c',
    'mesh.c',
    'wav.c',
    'siberian.c'
]

debug = ''
optimise = ''

for i in ('1','2','3'):
    flag = '-g'+i
    if flag in sys.argv:
        debug = flag

for i in ('1','2','3','s','g','fast'):
    flag = '-O'+i
    if flag in sys.argv:
        optimise = flag

shared = '-shared' if '-shared' in sys.argv else ''

artifact_name = progname

if platform.system()=='Windows':
    libs = [
        'openal32',
        'glfw3dll',
        'ode_double',
        'opengl32',
        'm'
    ]
    #gcc_dir = '..\\mingw32\\bin\\gcc'
else:
    libs = [
        'pthread',
        'openal',
        'glfw',
        'ode',
        'GL',
        'm'
    ]
    
if shared:
    if platform.system()=='Windows':
        artifact_name += '.dll'
    else:
        artifact_name += '.so'
else:
    if platform.system()=='Windows':
        artifact_name += '.exe'

objects = [os.path.join(obj_dir, i[i.rfind('/')+1:i.rfind('.')] + '.o') for i in sources]
sources = [os.path.join(src_dir, i) for i in sources]

fPIC = '-fPIC ' if '-shared' in sys.argv else ''
libs = ' '.join(['-l' + i for i in libs])

compile_command = "{} -std=c11 -Wall {} -c {} -I{} {} {} -o {}"
link_command = "{} -Wall {} {} -o {} {}".format(gcc_dir, shared, ' '.join(objects), artifact_name, libs)

threads = []
for i in range(len(sources)):
    cmd = compile_command.format(gcc_dir, fPIC, debug, inc_dir, optimise, sources[i], objects[i])
    thread = threading.Thread(target=subprocess.call, args=(cmd,), kwargs={'shell':True})
    threads.append(thread)
    #subprocess.call(cmd, shell=True)

t1 = time.time()
i = 0
while i<len(threads):
    if threading.active_count()<8:
        threads[i].start()
        i += 1
    else:
        time.sleep(0.1)

for t in threads:
    t.join()

subprocess.call(link_command, shell=True)

print('Сборка выполнена за', round(time.time()-t1, 5),'секунд')
