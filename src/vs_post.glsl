in vec2 position;

out vec2 v_uv;

void main() {
  v_uv = position + vec2(0.5, 0.5);
  gl_Position = vec4(position, 0.0, 1.0);
}
