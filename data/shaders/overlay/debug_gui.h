#ifndef DEBUG_GUI_GLSL
#define DEBUG_GUI_GLSL

#define font_mip 2
// Параметры для изображения 1024
ivec2 char_size = ivec2(64>>font_mip);
ivec2 debug_gui_coords = ivec2(0);
ivec2 debug_gui_resolution = ivec2(0);

void draw_cross(inout vec4 image, vec3 color)
{
    ivec2 halfres = debug_gui_resolution/2;
    ivec2 centered = debug_gui_coords - halfres;
    float vertical = float( centered.x == 0 && abs(centered.y) > 5 && abs(centered.y) < 20 );
    float horizontal = float( centered.y == 0 && abs(centered.x) > 5 && abs(centered.x) < 20 );
    float center = float(centered.x == 0 && centered.y == 0);
    float _cross = vertical + horizontal + center;
    image = mix(image, vec4(color, 1.0), _cross);
    //image = vec4(_cross);
}

float draw_char(int chr, ivec2 coord)
{
    int x0 = char_size.x * int(mod(chr, 16));
    int y0 = char_size.y * (chr / 16);
    int x1 = x0 + char_size.x;
    int y1 = y0 + char_size.y;
    ivec2 crd = debug_gui_coords + ivec2(x0, y0) - coord;
    float alpha = float(crd.x >= x0 && crd.x <= x1 && crd.y >= y0 && crd.y <= y1);
    float sampl = texelFetch(font, crd, font_mip).x;
    return alpha * sampl;
}

float draw_num(int val, ivec2 coord, int point)
{
    int len = 0;
    int digits[10] = int[](
        int(mod(val/1000000000, 10)),
        int(mod(val/100000000, 10)),
        int(mod(val/10000000, 10)),
        int(mod(val/1000000, 10)),
        int(mod(val/100000, 10)),
        int(mod(val/10000, 10)),
        int(mod(val/1000, 10)),
        int(mod(val/100, 10)),
        int(mod(val/10, 10)),
        int(mod(val, 10))
    );

    int first_digit_num = 0;
    for (int i=0; i<10; i++) {
        if (digits[i] > 0) {
            first_digit_num = i;
            break;
        }
    }
    
    first_digit_num = min(first_digit_num, 9 - point);

    int half_cs = char_size.x / 2;
    int quart_cs = half_cs / 2;
    float result = 0.0;
    ivec2 was_point = ivec2(0);
    for (int i = 0; i < 10 - first_digit_num; i++) {
        ivec2 delta = ivec2(i * half_cs, 0);
        result += draw_char(0x30 + digits[i + first_digit_num], coord + delta + was_point);
        if (9 - i - first_digit_num == point) {
            was_point.x = quart_cs;
            result += draw_char(46, coord + delta + ivec2(quart_cs + quart_cs/2, 0));
        }
    }

    return result;
}

float draw_float(float val, ivec2 crd, int point)
{
    int num = int(round(val * pow(10, point)));
    return draw_num(num, crd, point);
}

/*vec4 graph(float value, float mn, float mx, ivec2 location, ivec2 size, ivec2 resol)
{

    if (
        gl_FragCoord.xy.x >= location.x && gl_FragCoord.xy.y >= location.y && 
        gl_FragCoord.xy.x <= size.x + location.x && gl_FragCoord.xy.y <= size.y + location.y
    ) {
        ivec2 grid = ivec2(gl_FragCoord.xy);
        vec3 acc = texelFetch(accumulator_in, grid + ivec2(1, 0), 0).rgb;
        float prev_val = texelFetch(accumulator_in, location, 0).a;
        float range = mx - mn;
        value = 1.0 - (value - mn) / range;
        if (gl_FragCoord.xy == location) {
            accumulator_out.a = value;
        }
        if (gl_FragCoord.xy.x >= size.x + location.x - 1) {
            float curr, prev;
            if (prev_val > value) {
                curr = float(value <= float(gl_FragCoord.xy.y - location.y+1) / float(size.y));
                prev = float(prev_val >= float(gl_FragCoord.xy.y - location.y) / float(size.y));
            } else {
                curr = float(value >= float(gl_FragCoord.xy.y - location.y-1) / float(size.y));
                prev = float(prev_val <= float(gl_FragCoord.xy.y - location.y) / float(size.y));
            }
            acc = vec3(curr * prev);
        }
        return vec4(acc, 1.0);
    }
    return vec4(0.0);
}*/

