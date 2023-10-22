#version 430 core

out vec4 color;

struct Sphere {
    vec3 pos;
    float radius;
};

layout (std430, binding=2) buffer SphereBuffer
{
    Sphere spheres[];
};

void main()
{
    color = vec4(spheres[0].pos, 1.0);
}