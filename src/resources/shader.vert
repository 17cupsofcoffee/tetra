#version 130

in vec4 a_pos_tex;
in vec4 a_color;

uniform mat4 u_projection;

out vec2 v_uv;
out vec4 v_color;

void main() {
    v_color = a_color;
    v_uv = a_pos_tex.zw;

    gl_Position = u_projection * vec4(a_pos_tex.xy, 0.0, 1.0);
}