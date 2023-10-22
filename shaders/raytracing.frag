#version 440 core

out vec4 color;

// RTSettings
struct Settings {
    uint maxBounces;
    uint raysPerFrag;
    float divergeStrength;
    float focusDistance;
};

// RTMaterial
struct Material {
    vec4 color;
    vec4 emissionColor;
    vec4 specularColor;
    float smoothness;
};

// RTSphere
struct Sphere {
    vec3 center;
    float radius;
    Material material;
};

// Uniform for holding settings
uniform Settings settings;

// Buffer for holding sphere objects
layout (std430, binding=0) buffer SphereBuffer
{
    Sphere spheres[];
};

// The main function
void main()
{
    color = vec4(spheres[1].material.specularColor);
}