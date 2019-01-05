#version 130

in vec2 v_uv;
in vec4 v_color;

uniform sampler2D u_texture;

void main() {
    gl_FragColor = v_color * texture(u_texture, v_uv) * vec4(v_uv.x, v_uv.y, 1.0, 1.0);
}