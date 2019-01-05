#version 130

in vec2 v_uv;
in vec4 v_color;

uniform sampler2D u_texture;

void main() {
    vec4 color = texture(u_texture, v_uv);
    vec3 grey = vec3(0.299 * color.r + 0.587 * color.g + 0.114 * color.b);

    gl_FragColor = vec4(grey, color.a);
}