float print(mat4 mat, ivec2 offset)
{
    float result = 0.0;
    for (int i=0; i<4; i++) {
        for (int j=0; j<4; j++) {
            result += draw_float(mat[i][j], offset + ivec2(j*char_size.x*6, i * char_size.y), 3);
        }
    }
    return result;
}

float print(vec4 mat, ivec2 offset)
{
    float result = 0.0;
    for (int i=0; i<4; i++) {
        result += draw_float(mat[i], offset + ivec2(i*char_size.x*6, 0), 3);
    }
    return result;
}

float print(vec3 mat, ivec2 offset)
{
    float result = 0.0;
    for (int i=0; i<3; i++) {
        result += draw_float(mat[i], offset + ivec2(i*char_size.x*6, 0), 3);
    }
    return result;
}

float print(vec2 mat, ivec2 offset)
{
    float result = 0.0;
    for (int i=0; i<2; i++) {
        result += draw_float(mat[i], offset + ivec2(i*char_size.x*6, 0), 3);
    }
    return result;
}

float print(float scalar, ivec2 offset)
{
    return draw_float(scalar, offset, 3);
}

/*vec4 overlay()
{
    mat4 light_transform = mat4(
        texelFetch(lights_data, ivec2(0, 0), 0),
        texelFetch(lights_data, ivec2(1, 0), 0),
        texelFetch(lights_data, ivec2(2, 0), 0),
        texelFetch(lights_data, ivec2(3, 0), 0)
    );

    mat3 sm_tr = mat3(
        1.0, 0.0, 0.0,
        0.0, 1.0, 0.0,
        0.0, 0.0, 1.0
    );
    sm_tr[0][0] *= textureSize(shadowmap, 0).x / iResolution.x;
    sm_tr[1][1] *= textureSize(shadowmap, 0).y / iResolution.y;
    
    vec2 gl_FragCoord.xy = gl_FragCoord.xyWp * vec2(1.0, -1.0) + vec2(0.0, 1.0);
    vec2 sm_coord = (vec3(gl_FragCoord.xy, 1.0) * inverse(sm_tr)).xy;
    
    float depth = texture(shadowmap, sm_coord).r;
    vec2 zrange = depth_range(light.projection);
    float znear = zrange.x;
    float zfar = zrange.y;
    depth = abs(linearize_depth(depth, znear, zfar));
    float threshold = sin(timer.uptime)*0.5+0.5;
    float bounds = float(sm_coord.x >= 0.0 && sm_coord.x < 1.0 && sm_coord.y >= 0.0 && sm_coord.y < 1.0);
    bounds *= min(1.0, .99 + timer.uptime);
    vec4 _output = vec4(depth.rrr / (1.0 + depth.rrr), bounds);
    
    float debug_depth = linearize_depth(texture(shadowmap, vec2(0.5)).r, znear, zfar);
    debug_depth = draw_float(zrange.x, ivec2(0), 2);
    _output = mix(_output, vec4(1.0, 1.0, 0.0, 1.0), print(light_transform, ivec2(0, 0)));
    return _output;
}*/

/*vec3 graph()
{
    float znear_text = draw_float(zrange.x, ivec2(0), 2);
    float zfar_text = draw_float(zrange.y, ivec2(0, char_size.y), 2);
    float d_text = draw_float(texelFetch(accumulator_in, ivec2(321, 241), 0).a, ivec2(0, char_size.y*2), 4);
    
    vec4 diagram = graph(timer.delta, 0.0, 0.01, ivec2(320, 240), ivec2(320, 240), textureSize(accumulator_in, 0));

    swapchain_out.rgb = pow(swapchain_out.rgb / (swapchain_out.rgb + 1.0), vec3(1.5));
    accumulator_out.rgb = swapchain_out.rgb = mix(swapchain_out.rgb, diagram.rgb, diagram.a);
    swapchain_out = mix(swapchain_out, vec4(0, 1, 0, 1), znear_text + zfar_text + d_text);
}*/

#endif