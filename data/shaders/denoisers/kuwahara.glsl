#include "data/shaders/functions/extensions.glsl"

input vec2 tex_map;
uniform sampler2D gLighting, gAlbedo, gSpace, gOutput, gMasks;
uniform vec2 gResolution;
uniform int gFilterPass;

#define ACTIVATE_FILTER 1
#define SLOPE_FILTER 1
#define X_RANGE 1.0
#define Y_RANGE	1.0

float GetPixelAngle(vec2 uv)
{
    float GradientX = 0.0;
    float GradientY = 0.0;
    
    float SobelX[9] = float[9](-1.0, -2.0, -1.0, 
        				0.0, 0.0, 0.0, 
        				1.0, 2.0, 1.0);
        
    float SobelY[9] = float[9](-1.0, 0.0, 1.0,
        			   -2.0, 0.0, 2.0,
        			   -1.0, 0.0, 1.0);
        
    int i = 0;
    
    for (float x = -1.0; x <= 1.0; x++)
    {
        for (float y = -1.0; y <= 1.0; y++)
        {
            vec2 offset = vec2(x,y);
            vec2 Coords = uv + offset;
            vec3 PixelColor = texture(gOutput, Coords).rgb;
            float PixelValue = dot(PixelColor, vec3(0.3, 0.59, 0.11));
            
            GradientX += PixelValue * SobelX[i];
            GradientY += PixelValue * SobelY[i];
            i++;
        }
    }
    
    return atan(GradientY/GradientX);
}

vec4 GetKernelMeanAndVariance(vec2 uv, vec4 Range, mat2 RotationMatrix)
{
    vec3 Mean = vec3(0.0);
    vec3 Variance = vec3(0.0);
    float Samples = 0.0;
    
    for (float x = Range.x; x <= Range.y; x++)
    {
        for (float y = Range.z; y <= Range.w; y++)
        {            
            vec2 offset = vec2(0.0);
            
            #if SLOPE_FILTER
            offset = vec2(x,y) * RotationMatrix;
            #else
            offset = vec2(x,y);
            #endif
            
            vec2 Coords = (uv + offset) / gResolution;
            vec3 PixelColor = texture(gOutput, Coords).rgb;
            
            Mean+= PixelColor;
            Variance += PixelColor * PixelColor;
            Samples++;
        }
    }
    
    Mean /= Samples;
    Variance = Variance / Samples - Mean * Mean;
    
    float TotalVariance = Variance.r + Variance.g + Variance.b;
    return vec4(Mean.r, Mean.g, Mean.b, TotalVariance);
}

vec3 KuwaharaFilter(vec2 uv)
{    
    vec4 MeanAndVariance[4];
    vec4 Range;
    
    float Angle = GetPixelAngle(uv);
    mat2 RotationMatrix = mat2(cos(Angle), -sin(Angle),
                               sin(Angle), cos(Angle));
    
    Range = vec4(-X_RANGE, 0.0, -Y_RANGE, 0);
    MeanAndVariance[0] = GetKernelMeanAndVariance(uv, Range, RotationMatrix);
    
    Range = vec4(0.0, X_RANGE, -Y_RANGE, 0.0);
    MeanAndVariance[1] = GetKernelMeanAndVariance(uv, Range, RotationMatrix);
    
    Range = vec4(-X_RANGE, 0.0, 0.0, Y_RANGE);
    MeanAndVariance[2] = GetKernelMeanAndVariance(uv, Range, RotationMatrix);
    
    Range = vec4(0.0, X_RANGE, 0.0, Y_RANGE);
    MeanAndVariance[3] = GetKernelMeanAndVariance(uv, Range, RotationMatrix);
    
    vec3 FinalColor = MeanAndVariance[0].rgb;
    float MinVariance = MeanAndVariance[0].a;
    
    for (int i = 1; i < 4; i++)
    {
       	if (MeanAndVariance[i].a < MinVariance)
        {
            FinalColor = MeanAndVariance[i].rgb;
            MinVariance = MeanAndVariance[i].a;
        }
    }
    
    return FinalColor;
}

void main() {
    //vec2 pass = float(gFilterPass/2 == (gFilterPass+1)/2) / gResolution;
    if (texture(gAlbedo, tex_map).a < 0.5)
    {
      fragColor = texture(gOutput, tex_map);
      return;
    }
    
    fragColor = vec4(KuwaharaFilter(tex_map * gResolution), 1.0);
}
