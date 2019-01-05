#version 130

in vec4 v_color;
in vec2 v_uv;

uniform sampler2D sampler1;

out vec4 out_color;

void main() {
    vec4 color = texture(sampler1, v_uv);
    vec3 grey = vec3(0.299 * color.r + 0.587 * color.g + 0.114 * color.b);
    out_color = vec4(grey, color.a);
}