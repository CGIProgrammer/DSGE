/*
 * De-noise filter for dynamic noise.
 * 
 * Takes 3 passes:
 *   1. circular blur (with optional "sharpening")
 *   2. Kuwahara + median filters:
 *       Averaging the results of these filters, suddenly,
 *       gives a better result than applying each of them individually!
 *       It was a very random idea, and it works!
 *   3. Motion detection and temporal filter:
 *       The averaging of the Kuwahara filter and the median filter
 *       would hardly give such a result without the temporal filter.
 */

#define PRESET_BALANCED
//#define SHARPENING 4.0

// For a weak noise
#ifdef PRESET_LITE
    #define NOISE_MULTIPLIER 0.2
    #define BLUR_RADIUS 2
    #define MEDIAN
    #define MEADIAN_FILTER_RADIUS 4
    #define KUWAHARA
    #define KUWAHARA_RADIUS 4
    // The lower the value, the less blur, but more noise. The higher the value, the more blur, but less noise.
    #define SENSITIVITY 50.0
    #define SEARCH_RADIUS 7
    #define SEARCH_KERNEL 3
    #define CONTRAST 1.0
#endif

// Basig settings
#ifdef PRESET_BALANCED
    #define NOISE_MULTIPLIER 0.6
    #define BLUR_RADIUS 2
    #define MEDIAN
    #define MEADIAN_FILTER_RADIUS 6
    #define KUWAHARA
    #define KUWAHARA_RADIUS 8
    
    #define SENSITIVITY 25.0
    #define SEARCH_RADIUS 8
    #define SEARCH_KERNEL 5
    #define CONTRAST 1.0
#endif

#ifdef PRESET_HIGH
    #define NOISE_MULTIPLIER 1.5
    #define BLUR_RADIUS 5
    #define MEDIAN
    #define MEADIAN_FILTER_RADIUS 10
    #define KUWAHARA
    #define KUWAHARA_RADIUS 10
    
    #define SENSITIVITY 15.0
    #define SEARCH_RADIUS 8
    #define SEARCH_KERNEL 5
    #define BRIGHTNESS 0.5
    #define CONTRAST NOISE_MULTIPLIER
#endif

// For extreemely strong noise!
#ifdef PRESET_ULTRA
    #define NOISE_MULTIPLIER 5.0
    #define BLUR_RADIUS 15
    #define MEDIAN
    #define MEADIAN_FILTER_RADIUS 15
    #define KUWAHARA
    #define KUWAHARA_RADIUS 15
    #define SENSITIVITY 3.0
    #define SEARCH_RADIUS 10
    #define SEARCH_KERNEL 5
    #define CONTRAST 5.0
#endif

// Motion detection parameters
//#define SEARCH_RADIUS 8
//#define SEARCH_KERNEL 5

// Value of contribution to the pixel difference when searching for a vector.
#define COLOR_IMPORTANCE 0.2

#define DETECT_MOTION_SNAKE_STEP 10
#define COMPARE_SNAKE_STEP 1

float pix_diff(vec3 a, vec3 b)
{
    vec3 lum_a = vec3(dot(a, vec3(1.0/3.0)));
    vec3 lum_b = vec3(dot(b, vec3(1.0/3.0)));
    a = mix(a, a/lum_a, COLOR_IMPORTANCE);
    b = mix(b, b/lum_b, COLOR_IMPORTANCE);
    return dot(pow(a - b, vec3(2.0)), vec3(1.0/3.0));
}

