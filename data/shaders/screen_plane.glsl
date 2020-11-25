output vec2 tex_map;
output vec2 screen_map;

uniform vec2 gResolution;

void main()
{
    tex_map = uv;
    screen_map = uv * gResolution;
    gl_Position = vec4(pos.xy, 0.0, 1.0);
}
