output vec2 tex_map;
uniform float width, height;

void main()
{
    tex_map = uv;
    gl_Position = vec4(pos.xy, 0.0, 1.0);
}
