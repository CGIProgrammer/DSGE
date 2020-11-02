#include "data/shaders/functions/extensions.glsl"

output vec2 tex_map;
output vec2 tex_map2;
output vec3 normals;
output float r,depth;
output mat3 TBN;
output vec3 viewDirTBN,viewDir;

output vec4 position;
output vec3 viewPos;

uniform int render_projection;
uniform mat4 camera_inverted;
uniform mat4 projection,camera_transform;
uniform float width,height,zFar,zNear,Angle,material_displacement;

#if MAX_INSTANCES==1
uniform mat4 transform;
#else
uniform mat4 transform[MAX_INSTANCES];
#endif

void main(void)
{
    #if MAX_INSTANCES==1
    mat4 model = mat3x4to4(transform);
    #else
    mat4 model = mat3x4to4(transform[gl_InstanceID]);
    #endif

    viewPos = transpose(camera_inverted)[3].xyz;
    mat4 modelWiew = model*mat3x4to4(camera_inverted);
    float nLength = length(nor);
    vec3 nnor = normalize(nor);
    if (nLength>1.1)
        TBN = transpose(mat3(tang, normalize(cross(nor, tang)), nor))*mat3(model[0].xyz,model[1].xyz,model[2].xyz);
    else
        TBN = transpose(mat3(tang, normalize(cross(tang, nor)), nor))*mat3(model[0].xyz,model[1].xyz,model[2].xyz);
    
    tex_map  = vec2(uv.x, 1.0-uv.y);
    tex_map2 = vec2(uv2.x,1.0-uv2.y);

    normals = transpose(TBN)[2].xyz;

    position.xyz = (vec4(pos,1.0)*model).xyz;
    vec4 finalPosition = vec4(position.xyz, 1.0)*mat3x4to4(camera_inverted);
    position.xyz += normals.xyz * material_displacement * abs(finalPosition.z);

    viewDir = position.xyz-transpose(camera_transform)[3].xyz;
    viewDirTBN = TBN*viewDir;
    
    position = vec4(position.xyz, 1.0)*mat3x4to4(camera_inverted);
    depth = position.z;
    gl_Position = position*mat3x4to4(projection);
    position = gl_Position;

    /*position = vec4(pos, 1.0)*(modelWiew*mat3x4to4(projection));
    gl_Position = position;
    depth = position.z;*/
}
