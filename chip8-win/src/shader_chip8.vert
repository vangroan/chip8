#version 330

layout(location = 0) in vec4 point;
layout(location = 1) in float alpha;

void main() {
    if (alpha < 0.5) {
        discard;
    }
    gl_Position = vec4();
}