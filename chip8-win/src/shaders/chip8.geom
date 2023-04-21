#version 330

layout (points) in;
layout (triangle_strip, max_vertices = 4) out;

// Chip8 display resolution is known and fixed.
vec2 resolution = vec2(64.0, 32.0);

in float statev[];
out float state;

void build_quad(vec4 position) {
    // This mimics the transform of the matrix passed into the vertex shader.
    vec2 px = (1 / resolution) * 2;
    px.y *= -1;

    state = 1.0 * statev[0];
    gl_Position = position + vec4(0, 0, 0, 0);   // top-left
    EmitVertex();
    state = 1.0 * statev[0];
    gl_Position = position + vec4(px.x, 0, 0, 0);  // top-right
    EmitVertex();
    state = 1.0 * statev[0];
    gl_Position = position + vec4(0, px.y, 0, 0); // bottom-left
    EmitVertex();
    state = 1.0 * statev[0];
    gl_Position = position + vec4(px.x, px.y, 0, 0); // bottom-right
    EmitVertex();

    EndPrimitive();
}

void main() {
    build_quad(gl_in[0].gl_Position);
}
