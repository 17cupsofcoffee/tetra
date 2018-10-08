#version 330 core

in VertexData {
    vec4 color;
    vec2 uv;
} i;

uniform sampler2D sampler1;

out vec4 out_color;

void main() {
    out_color = i.color * texture(sampler1, i.uv);
}