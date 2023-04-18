#version 330

// Foreground color of the 
uniform vec4 u_Color;

in float state;

out vec4 frag_color;

void main() {
    if (state < 0.1) {
        discard;
    }
    frag_color = u_Color * state;
}
