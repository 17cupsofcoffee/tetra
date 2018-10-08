#version 330 core

layout (location = 0) in vec4 in_pos_tex;
layout (location = 1) in vec4 in_color;

// TODO: Interface block?
uniform mat4 projection;

out VertexData {
    vec4 color;
    vec2 uv;
} o;

void main() {
    gl_Position = projection * vec4(in_pos_tex.xy, 0.0, 1.0);
    o.color = in_color;
    o.uv = in_pos_tex.zw;
}