float compare_squares(sampler2D prev, ivec2 crd1, sampler2D curr, ivec2 crd2, int radius, float if_less)
{
    float result = 0.0;
    int n = 0;
    int square = radius*2+1;
    square *= square;
    int x=0,y=0;
    int dx=0,dy=-COMPARE_SNAKE_STEP;
    int check_counter = 0;
    for (n = 0; n < square; n++) {
        if ((-radius < x && x <= radius) && (-radius < y && y <= radius))
        {
            ivec2 offset = ivec2(x, y);
            vec3 a = texelFetch(prev, crd1+offset, 0).rgb;
            vec3 b = texelFetch(curr, crd2+offset, 0).rgb;
            float l = pix_diff(a, b);
            result += l;
            check_counter += 1;
            if (check_counter >= 6) {
                if (result / float(n - 1) > if_less) {
                    return if_less;
                }
                check_counter = 0;
            }
        }
        if (x == y || (x < 0 && x == -y) || (x > 0 && x == 1-y)) {
            int swap = dx;
            dx = -dy;
            dy =  swap;
        } else {
            x += dx;
            y += dy;
        }
    }
    return result / float(n - 1);
}


// Author https://www.shadertoy.com/view/lls3WM
void kuwahara( out vec4 fragColor, in vec2 fragCoord, sampler2D iChannel0, vec3 iResolution ) {
    vec2 iChannel0_size = iResolution.xy;
    vec2 uv = fragCoord.xy / iChannel0_size;
    const int radius = KUWAHARA_RADIUS;
    float n = float((radius + 1) * (radius + 1));

    vec3 m[4];
    vec3 s[4];
    for (int k = 0; k < 4; ++k) {
        m[k] = vec3(0.0);
        s[k] = vec3(0.0);
    }

    for (int j = -radius; j <= 0; ++j)  {
        for (int i = -radius; i <= 0; ++i)  {
            vec3 c = texture(iChannel0, uv + vec2(i,j) / iChannel0_size).rgb;
            m[0] += c;
            s[0] += c * c;
        }
    }

    for (int j = -radius; j <= 0; ++j)  {
        for (int i = 0; i <= radius; ++i)  {
            vec3 c = texture(iChannel0, uv + vec2(i,j) / iChannel0_size).rgb;
            m[1] += c;
            s[1] += c * c;
        }
    }

    for (int j = 0; j <= radius; ++j)  {
        for (int i = 0; i <= radius; ++i)  {
            vec3 c = texture(iChannel0, uv + vec2(i,j) / iChannel0_size).rgb;
            m[2] += c;
            s[2] += c * c;
        }
    }

    for (int j = 0; j <= radius; ++j)  {
        for (int i = -radius; i <= 0; ++i)  {
            vec3 c = texture(iChannel0, uv + vec2(i,j) / iChannel0_size).rgb;
            m[3] += c;
            s[3] += c * c;
        }
    }


    float min_sigma2 = 1e+2;
    for (int k = 0; k < 4; ++k) {
        m[k] /= n;
        s[k] = abs(s[k] / n - m[k] * m[k]);

        float sigma2 = s[k].r + s[k].g + s[k].b;
        if (sigma2 < min_sigma2) {
            min_sigma2 = sigma2;
            fragColor = vec4(m[k], 1.0);
        }
    }
}



// fastMedian from https://www.shadertoy.com/view/WdX3Wj
// Replaced readInput with texelFetchOffset
#define ADAPTIVE_QUANTIZATION
//#define BIN_COUNT 4
//#define BIN_COUNT 8
#define BIN_COUNT 12
//#define BIN_COUNT 24
//#define BIN_COUNT 48

//

#if BIN_COUNT == 4
	#define UNROLL(X) X(0)X(1)X(2)X(3)

#elif BIN_COUNT == 8
	#define UNROLL(X) X(0)X(1)X(2)X(3)X(4)X(5)X(6)X(7)

#elif BIN_COUNT == 12
	#define UNROLL(X) X(0)X(1)X(2)X(3)X(4)X(5)X(6)X(7)X(8)X(9)X(10)X(11)

#elif BIN_COUNT == 24
	#define U00_11(X) X(0)X(1)X(2)X(3)X(4)X(5)X(6)X(7)X(8)X(9)X(10)X(11)
	#define U12_23(X) X(12)X(13)X(14)X(15)X(16)X(17)X(18)X(19)X(20)X(21)X(22)X(23)
	#define UNROLL(X) U00_11(X)U12_23(X)
            
