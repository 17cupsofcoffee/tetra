#ifdef GL_ES
    #define LOWP lowp
    precision mediump float;
#else
    #define LOWP
#endif

varying vec2 v_uv;
varying LOWP vec4 v_color;

uniform sampler2D u_texture;

void main() {
    gl_FragColor = v_color * texture2D(u_texture, v_uv);
}