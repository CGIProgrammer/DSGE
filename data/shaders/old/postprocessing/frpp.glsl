#include "data/shaders/functions/extensions.glsl"

uniform sampler2D rendered;
uniform sampler2D spheremap;
uniform samplerCube cubemap;
uniform float width,height;
input vec2 tex_map;
uniform mat4 camera_inverted, projection_inv;

float gaussian(float x,float sig)
{
    return 1.0/(sig*2.5066282745951782*exp(x*x/(2.0*sig*sig)));
}

void main()
{
    vec4 skybox = vec4((tex_map*2.0-1.0)*5.0,0.0,0.0) * projection_inv;
    skybox.zw = vec2(-5.0, 1.0);
    skybox *= mat4(vec4(camera_inverted[0].x,camera_inverted[1].x, camera_inverted[2].x, 0.0),
                   vec4(camera_inverted[0].y,camera_inverted[1].y, camera_inverted[2].y, 0.0),
                   vec4(camera_inverted[0].z,camera_inverted[1].z, camera_inverted[2].z, 0.0),
                   vec4(0.0,0.0,0.0,1.0));
                   
    skybox.xyz = normalize(skybox.xzy);
    skybox.z *= -1.0;
    skybox.rgb = textureSpheremap(spheremap, skybox.xyz).rgb;
    
    vec4 color = texture(rendered,tex_map);
    fragColor.rgb = color.rgb; //mix(skybox.rgb, color.rgb, 1.0);
}
