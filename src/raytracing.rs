use crate::shader::Shader;


/**
 * Struct for storing raytracing settings.
 */
pub struct RTSettings {
    pub max_bounces: u32,
    pub rays_per_frag: u32,
    pub diverge_strength: f32,
    pub focus_distance: f32,
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
        gl::Uniform1f( shader.get_uniform_location( format!("{uniform_name}.focusDistance").as_str() ), self.focus_distance);
        
        // Switch back and return
        gl::UseProgram( prev_pid as u32 );
    }
}

/**
 * Struct for a raytracing material.
 */
#[repr(C, align(16))] // requirement for the std(140/430) standard, see https://www.khronos.org/opengl/wiki/Interface_Block_(GLSL)#Memory_layout.
pub struct RTMaterial {
    pub color: glm::Vec4,
    pub emission_color: glm::Vec4,
    pub specular_color: glm::Vec4,
    pub smoothness: f32,
}

/**
 * Struct for a raytraced sphere.
 */
#[repr(C, align(16))] // requirement for the std(140/430) standard, see https://www.khronos.org/opengl/wiki/Interface_Block_(GLSL)#Memory_layout.
pub struct RTSphere {
    pub center: glm::Vec3,
    pub radius: f32,
    pub material: RTMaterial,
}