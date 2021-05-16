#version 150

in vec2 a_position;
in vec2 a_uv;
in vec4 a_color;

uniform mat4 u_projection;
uniform vec2 u_offsets[256];

out vec2 v_uv;
out vec4 v_color;

void main() {
    v_color = a_color;
    v_uv = a_uv;

    vec2 offset = u_offsets[gl_InstanceID];

    gl_Position = u_projection * vec4(a_position + offset, 0.0, 1.0);
}