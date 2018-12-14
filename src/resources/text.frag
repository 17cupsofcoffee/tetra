#version 130

in vec4 v_color;
in vec2 v_uv;

uniform sampler2D sampler1;

out vec4 out_color;

void main() {
    float alpha = texture(sampler1, v_uv).r;
    if (alpha <= 0.0) {
        discard;
    }
    out_color = v_color * vec4(1.0, 1.0, 1.0, alpha);
}
