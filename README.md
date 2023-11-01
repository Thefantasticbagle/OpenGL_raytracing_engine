# opengl-raytracing-engine
![Image of a knight model](https://github.com/Thefantasticbagle/OpenGL_raytracing_engine/blob/b50dadbdff04a113e2184796990bea895b9ffd51/images/knight.png)
A simple raytracing engine made from scratch in [Rust](https://en.wikipedia.org/wiki/Rust_(programming_language)) with [OpenGL](https://en.wikipedia.org/wiki/OpenGL).

Thanks to the book series "Ray Tracing in One Weekend", which is available at https://raytracing.github.io/ for free.

## Features
### Spheres and triangle meshes
So far the only primitives are `spheres` and `triangles`, but in the future I would like to add other primitives such as toruses, cylinders, discs and more.

Primitives are loaded into the shader via [SSBOs](https://www.khronos.org/opengl/wiki/Shader_Storage_Buffer_Object), which allows for large amounts of data to be passed and updated.

However, this leads to some interesting problems, as OpenGL's std130/430 does not handle certain datatypes well. For example, this is what [Khronos](https://www.khronos.org/opengl/wiki/Interface_Block_(GLSL)) has to say about using the `vec3` datatype:
> You are advised to manually pad your structures/arrays out and avoid using vec3 at all.

To account for this, a custom struct with the name `vec3a16` was made. Use this rather than [glm](https://glm.g-truc.net/)'s `vec3` when passing data to the shader.

### Material properties
Each primitive has a material which describes its physical attributes. So far, these properties have been implemented:
- Color
- Emission color
- Specular probability & color
- Smoothness

In the future I would like to add properties such as transparency and refractive index.

![Image of a circle of spheres showing different degrees of smoothness](https://github.com/Thefantasticbagle/OpenGL_raytracing_engine/blob/b50dadbdff04a113e2184796990bea895b9ffd51/images/smoothness.png)
*reflective spheres with varying degrees of smoothness*

![Image of a circle of spherical mirrors](https://github.com/Thefantasticbagle/OpenGL_raytracing_engine/blob/b50dadbdff04a113e2184796990bea895b9ffd51/images/mirrors.png)
*mirror spheres showing reflections in reflections*

### Anti-aliasing
If you look closely at the image above you might notice that the edges look "choppy", especially in the reflections. This image was taken without anti-aliasing.

To avoid aliasing artifacts, [Distributed Ray Tracing (DRT)](https://en.wikipedia.org/wiki/Distributed_ray_tracing) and [Supersampling](https://en.wikipedia.org/wiki/Supersampling) is used.

![Image showing comparison between no anti-aliasing and anti-aliasing enabled](https://github.com/Thefantasticbagle/OpenGL_raytracing_engine/blob/b50dadbdff04a113e2184796990bea895b9ffd51/images/antialiascomparison.png)
*comparison with and without SSAA anti-aliasing enabled*

### Acceleration structures & culling
AABB bounding boxes are used as [bounding volumes](https://en.wikipedia.org/wiki/Bounding_volume) for culling. Additionally, [Back-face culling](https://en.wikipedia.org/wiki/Back-face_culling) can be enabled for triangles.

This means that meshes outside of the [view frustum](https://en.wikipedia.org/wiki/Viewing_frustum) are not rendered, and rays pass through triangles which are oriented counter-clockwise relative to the ray's direction.

In the future I would like to expand on these acceleration structures and create a [Bounding Volume Hierarchy (BVH)](https://en.wikipedia.org/wiki/Bounding_volume_hierarchy) for them to allow for quick and easy updating.

## Setup
### Downloading the repository
```sh
$ git clone https://github.com/Thefantasticbagle/opengl-raytracing-engine.git
$ cd opengl-raytracing-engine
```

### Installing dependencies
To run the project, both cargo and rustc is required.<br> Visit https://rustup.rs/ to download both in one.

Verify that cargo and rustc is installed, use the following commands:
```sh
$ cargo --version
$ rustc --version
```

## Run
To run the program with cargo, use the following command:
```sh
$ cargo run
```
Note that the program only contains controls for navigating the camera around the scene, and that objects such as spheres and meshes must be added manually in the code.
