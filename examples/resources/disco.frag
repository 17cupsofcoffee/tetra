#version 150

in vec2 v_uv;
in vec4 v_color;

uniform sampler2D u_texture;
uniform sampler2D u_overlay;
uniform float u_red;
uniform float u_green;
uniform float u_blue;

out vec4 o_color;

void main() {
    o_color = v_color * texture(u_texture, v_uv) * texture(u_overlay, v_uv) * vec4(u_red, u_green, u_blue, 1.0);
}