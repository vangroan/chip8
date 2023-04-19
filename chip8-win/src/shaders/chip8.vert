#version 330

layout(location = 0) in vec2 position;
layout(location = 1) in float alpha;

out float statev;

uniform mat4 u_Matrix;

void main() {
    statev = alpha;
    gl_Position = u_Matrix * vec4(position, 0, 1);
}
