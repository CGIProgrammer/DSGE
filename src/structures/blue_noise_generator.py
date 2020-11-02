import numpy as np
from PIL import Image
from math import e, pi

noise = np.array(Image.open('input.png').convert('L'), dtype=np.float)/255.0*2.0-1.0
n = 64
img = np.zeros((n,n), dtype=np.float)
img2 = np.zeros((n,n), dtype=np.float)
for k in range(n):
    for l in range(n):
        comp = 0
        for i in range(n):
            for j in range(n):
                comp += noise[i][j] * (e**-(1j * 2 * pi * (k*i/n + l*j/n)))
        img[k][l] = abs(comp)
    print('',str(round(k/(n-1)*100, 2))+'%', end='\r')
print('\nend')

def save_array(img, name):
    img.shape = n*n
    mx = np.max(img)
    mn = np.min(img)
    img.shape = n,n
    img = np.round((img-mn)/(mx-mn)*255).astype(np.uint8)
    img = Image.fromarray(img)
    img.save(name)

save_array(img2,'backtra.png')
save_array(img,'spectrum.png')