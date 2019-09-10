from siberian_ctypes import *
import os

class ImageViewer:
    def __init__(self, width, height, img='/home/ivan/Изображения/Снимок экрана в 2019-04-01 00-40-22.png'):
        self.__width = width
        self.__height = height
        self.__main = fForm()
        self.__main.width = width
        self.__main.height = height
        self.__bg  = self.__main.addElement(width, height)
        self.__bg.planeColor = 0.5,0.5,0.5,1
        self.__img = self.__main.addElement(width, height)
        self.__img.planeColor = 0.0,0.0,0.0,0.0
        self.__scaling = 1.0
        self.__translation = [0,0]
        #if img:
        #    self.showFile(img)

    @property
    def form(self):
        return self.__main

    def showFile(self, filename):
        self.__scaling = 1.0
        self.__translation = [0,0]
        if self.__img.image:
            self.__img.image.delete()
        if filename[-3:]=='dds':
            self.__img.image = sTexture.loadImage(filename)
        else:
            self.__img.image = sTexture.loadWithCompression(filename)
        self.__img.planeColor = 0,0,0,1

        self.__img.width, self.__img.height = self.__img.image.size
        
        w,h = self.__img.image.size
        aspect = w/h
        
        if w > self.__width:
            w = self.__width
            h = w/aspect
        if h > self.__height:
            h = self.__height
            w = h*aspect

        self.__img.width  = w
        self.__img.height = h

        x = (self.__width  - self.__img.width)  / 2
        y = (self.__height - self.__img.height) / 2

        self.__img.localPosition = x, y

    @property
    def position(self):
        return self.__main.globalPosition
    @position.setter
    def position(self, value):
        self.__main.globalPosition = value

#class ModelViewer:
#    def __init__(self):


