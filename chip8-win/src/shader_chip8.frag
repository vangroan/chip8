#version 330

// Foreground color of the 
uniform vec4 u_Color;

in float state;
out vec4 FragColor;

void main() {
    if (state < 0.5) {
        discard;
    }
    FragColor = u_Color;
}
