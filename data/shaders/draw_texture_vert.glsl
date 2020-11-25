output vec2 tex_map;
uniform vec2 gSize;
uniform vec2 gPosition;
uniform vec2 gResolution;

void main()
{
    tex_map = uv;
    vec2 crd = uv*2.0 - 1.0;
    gl_Position = vec4((crd*gSize + gPosition)/gResolution, 0.0, 1.0);
}
