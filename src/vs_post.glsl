#if 0
in vec2 position;

out vec2 v_uv;

void main() {
  v_uv = position + vec2(0.5, 0.5);
  gl_Position = vec4(position * 2.0, 0.0, 1.0);
}
#endif

out vec2 v_uv;

vec2[4] CO = vec2[](
  vec2(-1., -1.),
  vec2( 1., -1.),
  vec2( 1.,  1.),
  vec2(-1.,  1.)
);

void main() {
  vec2 p = CO[gl_VertexID];

  gl_Position = vec4(p, 0., 1.);
  v_uv = p * .5 + .5;
}