#elif BIN_COUNT == 48
	#define U00_11(X) X(0)X(1)X(2)X(3)X(4)X(5)X(6)X(7)X(8)X(9)X(10)X(11)
	#define U12_23(X) X(12)X(13)X(14)X(15)X(16)X(17)X(18)X(19)X(20)X(21)X(22)X(23)
	#define U24_35(X) X(24)X(25)X(26)X(27)X(28)X(29)X(30)X(31)X(32)X(33)X(34)X(35)
	#define U36_47(X) X(36)X(37)X(38)X(39)X(40)X(41)X(42)X(43)X(44)X(45)X(46)X(47)
	#define UNROLL(X) U00_11(X)U12_23(X)U24_35(X)U36_47(X)
            
#endif


void fastMedian( out vec4 fragColor, in vec2 fragCoord, sampler2D iChannel0, vec3 iResolution )
{
    fragColor = texelFetch(iChannel0, ivec2(fragCoord), 0);
#ifndef RADIUS
    return;
#else
    // Fit image to touch screen from outside
    vec2 img_res = iChannelResolution[0].xy;
    vec2 res = iResolution.xy / img_res;
    vec2 img_size = img_res * max(res.x, res.y);
    vec2 img_org = 0.5 * (iResolution.xy - img_size);
    ivec2 uv = ivec2(fragCoord - img_org);

    vec3 ocol = texelFetchOffset(iChannel0, uv, 0, ivec2(0));
    vec3 col = ocol;
    
    const int r = MEADIAN_FILTER_RADIUS;
    
	vec4 bins[BIN_COUNT];
	#define INIT(n) bins[n] = vec4(0);
    UNROLL(INIT)

#ifdef ADAPTIVE_QUANTIZATION        
	float vmin = 1.0;
	float vmax = 0.0;

	for (int y = -r; y <= r; y++)
	for (int x = -r; x <= r; x++)
	{
        vec3 img = texelFetchOffset(iChannel0, uv, 0, ivec2(x, y));
		float v = (img.r + img.g + img.b) / 3.0;

		vmin = min(vmin, v);
		vmax = max(vmax, v);
	}
    
#else
   	float vmin = 0.0;
	float vmax = 1.0;
    
#endif

	for (int y = -r; y <= r; y++)
	for (int x = -r; x <= r; x++)
	{
        vec3 img = texelFetchOffset(iChannel0, uv, 0, ivec2(x, y));
		float v = (img.r + img.g + img.b) / 3.0;

		int i = int(0.5 + ((v - vmin) / (vmax - vmin)) * float(BIN_COUNT));

		#define UPDATE(n) if (i == n) bins[n] += vec4(img.rgb, 1.0);
        UNROLL(UPDATE)
	}
    
	float mid = floor((float(r * 2 + 1) * float(r * 2 + 1)) / 2.0);
	float pos = 0.0;

    #define M1(i) col.rgb = pos <= mid && bins[i].a > 0.0 ?
    #define M2(i) bins[i].rgb / bins[i].aaa : col.rgb;
    #define M3(i) pos += bins[i].a;
    #define MEDIAN(i) M1(i)M2(i)M3(i)
    UNROLL(MEDIAN)

    // Show original image on click
    //if (iMouse.w > 0.0) col = ocol;
        
    fragColor = vec4(col, 1.0);
#endif
}

vec3 blur(sampler2D img, ivec2 crd, int radius)
{
    vec3 col = vec3(0.0);
    float att = 0.0;
    for (int i=-radius; i<=radius; i++)
    for (int j=-radius; j<=radius; j++) {
        float w = inversesqrt(float(i*i + j*j)+1.0) * float(radius);
        w = 1.0 / clamp(1.0/w, 0.0, 1.0);
        w *= w;
        col += texelFetch(img, crd + ivec2(i, j), 0).rgb * w;
        att += w;
    }
    
    return col / att;
}