in vec2 v_uv;

uniform sampler2D tex_col;
uniform sampler2D tex_white;

out vec4 frag_color;
void main() {
    frag_color = vec4(texture(tex_col, v_uv).rgb, 1.0) + vec4(texture(tex_white, v_uv).rgb, 1.0);
}
