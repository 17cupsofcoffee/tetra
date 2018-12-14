#version 130

in vec4 v_color;
in vec2 v_uv;

uniform sampler2D sampler1;

out vec4 out_color;

void main() {
    out_color = v_color * texture(sampler1, v_uv);
}