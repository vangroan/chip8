#version 330

layout(location = 0) in vec2 position;
layout(location = 1) in float alpha;

vec2 resolution = vec2(64.0, 32.0);

out float statev;

void main() {
    statev = alpha;

    // Normalize the position from chip8 pixels to 0.0 to 1.0
    vec2 norm_position = position / resolution;

    // Convert from normalized position (0,+1) to clip space is (-1,+1)
    vec2 clip_position = norm_position * 2 - 1;

    // Chip8 y increases downwards.
    clip_position.y *= -1;

    gl_Position = vec4(clip_position, 0, 1);
}
