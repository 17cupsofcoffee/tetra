#version 330 core

in VertexData {
    vec4 color;
    vec2 uv;
} i;

uniform sampler2D sampler1;

out vec4 out_color;

void main() {
    float alpha = texture(sampler1, i.uv).r;
    if (alpha <= 0.0) {
        discard;
    }
    out_color = i.color * vec4(1.0, 1.0, 1.0, alpha);
}