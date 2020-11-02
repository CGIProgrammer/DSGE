import numpy as np
from numpy import linalg as la
try:
    from PIL import Image
except:
    pass
from math import pi,sin,log
from random import randint, shuffle

def prod(arr):
    result = 1
    for i in arr:
        result *= i
        if result==0:
            return 0
    return result

def save_array(img, name):
    shp = tuple(img.shape)
    #img.shape = prod(shp)
    #mx = np.max(img)
    #mn = np.min(img)
    #img.shape = shp
    #img = np.round((img-mn)/(mx-mn)*255).astype(np.uint8)
    img = np.round(np.clip(img, 0.0, 1.0)*255).astype(np.uint8)
    img = Image.fromarray(img)
    img.save(name)

R2 = 19
SIGMA = 1.414
M_PI = 3.14159265359

adot = lambda arr1,arr2 : np.sum(arr1*arr2, axis=2)
normalize = lambda arr : la.norm(arr, axis=2)
length = lambda arr : np.sqrt(adot(arr, arr))
ivec2 = lambda x,y : np.array([x,y], dtype=np.int)
ivec3 = lambda x,y,z : np.array([x,y,z], dtype=np.int)
vec2 = lambda x,y : np.array([x,y], dtype=np.float)
vec3 = lambda x,y,z : np.array([x,y,z], dtype=np.float)

def merge(*arrays):
    cmps = 0
    for i in arrays:
        if isinstance(i, np.ndarray) and len(i.shape)==3:
            cmps += i.shape[2]
        else:
            cmps += 1
    s = arrays[0].shape
    result = np.zeros((s[0]*s[1], cmps))
    component = 0
    for array in arrays:
        if isinstance(array, (float, int)):
            result[:,component] += array
            component += 1
        else:
            shp = array.shape
            if len(shp)==2:
                result[:,component] = shp
                component += 1
            else:
                array.shape = shp[0]*shp[1], shp[2]
                for i in range(shp[2]):
                    result[:,component] = array[:,i]
                    component += 1
            array.shape = shp
    result.shape = s[0], s[1], cmps
    return result

def recomp(arr, *comps):
    shape = tuple(arr.shape)
    if len(shape)==2:
        arr.shape = shape[0] * shape[1], 1
    else:
        arr.shape = shape[0] * shape[1], shape[2]
    result = np.zeros((shape[0] * shape[1], len(comps)))
    for i,comp in enumerate(comps):
        result[:,i] = arr[:,comp]
    arr.shape = shape
    if len(comps)==1:
        result.shape = (shape[0], shape[1])
    else:
        result.shape = (shape[0], shape[1], len(comps))
    return result

def hash21(p:np.ndarray):
    if isinstance(p, (float,int)):
        p3 = np.modf(vec3(p*0.1031, p*0.1030, p*0.0973))[0]
        p3 += np.dot(vec3(p,p,p), vec3(0.1031, 0.1030, 0.0973))
        p3xx = vec2(p3[0], p3[0])
        p3yz = vec2(p3[1], p3[2])
        p3zy = vec2(p3[2], p3[1])
    else:
        p3 = np.modf(recomp(p, 0,0,0) * np.array([.1031, .1030, .0973]))[0]
        p3 += recomp(adot(p3, recomp(p3, 1,2,0) + 19.19), 0,0,0)
        p3xx = recomp(p3, 0,0)
        p3yz = recomp(p3, 1,2)
        p3zy = recomp(p3, 2,1)
    return np.modf((p3xx+p3yz)*p3zy)[0]

def hash13(p3:np.ndarray):
    if len(p3.shape)==1:
        p3  = np.modf(p3 * 0.1031)[0]
        p3 += np.dot(p3, vec3(p3[1], p3[2], p3[0]) + 19.19)
        return np.modf((p3[0] + p3[2]) * p3[2])[0]
    elif p3.shape[-1]==3:
        p3  = np.modf(p3 * .1031)[0]
        p3 += recomp(adot(p3, recomp(p3, 1,2,0) + 19.19), 0,0,0)
        return np.modf((recomp(p3,0) + recomp(p3,2)) * recomp(p3,2))[0]
    else:
        raise TypeError("hash13 wrong type")

