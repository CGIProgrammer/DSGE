#coding: utf-8
from struct import *
from PIL import Image,ImageDraw,ImageFont
from sys import argv

if len(argv)>1:
    font_name = argv[1]
else:
    font_name = 'Hack-Regular'

fs = 64
fw = int(round(fs*0.5))

I = 16
J = 16

img_fnt = Image.new('L', size=(fw*I,fs*J))
canvas = ImageDraw.Draw(img_fnt)

font = ImageFont.truetype(font_name, int(round(fs*0.8)))

'абвгдеёжзийклмнопрстуфхцчшщъыьэюяАБВГДЕЁЖЗИЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯ'

for i in range(0,I):
    for j in range(0,J):
        code = i*J+j
        rect_start = i * fw, j * fs
        rect_end = (i+1) * fw, (j+1) * fs
        canvas.rectangle((rect_start, rect_end), fill=(0,))
        try:
            char = pack('!B',i*J+j).decode('cp1251')
            canvas.text((i*fw,j*fs),char,(255,),font=font)
            
            bs = char.encode()
            if len(bs)==1:
                bs = b'\x00' + bs
            print(hex(unpack("<h", bs)[0]))
            
        except:
            print(0, char)
            pass

if font_name.rfind('.')>-1:
    img_fnt.save(font_name[:font_name.rfind('.')] + '.png')
else:
    img_fnt.save(font_name + '.png')
