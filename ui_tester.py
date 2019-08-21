from siberian_ctypes import *

sEngineSetSwapInterval(1)
sEngineCreateWindow(800, 600, 0)
sMouseShow()
sEngineStartOpenGL()

def func(obj, num):
    print(num)

sad = fList(300,100, 150, 300, func)
sad.addItem("Строка 1")
sad.addItem("Строка 2")
sad.addItem("Кириллица")

sEngineStartLoop()
sSoundCloseDevice()