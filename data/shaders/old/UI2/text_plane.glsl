output vec2 tex_map;
uniform vec2 char_position;
uniform float char_width;
uniform float char_height;
uniform float width;
uniform float height;
uniform int char_index;
uniform mat3 transform;
uniform int layer;

vec3 position;

void main()
{
    vec2 resolution = vec2(width,height);
    int dx = char_index / 16;
    int dy = char_index - dx*16;
    tex_map.x = (pos.x + float(dx))/16.0;
    tex_map.y = (pos.y + float(dy))/16.0;
    
    position = vec3(pos.xy*vec2(char_width, char_height) + char_position, 1.0);
    position*= transform;
    
    gl_Position = vec4(position.xy/resolution*vec2(2.0,-2.0) + vec2(-1.0,1.0), 0.0, 1.0);
    
}
