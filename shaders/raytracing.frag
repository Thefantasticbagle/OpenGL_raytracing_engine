#version 440 core

// --- Macros ---
#define HitInfo0 HitInfo( false, 0.0, vec3(0), vec3(0), Material(vec4(0), vec4(0), vec4(0), 0.0) )

// --- Constants ---
const float PI = 3.1415926;
const vec3  sunPosition = vec3(0, 0, 0);

// --- Structs ---

// RTSettings
struct Settings {
    uint maxBounces;
    uint raysPerFrag;
    float divergeStrength;
};

// RTCamera
struct Camera {
    vec2 screenSize;
    float fov;
    float focusDistance;
    vec3 pos;
    mat4 localToWorld;
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

// Hit information
struct HitInfo {
    bool        didHit;
    float       dist;
    vec3        pos;
    vec3        normal;
    Material    material;
};

// Ray
struct Ray {
    vec3 origin;
    vec3 dir;
};

// --- Inputs / outputs ---
out vec4 color;

uniform Settings settings;  // Raytracing settings
uniform Camera camera;      // Raytracing camera variables
uniform int spheresCount;

// Buffer for holding sphere objects
layout (std430, binding=0) buffer SphereBuffer
{
    Sphere spheres[];
};

// --- Randomness functions ---

// www.pcg-random.org, www.shadertoy.com/view/XlGcRh
/**
 * Generates a psuedo-random unsigned integer with value [0, 2^32 - 1].
 *
 * @param seed The seed, which is changed after use.
 * @return A psuedo-random unsigned integer.
 */
uint randInt(inout uint seed) {
    seed = seed * 747796405 + 2891336453;
    uint result = ((seed >> ((seed >> 28) + 4)) ^ seed) * 277803737;
    result = (result >> 22) ^ result;
    return result;
}

/**
 * Generates a psuedo-float with value [0, 1].
 *
 * @param seed The seed, which is changed after use. 
 * @return A psuedo-random float.
 */
float randFloat(inout uint seed) {
    return randInt(seed) / 4294967295.0; // 2^32 - 1
}

// https://stackoverflow.com/a/6178290
/**
 * Generates a normal-distributed psuedo-random float.
 *
 * @param seed The seed, which is changed after use. 
 * @return A normal-distributed psuedo-random float.
 */
float randFloatNormDist(inout uint seed) {
    float theta = 2 * PI * randFloat(seed);
    float rho = sqrt(abs(-2 * log(randFloat(seed))));
    return rho * cos(theta);
}

/**
 * Generates a normal-distributed psuedo-random 2D vector.
 *
 * @param seed The seed, which is changed after use. 
 * @return A normal-distributed psuedo-random vec2 for use in polar spaces.
 */
vec3 randVecNormDist(inout uint seed) {
    float x = randFloatNormDist(seed),
            y = randFloatNormDist(seed),
            z = randFloatNormDist(seed);
    return normalize(vec3(x, y, z));	
}

/**
 * Geneates a normal-distributed psuedo-random 2D vector.
 * While randVecNormDist() generates a normal-distribution for polar coordinates, this function does so for a square (cartesian space). 
 *
 * @param seed The seed, which is changed after use. 
 * @return A normal-distributed psuedo-random vec2 for use in cartesian spaces.
 */
vec2 randVecCartesianNormDist(inout uint seed) {
    float ang = randFloat(seed) * 2 * PI;
    vec2 pos = vec2(cos(ang), sin(ang));
    return pos * sqrt(abs(randFloatNormDist(seed))); // Normal distribution
}

// --- Environment functions ---
/**
 * Gets the environment light where a ray goes.
 *
 * @param ray The ray.
 * @return The environment light for the ray. 
 */
vec3 GetEnvironmentLight(Ray ray) {
    // Set up environment
    // TODO: Move these to another place.
    vec3 	SkyColourHorizon = vec3(1,0,0),
            SkyColourZenith = vec3(1,1,0),
            GroundColour = vec3(0,0,0);
    float   SunFocus = 10.f,
            SunIntensity = 5.f;
    
    // Calculate gradients
    float skyGradientT = pow(smoothstep(0, 0.4, ray.dir.y), 0.35);
    float groundToSkyT = smoothstep(-0.01, 0, ray.dir.y);
    vec3 skyGradient = mix(SkyColourHorizon, SkyColourZenith, skyGradientT);
    float sun = pow(max(0, dot(ray.dir, sunPosition)), SunFocus) * SunIntensity;

    // Combine ground, sky, and sun, and return the final color
    return mix(GroundColour, skyGradient, groundToSkyT) + sun * int(groundToSkyT>=1);
}

// --- Ray intersection functions ---
/**
 * Checks for an intersection between a ray and a sphere.
 *
 * @param ray The ray.
 * @param sphere The sphere.
 *
 * @return The hit information from the (possible) intersection.
 */
HitInfo RaySphere(Ray ray, Sphere sphere) {	
    HitInfo hitInfo = HitInfo0;
    vec3 offsetRayOrigin = ray.origin - sphere.center;

    // Solve for distance with a quadratic equation
    float a = dot(ray.dir, ray.dir);
    float b = 2 * dot(offsetRayOrigin, ray.dir);
    float c = dot(offsetRayOrigin, offsetRayOrigin) - sphere.radius*sphere.radius;

    // Quadratic discriminant
    float discriminant = b * b - 4 * a * c; 

    // If d > 0, the ray intersects the sphere => calculate hitinfo
    if (discriminant >= 0) {
        float dist = (-b - sqrt(abs(discriminant))) / (2 * a);

        // (If the intersection happens behind the ray, ignore it)
        if (dist >= 0) {
            hitInfo.didHit = true;
            hitInfo.dist = dist;
            hitInfo.pos = ray.origin + ray.dir * dist;
            hitInfo.normal = normalize(hitInfo.pos - sphere.center);
        }
    }

    // Otherwise, ray does not intersect sphere => return blank hitinfo
    return hitInfo;
}

// --- Raytracing functions ---
/**
 * Gets the first intersection which the ray might make.
 *
 * @param ray The ray.
 * @return The hit information from the (possible) intersection.
 */
HitInfo CalculateRayCollision(Ray ray) {
    HitInfo closestHit = HitInfo0;
    closestHit.dist = -1;

    // Iterate primitives, checking for collisions
    for (int i = 0; i < spheresCount; i++)
    {
        Sphere sphere = spheres[i];
        HitInfo hitInfo = RaySphere(ray, sphere);
        if (hitInfo.didHit && ( closestHit.dist < 0 || hitInfo.dist < closestHit.dist ) )
        {
            closestHit = hitInfo;
            closestHit.material = sphere.material;
        }
    }

    // Return the collision which occured closest to the origin
    return closestHit;
}

/**
 * Traces a ray's path as it bounces around the scene, collecting hit information along the way.
 *
 * @param ray The ray.
 * @param seed The seed, which is changed after use.
 *
 * @return The end color of the ray.
 */
vec3 Trace(Ray ray, inout uint seed) {
    vec3 	incomingLight = vec3(0),
            rayColor = vec3(1);
    bool	hitAny = false;
    
    for (int i = 0; i < settings.maxBounces; i++)
    {
        HitInfo hitInfo = CalculateRayCollision(ray);
        if (hitInfo.didHit)
        {
            hitAny = true;
            Material material = hitInfo.material;
            
            // Calculate new pos and dir
            ray.origin = hitInfo.pos;

            bool 	isSpecular  = material.specularColor.w >= randFloat(seed);
            vec3 	specularDir = reflect(ray.dir, hitInfo.normal),
                    diffuseDir  = normalize(hitInfo.normal + randVecNormDist(seed));
            ray.dir = normalize(mix(diffuseDir, specularDir, material.smoothness * int(isSpecular)));

            // Update light and color
            vec3 emittedLight = material.emissionColor.xyz * material.emissionColor.w;
            incomingLight += emittedLight * rayColor;
            rayColor *= mix(material.color, material.specularColor, isSpecular);

            // Early exit if ray color ~= 0
            // (Use some randomness to avoid "artificial" look)
            float p = max(rayColor.r, max(rayColor.g, rayColor.b));
            if (randFloat(seed) >= p) break;
            rayColor *= 1.0f / p;
        } else 
        {
            // If the ray did not hit anything, sample color from environment and return
            incomingLight += GetEnvironmentLight(ray) * rayColor;
            break;
        }
    }

    // (Return)
    return incomingLight;
}

// The main function
void main()
{
    // Create seed for RNG
    vec2 uv = vec2( gl_FragCoord.x / camera.screenSize.x, gl_FragCoord.y / camera.screenSize.y );
    uint i = uint( gl_FragCoord.y * camera.screenSize.x + gl_FragCoord.x );
    uint seed = i; //+ Frame * 719393; // TODO: Add frame counter so RNG changes over time

    // Calculate focus point
    float   planeHeight = camera.focusDistance * tan(camera.fov * 0.5 * PI / 180.0) * 2.0,
            planeWidth = planeHeight * (camera.screenSize.x / camera.screenSize.y);
    vec3    viewParams = vec3( planeWidth, planeHeight, camera.focusDistance );

    vec3    focusPointLocal = vec3(uv - 0.5, 1) * viewParams,
            focusPoint = (camera.localToWorld * vec4(focusPointLocal, 1)).xyz,
            camUp = normalize(camera.localToWorld[1].xyz),
            camRight = normalize(camera.localToWorld[0].xyz);

    // Fire rays
    Ray ray;
    vec3 totalIncomingLight = vec3(0);

    for ( int i = 0; i < settings.raysPerFrag; i++ )
    {
        // Calculate ray origin and dir
        vec2 jitter = randVecCartesianNormDist(seed) * settings.divergeStrength / camera.screenSize.x;
        vec3 focusPointJittered = focusPoint + camRight*jitter.x + camUp*jitter.y;

        ray.origin = camera.pos;
        ray.dir = normalize(focusPointJittered - ray.origin);
        totalIncomingLight += Trace(ray, seed);
    }

    // Return final color (average of the frag's rays)
    vec3 fragCol = totalIncomingLight / settings.raysPerFrag;
    color = vec4( fragCol, 1 );
}