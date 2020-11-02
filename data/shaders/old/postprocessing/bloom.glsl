#include "data/shaders/functions/extensions.glsl"

uniform sampler2D filtered,original;
uniform sampler2D gTextures;
uniform float width,height;
input vec2 tex_map;
float tm = 0.0;

float gauss(float x)
{
    const float a = sqrt(2.0*3.1415926535);
    return a * exp(-x*x/2.0);
}

vec3 bloom(float threshold)
{
    vec3 bl = vec3(0.0);
    float n=0;
    
    vec2 directions[] = vec2[]
    (
        vec2( 0.5,-1.0),
        vec2(-0.5,-1.0),
        vec2( 0.5, 1.0),
        vec2(-0.5, 1.0),
        vec2( 1.0, 0.5),
        vec2( 1.0,-0.5),
        vec2(-1.0, 0.5),
        vec2(-1.0,-0.5)
    );
    
    for (float lod=0.0;lod<8.0;lod+=1.0)
    {
        vec2 res = 1.0/vec2(width, width)*pow(2.0, lod);
        float x = gauss(length(res*directions[0]*0.7));
        for (int i=0;i<7;i++)
        {
            vec3 s = textureLod(filtered,tex_map+res*directions[i]*0.7, lod).rgb * x * lod;
            float l = length(s);
            if (l==0.0)
            {
                continue;
            }
            s = normalize(s);
            bl += s * max(l-threshold, 0);
        }
        n+=8*x * lod;
    }
    return bl/n;
}

vec3 blur()
{
    vec3 bl = vec3(0.0);
    float n=0;
    
    vec2 directions[] = vec2[]
    (
        vec2( 0.5,-1.0),
        vec2(-0.5,-1.0),
        vec2( 0.5, 1.0),
        vec2(-0.5, 1.0),
        vec2( 1.0, 0.5),
        vec2( 1.0,-0.5),
        vec2(-1.0, 0.5),
        vec2(-1.0,-0.5)
    );
    
    for (float lod=3.0;lod<9.0;lod+=1.0)
    {
        vec2 res = 1.0/vec2(width, width)*pow(2.0, lod);
        for (int i=0;i<7;i++)
        {
            vec3 s = textureLod(filtered,tex_map+res*directions[i]*0.7, lod).rgb * lod;
            bl += s;
        }
        n+=8*lod;
    }
    return bl/n;
}

vec3 reinhard(vec3 color)
{
    //vec3 averangeColor = textureLod(filtered, tex_map, 10.0).rgb;
    vec3 averangeColor = blur();
    float a = 0.5;
    float lum = dot(color, vec3(0.2126f, 0.7152f, 0.0722f));
    float avLum = dot(averangeColor, vec3(0.2126f, 0.7152f, 0.0722f));
    mat3 rgbMap = mat3( 0.4124, 0.3576, 0.1806,
                        0.2126, 0.7152, 0.0722,
                        0.0193, 0.1192, 0.9505);
    mat3 xyzMap = mat3( 3.2406,-1.5372,-0.4986,
                       -0.9689, 1.8758, 0.0415,
                        0.0557,-0.2040, 1.0570);
    float L = a/avLum*lum;
    float Ld = L*(1.0+L/2.25)/(1.0+L);
    vec3 XYZ = rgbMap*color;
    vec3 xyY = vec3( XYZ.x/(XYZ.x+XYZ.y+XYZ.z),
                     XYZ.y/(XYZ.x+XYZ.y+XYZ.z),
                     XYZ.y);
    
    float Y = XYZ.y * Ld;
    
    XYZ = vec3(Y/xyY.y*xyY.x, Y, Y/xyY.y*(1.0-xyY.x-xyY.y));
    color = xyzMap*XYZ;
    return color;
}

void main()
{
    float lum = dot(blur(), vec3(0.2126f, 0.7152f, 0.0722f));
    //lum += max(lum-1.2, 0.0);
    fragColor = texture(filtered, tex_map);
    vec3 bloomValue = bloom(5.0);
    float bloomLuminance = dot(bloomValue, vec3(0.2126f, 0.7152f, 0.0722f));
    fragColor.rgb += bloomValue;
    fragColor.rgb /= mix(1.0, 3.0*lum, 0.5);
    //fragColor.rgb *= 1.1;
    //fragColor.rgb *= pow(length(fragColor.rgb),0.25);
}
