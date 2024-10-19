from math import exp, sqrt, log
import math

def gaussianKernel(sigma, size):
    sum = 0.0
    kernel = [0 for _ in range(size)]
    for i in range(size):
        x = float(i)
        weight = exp(-x * x / (2.0 * sigma * sigma)) / (sigma * sqrt(2.0 * math.pi))
        kernel[i] = weight
        sum += weight

    return kernel

def gaussian_kernel_size(sigma):
    return round(1.4142135623731*sigma*5)

def gaussian_kernel_sigma(kernel_size):
    sigma = kernel_size / (1.4142135623731*3)
    return sigma

if __name__ == '__main__':
    kernel_radius = 3
    sigma = gaussian_kernel_sigma(kernel_radius)
    sigma = 8
    print('#define SIGMA', sigma)
    kernel = gaussianKernel(sigma, kernel_radius)
    kernel_offsets = []
    kernel_weight = []
    min_weight = kernel[0] * kernel[-1]
    for i, y in enumerate(kernel[:0:-1] + kernel):
        for j, x in enumerate(kernel[:0:-1] + kernel):
            w = x*y
            if w >= min_weight:
                kernel_weight.append(x * y)
                kernel_offsets.append((i-len(kernel), j-len(kernel)))
    print(f'#define KERNEL_ARRAY_SIZE', len(kernel_weight))
    print(f'const float kernel_weights[{len(kernel_weight)}] = float[](')
    for i in range(0, len(kernel_weight), 6):
        print('   ', ', '.join(str(i) for i in kernel_weight[i:i+6]), ',', sep='')
    print(');')
    print(f'const ivec2 kernel_offsets[{len(kernel_weight)}] = ivec2[](')
    for i in range(0, len(kernel_offsets), 6):
        print('   ', ', '.join(f'ivec2({i}, {j})' for i, j in kernel_offsets[i:i+6]), ',', sep='')
    print(');')
    print(sum(kernel_weight), len(kernel_weight))
    # print(kernel_offsets)
    # print(kernel_weight)
    # print(size, sum(kernel))
