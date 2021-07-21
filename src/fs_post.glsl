in vec2 v_uv;

uniform sampler2D tex_col;
uniform vec2 pixel_size;

out vec4 frag_color;
void main() {
    float x = pixel_size.x;
    float y = pixel_size.y;
    frag_color = vec4(
            4.0 * texture(tex_col, v_uv).rgb +

            1.0 * texture(tex_col, v_uv + vec2( x,  0)).rgb +
            1.0 * texture(tex_col, v_uv + vec2(-x,  0)).rgb +
            1.0 * texture(tex_col, v_uv + vec2( 0, -y)).rgb +
            1.0 * texture(tex_col, v_uv + vec2( 0,  y)).rgb +

            0.0 * texture(tex_col, v_uv + vec2( x,  y)).rgb +
            0.0 * texture(tex_col, v_uv + vec2(-x,  y)).rgb +
            0.0 * texture(tex_col, v_uv + vec2(-x, -y)).rgb +
            0.0 * texture(tex_col, v_uv + vec2( x, -y)).rgb
    , 8.0) / 8.0;

    frag_color *= min(v_uv.y, 0.1) * (7.0 + 3.0);
}
