#version

layout (points) in;
layout (triangle_strip, max_vertices = 4) out;

in vec4 colorv[];
out vec4 color;

void main() {
    gl_Position = gl_in[0].gl_Position + vec4(-0.1, 0.0, 0.0, 0.0); 
    EmitVertex();

    gl_Position = gl_in[0].gl_Position + vec4( 0.1, 0.0, 0.0, 0.0);
    EmitVertex();

    gl_Position = gl_in[0].gl_Position + vec4( 0.1, 1.0, 0.0, 0.0);
    EmitVertex();

    gl_Position = gl_in[0].gl_Position + vec4(-0.1, 1.0, 0.0, 0.0);
    EmitVertex();
    
    EndPrimitive();
}