class FileBrowser:
    def __init__(self, directory, width = 320, height = 480):
        self.__folderIcon = sTexture.loadImage("/home/ivan/Изображения/UI_icons/places/folder.dds")
        self.__fileIcon = sTexture.loadImage("/home/ivan/Изображения/UI_icons/mimetypes/unknown.dds")
        self.__pyFileIcon = sTexture.loadImage("/home/ivan/Изображения/UI_icons/mimetypes/text-x-python.dds")
        self.__blendFileIcon = sTexture.loadImage("/home/ivan/Изображения/UI_icons/mimetypes/application-x-blender.dds")
        self.__zipFileIcon = sTexture.loadImage("/home/ivan/Изображения/UI_icons/mimetypes/application-x-lzma.dds")
        self.__imgFileIcon = sTexture.loadImage("/home/ivan/Изображения/UI_icons/mimetypes/image-x-generic.dds")
        self.__files_list = None
        self.__imgTypes = {'.png', '.jpg', '.jpeg', '.tga', '.bmp', '.svg', '.dds'}
        self.__archives = {'.zip', '.rar', '.7z', '.tar', '.gz', '.bz2'}
        self.__width = width
        self.__height = height
        self.__root = fForm()
        self.__root.width = 800
        self.__root.height= 480
        self.__iv = ImageViewer(799-width, height)
        self.__iv.position = 321, 0
        self.__root.addForm(self.__iv.form)
        self.scanFiles(directory)
        self.showFiles()
        self.__root.setIdle(self.__slider)
        self.__root['slide'] = False
        self.__root.localPosition = 0, -self.__root.height

    def __enterToFolder(self, form):
        print('Entering to', form['directory'])
        self.scanFiles(form['directory'])
        self.showFiles()

    def __openImage(self, form):
        print('Opening', form['directory'])
        self.__iv.showFile(form['directory'])

    def __openScene(self, form):
        scene = sEngineGetActiveScene()
        scene.destroy()
        scene = sScene(form['directory'][form['directory'].rfind('/')+1:form['directory'].rfind('.')])
        sEngineSetActiveScene(scene)

    def __slider(self, form):
        if sKeyboardGetKeyState(KEY_TAB)==1:
            form['slide'] = not form['slide']
            if form['slide']:
                print('self.__slideIn')
                form.position = 0, -form.height
                form.setIdle(self.__slideIn)
            else:
                print('self.__slideOut')
                form.position = 0, -1
                form.setIdle(self.__slideOut)

    def __slideIn(self, form):
        v = form.localPosition
        v[1] += 10
        if v[1]>0:
            v[1] = 0
            form.setIdle(self.__slider)
            print('end')
        form.localPosition = v
        print(v[1])

    def __slideOut(self, form):
        v = form.localPosition
        v[1] -= 10
        print(v[1])
        if v[1]<-form.height:
            form.setIdle(self.__slider)
            print('end')
        form.localPosition = v

    @staticmethod
    def __file_line_hover(form):
        form['bg'].planeColor = 0.75,0.5,0.25,1

    @staticmethod
    def __file_line_unhover(form):
        form['bg'].planeColor = 0.75,0.75,0.75,1

    def __scrolling(self, form, scroll):
        form.verticalScrollValue -= scroll * 50
        if form.verticalScrollValue>self.__files_list['height']:
            form.verticalScrollValue = self.__files_list['height']
        if form.verticalScrollValue<0:
            form.verticalScrollValue = 0

    def scanFiles(self, directory):
        directory = os.path.abspath(directory)
        dirs = os.listdir(directory)
        self.__directory = directory
        self.__files = []
        self.__directories = []
        for i in dirs:
            if os.path.isdir(os.path.join(directory,i)):
                self.__directories.append(i)
            else:
                self.__files.append(i)
        self.__files.sort()
        self.__directories.sort()

    def showFiles(self, lineHeight=24):
        if self.__files_list:
            self.__files_list.delete()
        self.__files_list = fForm()
        self.__files_list.xRayBit = 1
        self.__files_list.setScroll(self.__scrolling)
        self.__files_list.width = self.__width
        self.__files_list.height = self.__height
        bg = self.__files_list.addElement(self.__width, self.__height)
        bg.width = self.__width
        bg.height = self.__height
        bg.planeColor = 0.5, 0.5, 0.5, 1
        directoriens = ['../',]*(self.__directory!='/') + self.__directories
        for i, filename in enumerate(directoriens):
            line = fForm()
            line['directory'] = os.path.join(self.__directory, filename)
            line.setLMB(self.__enterToFolder)
            line.setCursorRelease(self.__file_line_unhover)
            line.setCursorHover(self.__file_line_hover)
            w, h = self.__width - 2, lineHeight
            line.width = w
            line.height = h
            line['bg'] = line_bg = line.addElement(w, h)
            line_bg.planeColor = 0.75, 0.75, 0.75, 1
            fn = line.addElement(w, h, filename, lineHeight / 3)
            fn.textColor = 1,1,1,1
            fn.localPosition = int(lineHeight*1.16666), lineHeight/5

            ico = line.addElement(lineHeight, lineHeight)
            ico.planeColor = 1,1,1,1
            ico.image = self.__folderIcon

            line.globalPosition = 1, i*(h + 1) + 1
            self.__files_list.addForm(line)

        self.__root.addForm(self.__files_list)


        for i, filename in enumerate(self.__files):
            i += len(directoriens)
            line = fForm()
            line.setCursorRelease(self.__file_line_unhover)
            line.setCursorHover(self.__file_line_hover)
            line['directory'] = os.path.join(self.__directory, filename)
            w, h = self.__width - 2, lineHeight
            line.width = w
            line.height = h
            line['bg'] = line_bg = line.addElement(w, h)
            line_bg.planeColor = 0.75, 0.75, 0.75, 1
            fn = line.addElement(w, h, filename, lineHeight / 3)
            fn.textColor = 1,1,1,1
            fn.localPosition = int(lineHeight*1.16666), lineHeight/5

            ico = line.addElement(lineHeight, lineHeight)
            ico.planeColor = 1,1,1,1
            extension = filename[filename.rfind('.'):].lower()
            if extension=='.py':
                ico.image = self.__pyFileIcon
            elif extension in self.__imgTypes:
                ico.image = self.__imgFileIcon
                line.setLMB(self.__openImage)
            elif extension == '.scene':
                line.setLMB(self.__openScene)
            elif extension in self.__archives:
                ico.image = self.__zipFileIcon
            elif extension[:-1] == '.blend' or extension == '.blend':
                ico.image = self.__blendFileIcon
            else:
                ico.image = self.__fileIcon

            line.globalPosition = 1, i*(h + 1) + 1
            self.__files_list.addForm(line)

        self.__files_list['height'] = (len(directoriens) + len(self.__files))*(h + 1) - self.__height