def gaussian (x, sigma):
    h0 = x / sigma
    h = h0 * h0 * -0.5
    a = 1.0 / (sigma * np.sqrt(2.0 * M_PI))
    return a * np.exp(h)

def distf(v, x):
    return 1.0 - x

def texelFetch(sampler, coords):
    sshape = sampler.shape
    cshape = coords.shape
    coords = coords%np.array([cshape[1],cshape[0]])
    coords.shape = cshape[0]*cshape[1], 2
    lin_crd = coords[:,0] * cshape[1] + coords[:,1]
    if len(sampler.shape)==3:
        sampler.shape = sshape[0]*sshape[1], sshape[2]
        result = sampler[lin_crd]
        result.shape = cshape[0], cshape[1], sshape[2]
        sampler.shape = sshape
    else:
        sampler.shape = sshape[0]*sshape[1]
        result = sampler[lin_crd]
        result.shape = cshape[0], cshape[1]
        sampler.shape = sshape
    return result


def quantify_error(channel, p, sz, val0, val1):
    Rf = float(R2) / 2.0
    R = int(Rf)
    has0 = np.zeros(channel.shape[:2])
    has1 = np.zeros(channel.shape[:2])
    w = 0.0
    
    for sy in range(-R, R+1):
        for sx in range(-R, R+1):
            d = (sx**2 + sy**2)**0.5
            if (d > Rf) or ((sx == 0) and (sy == 0)):
                continue
            t = (p + ivec2(sx,sy) + sz) % sz
            v = texelFetch(channel, t)[:,:,0]

            q = gaussian(d, SIGMA)
            has0 += (1.0 - np.absolute(v - val0)) * q
            has1 += (1.0 - np.absolute(v - val1)) * q
            w += q

    result = np.zeros((has0.shape[0]*has0.shape[1], 2))
    result[:,0] = (has0 / w).flatten()
    result[:,1] = (has1 / w).flatten()
    result.shape = (has0.shape[0], has0.shape[1], 2)
    return result

def mean_curvature_flow (src_buf,src_stride,dst_buf,dst_width,dst_height,dst_stride):
    c = None
    x = None
    y = None
    center_pix = None
    O = lambda u,v : ((u)+((v) * src_stride)) * 4

    offsets = [O( -1, -1), O(0, -1), O(1, -1),
               O( -1,  0),           O(1,  0),
               O( -1,  1), O(0, 1),  O(1,  1)]

    LEFT = lambda c : (center_pix + offsets[3])[c]
    RIGHT = lambda c : (center_pix + offsets[4])[c]
    TOP = lambda c : (center_pix + offsets[1])[c]
    BOTTOM = lambda c : (center_pix + offsets[6])[c]
    TOPLEFT = lambda c : (center_pix + offsets[0])[c]
    TOPRIGHT = lambda c : (center_pix + offsets[2])[c]
    BOTTOMLEFT = lambda c : (center_pix + offsets[5])[c]
    BOTTOMRIGHT = lambda c : (center_pix + offsets[7])[c]

    for y in range(dst_height):
        print("y",y,'/',dst_height,end='        \r')
        dst_offset = dst_stride * y
        center_pix = src_buf[((y+1) * src_stride + 1) * 4:]
        for x in range(dst_width):
            for c in range(3):
                dsto = dst_offset * 4 + c
                dx  = RIGHT(c) - LEFT(c)
                dy  = BOTTOM(c) - TOP(c)
                #print(dx,dy)
                magnitude = (dx*dx + dy*dy) * 0.5
                dst_buf[dsto] = center_pix[c]

                if magnitude:
                    dx2 = dx*dx
                    dy2 = dy*dy

                    dxx = RIGHT(c) + LEFT(c) - 2. * center_pix[c]
                    dyy = BOTTOM(c) + TOP(c) - 2. * center_pix[c]
                    dxy = 0.25 * (BOTTOMRIGHT(c) - TOPRIGHT(c) - BOTTOMLEFT(c) + TOPLEFT(c))
                    n = dx2 * dyy + dy2 * dxx - 2. * dx * dy * dxy
                    d = (dx2 + dy2)**(3/2)
                    mean_curvature = n / d
                    dst_buf[dsto] += (0.25 * magnitude * mean_curvature)

            dst_buf[dst_offset * 4 + 3] = center_pix[3]

            dst_offset += 1
            center_pix += 4
    print('')

