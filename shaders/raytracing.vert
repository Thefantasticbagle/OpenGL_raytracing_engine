#version 440 core

in vec3 position;

// The main function
void main() {
    gl_Position = vec4(position, 1.0f);
}