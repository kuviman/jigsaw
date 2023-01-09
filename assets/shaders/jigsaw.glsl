uniform mat3 u_model_matrix;
uniform mat3 u_view_matrix;
uniform mat3 u_projection_matrix;

varying vec2 v_uv;

#ifdef VERTEX_SHADER
attribute vec3 a_pos;
attribute vec2 a_uv;

void main() {
    v_uv = a_uv;
    vec3 pos = u_projection_matrix * u_view_matrix * u_model_matrix * vec3(a_pos.xy, 1.0);
    gl_Position = vec4(pos.xy, a_pos.z, pos.z);
}
#endif

#ifdef FRAGMENT_SHADER
uniform sampler2D u_texture;

void main() {
    vec4 texture_color = texture2D(u_texture, v_uv);
    texture_color.a = 1.0;
    gl_FragColor = texture_color;
}
#endif
