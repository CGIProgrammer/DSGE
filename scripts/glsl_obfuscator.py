import sys
from random import randint

defines = {
    'sampler2D' : 's2D',
    'texture' : 'TX'
}

qualifiers = {'uniform', 'input', 'output', 'vec2', 'vec3', 'vec4', 'mat2', 'mat3', 'mat4', 'float'}
alphabet = 'abcdefghijklmnopqrstuvwxyz'
alphabet+= alphabet.upper()
alphabet+= '_0123456789'
ids = set()

def make_defines():
    return '\n'.join(['#define %s %s'%(v, k) for k,v in defines.items()])+'\n'

def compress_code(fname, rename=5):
    global defines, alphabet, qualifiers
    code = open(fname, 'r').read()
    #before = len(code)

    for line in code.split('\n'):
        words = line.split()
        if words:
            id = words[-1][:-1]
            f = sum([(i in alphabet) for i in id])==len(id)
            q = words[0] in qualifiers
            if id in defines.keys():
                continue
            if q and f:
                if len(id)>=rename:
                    rnd = 0
                    while rnd in ids:
                        rnd = randint(0, 255)
                    defines[id] = 'x' + hex(rnd)[2:].zfill(2)
                    ids.add(rnd)


    operators = '//', '=', '+', '-', '*', '+=', '-=', '*=', '/=', ',', '||', '&&', ';', '(', ')', '[', ']', '.', '{', '}', 'if', 'else'

    for op in operators:
        code = code.replace(op, ' '+op+' ')
        code = code.replace('\t', ' ')
        code = code.replace('  ', ' ')
        code = code.replace('\n\n', '\n')
        code = code.replace('\n ', '\n')

    words = code.split(' ')
    if rename:
        for i in range(len(words)):
            if words[i] in defines.keys():
                #print('replacing', words[i], defines[words[i]])
                words[i] = defines[words[i]]
        code = (' '.join(words))
    else:
        code = ' '.join(words)

    for op in operators:
        code = code.replace('\t', ' ').replace('  ', ' ').replace(op+' ', op).replace(' '+op, op).replace(op+'\n', op)
    code = code.replace('\r', '\n').replace('#', '\n#').replace('\n\n', '\n').replace('elseif', 'else if')

    #after = len(code)
    #compression_ratio = after*100/before
    return code

def save_text(text, fname):
    with open(fname, 'w') as file:
        file.write(text)
        file.close()

for i in sys.argv[1:]:
    fname = i
    if '/' in i:
        i = i[i.rfind('/')+1:]
    if '.' in i:
        i = i[:i.rfind('.')]
    code = compress_code(fname)
    #print('saving', 'data/shaders/compressed/%s.glz'%i)
    save_text(code, 'data/shaders/compressed/%s.glz'%i)
#print(compress_code(sys.argv[1:]))
save_text(make_defines(), 'data/shaders/compressed/defs.glsl')