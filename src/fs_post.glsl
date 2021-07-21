in vec2 v_uv;

uniform sampler2D tex_col;
uniform vec2 pixel_size;

out vec4 frag_color;
void main() {
    float x = pixel_size.x * 0.25;
    float y = pixel_size.y * 0.25;
    frag_color = vec4(
            4.0 * texture(tex_col, v_uv).rgb +

            2.0 * texture(tex_col, v_uv + vec2( x,  0)).rgb +
            2.0 * texture(tex_col, v_uv + vec2(-x,  0)).rgb +
            2.0 * texture(tex_col, v_uv + vec2( 0, -y)).rgb +
            2.0 * texture(tex_col, v_uv + vec2( 0,  y)).rgb +

            1.0 * texture(tex_col, v_uv + vec2( x,  y)).rgb +
            1.0 * texture(tex_col, v_uv + vec2(-x,  y)).rgb +
            1.0 * texture(tex_col, v_uv + vec2(-x, -y)).rgb +
            1.0 * texture(tex_col, v_uv + vec2( x, -y)).rgb
    , 16.0) / 16.0;
}
