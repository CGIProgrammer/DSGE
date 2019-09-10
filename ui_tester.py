from siberian_ctypes import *

sEngineSetSwapInterval(0)
sEngineCreateWindow(800, 600, 0)
sMouseShow()
sEngineStartOpenGL()

def func(obj):
    obj['text'].text = "frametime %.4f" % (sGetFrameTime()*1000.0,)

frametime = fForm()
frametime.width = 200
frametime.height = 50
print(frametime.width, frametime.height)
txt = frametime.addElement(200,50,"Frametime")
print(txt.width, txt.height)
txt.textColor = 1,1,1,1

frametime['text'] = txt
frametime.setIdle(func)

scene = sScene(filename="shooting_range")
skybox = sTexture.loadCubemap("data/textures/cubemap/cloudySea.dds")
scene.setSkyTexture(skybox)
sEngineSetActiveScene(scene)

from file_browser import FileBrowser

fb = FileBrowser('/home/ivan/SGM_SDK')

sEngineStartLoop()
#scene.destroy()
sSoundCloseDevice()