def process (inp, iterations):
    width  = len(inp[0])
    height = len(inp)
    src_buf = np.zeros((height+iterations*2, width+iterations*2, 4))
    src_buf[iterations:-iterations,iterations:-iterations] = inp
    dst_buf = np.zeros((height+iterations*2, width+iterations*2, 4)) + np.array([0.0,0.0,0.0,1.0])

    stride = width + iterations * 2

    for iteration in range(iterations):
        print(iteration+1)
        shp = src_buf.shape
        src_buf.shape = prod(src_buf.shape)
        dst_buf.shape = prod(dst_buf.shape)
        mean_curvature_flow (src_buf, stride,
                             dst_buf,
                             width  + (iterations - 1 - iteration) * 2,
                             height + (iterations - 1 - iteration) * 2,
                             stride)
        src_buf.shape = shp
        dst_buf.shape = shp
        save_array(dst_buf, 'mcf_%d.png'%(iteration,))
        tmp = src_buf
        src_buf = dst_buf
        dst_buf = tmp
    return src_buf

sas = np.array(Image.open('noised.png').convert("RGBA")).astype(np.float32)/255.0
save_array(process(sas, 6), 'fin.png')
exit(0)
def fragCoord(width, height, n=False):
    if n:
        result = np.array([[[j,i] for j in range(width)] for i in range(height)]) / np.array([width, height]) * 2.0 - 1.0
    else:
        result = np.array([[[j,i] for j in range(width)] for i in range(height)], dtype=np.int)
    return result

def mainImageA(iChannel0):
    #p0 = fragCoord(iChannel0.shape[1], iChannel0.shape[0])
    #c = texelFetch(iChannel0, p0)
    return np.array(iChannel0)

def mainImageB(iChannel0, iFrame):
    sz = ivec2(iChannel0.shape[1], iChannel0.shape[0])
    szf = vec2(iChannel0.shape[1], iChannel0.shape[0])
    p0 = fragCoord(iChannel0.shape[1], iChannel0.shape[0])
    p0x = recomp(p0, 0)
    p0y = recomp(p0, 1)
    maskf = hash21(iFrame)
    M = 3600
    F = int(iFrame) % M
    framef = F / M
    chance_limit = 0.5
    force_limit = 1.0 - np.clip(framef * 8.0, 0.0, 1.0)
    force_limit = force_limit * force_limit
    force_limit = force_limit * force_limit
    if (F == 0):
        c = (p0x * 61 + p0y) % 256
        return recomp(c, 0,0) * vec2(1.0/255.0, 0.0)
    else:
        mask = np.round((maskf + maskf * framef) * szf).astype(np.int)
        p1 = (p0 ^ mask) % sz
        #pp0 = (p1 ^ mask) % sz

        chance0 = hash13(merge(p0, iFrame))
        chance1 = hash13(merge(p1, iFrame))
        chance = np.max((chance0.flatten(), chance1.flatten()), axis=0)
        chance.shape = chance0.shape
        
        v0 = recomp(texelFetch(iChannel0, p0), 0,0)
        v1 = recomp(texelFetch(iChannel0, p1), 0,0)
        
        s0_x0 = quantify_error(iChannel0, p0, sz, v0[:,:,0], v1[:,:,0])
        s1_x1 = quantify_error(iChannel0, p1, sz, v1[:,:,0], v0[:,:,0])
        
        err_s = recomp(s0_x0, 0) + recomp(s1_x1, 0)
        err_x = recomp(s0_x0, 1) + recomp(s1_x1, 1)
        
        p = v0
        chance = recomp(chance, 0,0)
        err_x = recomp(err_x, 0,0)
        err_s = recomp(err_s, 0,0)
        bl = ((chance < force_limit) | ((chance < chance_limit) & (err_x < err_s)))
        p = v1*bl + (p * ~bl)
        return p

w,h = 64,64
buffA = np.zeros((h,w,2))
buffB = np.zeros((h,w,2))
for i in range(6*60):
    print(i,'/',5)
    np.random.seed(i)
    buffA = mainImageA(buffB)
    try:
        buffB = mainImageB(buffA, i)
    except KeyboardInterrupt:
        break
save_array(recomp(buffB, 0,0,0), "sas.png")
