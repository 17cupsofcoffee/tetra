attribute vec2 a_position;
attribute vec2 a_uv;
attribute vec4 a_color;

uniform mat4 u_projection;

varying vec2 v_uv;
varying vec4 v_color;

void main() {
    v_color = a_color;
    v_uv = a_uv;

    gl_Position = u_projection * vec4(a_position, 0.0, 1.0);
}