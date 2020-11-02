uniform float width,height;
output vec2 tex_map;
output vec2 resolution; 
output vec2 pixel_coord;
output vec3 normals,position;

void main()
{
    resolution = vec2(width,height);
    tex_map = uv;
    normals = nor;
    pixel_coord = tex_map;
    pixel_coord.x *= width;
    pixel_coord.y *= height;
    position = pos;
    gl_Position = vec4(pos, 1.0);
}
