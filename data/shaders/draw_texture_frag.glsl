uniform sampler2D gTexture;
input vec2 tex_map;

void main()
{
    fragColor = texture(gTexture, tex_map);
}