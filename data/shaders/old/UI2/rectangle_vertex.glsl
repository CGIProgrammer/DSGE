output vec2 tex_map;
output vec2 position;
uniform float width;
uniform float height;
uniform float rect_width;
uniform float rect_height;
uniform vec2 uv_start, uv_size;
uniform mat3 transform;
uniform int layer;

void main()
{
    vec2 resolution = vec2(width,height);
    vec2 abs_pos = (vec3(uv*vec2(rect_width, rect_height), 1.0) * transform).xy;
    
    tex_map = uv*uv_size+uv_start;
    tex_map -= min(uv_size, vec2(0.0));
    gl_Position = vec4(abs_pos.xy*vec2(2.0,-2.0)/resolution + vec2(-1.0,1.0), 0.0, 1.0);
}
