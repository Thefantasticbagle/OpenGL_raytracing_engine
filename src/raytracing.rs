use crate::shader::Shader;

/**
 * Vec3 for GLSL, put after normal floats.
 * Since GLSL std140/430 causes misalignment with vec3s, I had to make this abomination...
 * https://www.khronos.org/opengl/wiki/Interface_Block_(GLSL)#Memory_layout
 * https://stackoverflow.com/questions/38172696/should-i-ever-use-a-vec3-inside-of-a-uniform-buffer-or-shader-storage-buffer-o
 */
#[repr(C, align(16))]
pub struct Vec3a16 {
    pub x: f32,
        y: f32,
        z: f32,
}

/**
 * Conversion glm::vec3 -> Vec3a16.
 * Use glm::vec3::into() to invoke.
 */
impl From<glm::Vec3> for Vec3a16 {
    fn from(v: glm::Vec3) -> Vec3a16 {
        Vec3a16 { x: v.x, y: v.y, z: v.z }
    }
}

/**
 * Conversion Vec3a16 -> glm::vec3.
 * Use Vec3a16::into() to invoke.
 */
impl From<Vec3a16> for glm::Vec3 {
    fn from(v: Vec3a16) -> glm::Vec3 {
        glm::vec3(v.x, v.y, v.z)
    }
}

/**
 * Struct for storing raytracing settings.
 */
#[repr(C, align(16))]
pub struct RTSettings {
    pub max_bounces: u32,
    pub rays_per_frag: u32,
    pub diverge_strength: f32,
}

/**
 * Functions for dealing with raytrace settings.
 */
impl RTSettings {
    /**
     * Sends the RTSettings' data to a uniform variable in a given shader.
     * 
     * @param shader The shader.
     * @param uniform_name The name of the uniform variable in the shader.
     */
    pub unsafe fn send_uniform( self, shader: &Shader, uniform_name: &str ) {
        // Temporarily switch to the shader we're setting uniforms for
        let mut prev_pid: gl::types::GLint = 0;
        gl::GetIntegerv(gl::CURRENT_PROGRAM,&mut prev_pid);
        shader.activate();
        
        // Set uniforms
        gl::Uniform1ui( shader.get_uniform_location( format!("{uniform_name}.maxBounces").as_str() ), self.max_bounces);
        gl::Uniform1ui( shader.get_uniform_location( format!("{uniform_name}.raysPerFrag").as_str() ), self.rays_per_frag);
        gl::Uniform1f( shader.get_uniform_location( format!("{uniform_name}.divergeStrength").as_str() ), self.diverge_strength);
        
        // Switch back and return
        gl::UseProgram( prev_pid as u32 );
    }
}

/**
 * Struct for a raytracing material.
 */
#[repr(C, align(16))]
pub struct RTMaterial {
    pub color: glm::Vec4,
    pub emission_color: glm::Vec4,
    pub specular_color: glm::Vec4,
    pub smoothness: f32,
}

/**
 * RTMaterial functions.
 */
impl RTMaterial {
    /**
     * Creates a new, blank, RTMaterial.
     */
    pub fn new() -> RTMaterial {
        RTMaterial { color: glm::zero(), emission_color: glm::zero(), specular_color: glm::zero(), smoothness: 0.0 }
    }
}

/**
 * Struct for a raytraced sphere.
 */
#[repr(C, align(16))]
pub struct RTSphere {
    pub radius: f32,
    pub center: Vec3a16,
    pub material: RTMaterial,
}

/**
 * RTSphere functions.
 */
impl RTSphere {
    /**
     * Creates a new, blank, RTSphere.
     */
    pub fn new() -> RTSphere {
        RTSphere { radius: 0.0, center: glm::vec3(0.0, 0.0, 0.0).into(), material: RTMaterial::new() }
    }
}

// RTTriangle
#[repr(C, align(16))]
pub struct RTTriangle {
    pub p0: Vec3a16,
    pub p1: Vec3a16,
    pub p2: Vec3a16,
    pub normal0: Vec3a16,
    pub normal1: Vec3a16,
    pub normal2: Vec3a16,
    pub material: RTMaterial,
}

/**
 * RTTriangle functions.
 */
impl RTTriangle {
    /**
     * Creates a new, blank, RTTriangle.
     */
    pub fn new() -> RTTriangle {
        RTTriangle { 
            p0: glm::Vec3::zeros().into(), 
            p1: glm::Vec3::zeros().into(), 
            p2: glm::Vec3::zeros().into(), 
            normal0: glm::Vec3::zeros().into(), 
            normal1: glm::Vec3::zeros().into(), 
            normal2: glm::Vec3::zeros().into(), 
            material: RTMaterial::new(),
        }
    }
}

/**
 * Struct for a raytracing camera.
 */
#[repr(C, align(16))]
pub struct RTCamera {
    pub screen_size: glm::Vec2,
    pub fov: f32,
    pub focus_distance: f32,
    pub pos: Vec3a16,
    pub local_to_world: glm::Mat4,
}

/**
 * Functions for dealing with the raytracing camera.
 */
impl RTCamera {
    /**
     * Sends the RTCamera's data to a uniform variable in a given shader.
     * 
     * @param shader The shader.
     * @param uniform_name The name of the uniform variable in the shader.
     */
    pub unsafe fn send_uniform( self, shader: &Shader, uniform_name: &str ) {
        // Temporarily switch to the shader we're setting uniforms for
        let mut prev_pid: gl::types::GLint = 0;
        gl::GetIntegerv(gl::CURRENT_PROGRAM,&mut prev_pid);
        shader.activate();
        
        // Set uniforms
        gl::Uniform2f( shader.get_uniform_location( format!("{uniform_name}.screenSize").as_str() ), self.screen_size.x, self.screen_size.y);
        gl::Uniform1f( shader.get_uniform_location( format!("{uniform_name}.fov").as_str() ), self.fov);
        gl::Uniform1f( shader.get_uniform_location( format!("{uniform_name}.focusDistance").as_str() ), self.focus_distance);
        gl::Uniform3f( shader.get_uniform_location( format!("{uniform_name}.pos").as_str() ), self.pos.x, self.pos.y, self.pos.z);
        shader.set_uniform_mat4( format!("{uniform_name}.localToWorld").as_str(), self.local_to_world);

        // Switch back and return
        gl::UseProgram( prev_pid as u32 );
    }
}