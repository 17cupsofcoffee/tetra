#version 130

in vec4 in_pos_tex;
in vec4 in_color;

uniform mat4 projection;

out vec4 v_color;
out vec2 v_uv;

void main() {
    gl_Position = projection * vec4(in_pos_tex.xy, 0.0, 1.0);
    v_color = in_color;
    v_uv = in_pos_tex.zw;
}