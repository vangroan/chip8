#version 330

layout(location = 0) in vec2 point;
layout(location = 1) in float alpha;

out float state;

void main() {
    state = alpha;
    gl_Position = vec4(point, 0, 0);
}
