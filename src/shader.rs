use gl;
use std::{
    ptr,
    str,
    ffi::CString,
    path::Path,
};

use crate::util::{byte_size_of_array, pointer_to_array};

/**
 * Struct for a compiled shader program.
 */
pub struct Shader {
    pub pid: u32,
}

/**
 * Struct for a shader builder.
 */
pub struct ShaderBuilder {
    pid: u32,
    shaders: Vec::<u32>,
}

/**
 * Enum for different shader types.
 */
pub enum ShaderType {
    Vertex,
    Fragment,
}

/**
 * Type casting ShaderType -> GLenum.
*/
impl Into<gl::types::GLenum> for ShaderType {
    fn into ( self ) -> gl::types::GLenum {
        match self {
            ShaderType::Vertex      => { gl::VERTEX_SHADER },
            ShaderType::Fragment    => { gl::FRAGMENT_SHADER },
        }
    }
}

/**
 * ShaderType functions.
 */
impl ShaderType {
    /**
     * Automatically detect filetype and create the corresponding enum.
     */
    fn from_ext ( ext: &std::ffi::OsStr ) -> Result<ShaderType, String> {
        match ext.to_str().expect("ERROR::SHADER::EXTENSION_NOT_RECOGNIZED") {
            "vert" => { Ok(ShaderType::Vertex) },
            "frag" => { Ok(ShaderType::Fragment) },
            e => { Err(e.to_string()) },
        }
    }
}

/**
 * ShaderBuilder functions.
 */
impl ShaderBuilder {
    /**
     * Constructor.
     */
    pub unsafe fn new() -> ShaderBuilder {
        ShaderBuilder { pid: gl::CreateProgram(), shaders: vec![] }
    }

    /**
     * Gets the error message from a shader compilation failure, if it exists.
     * 
     * @param shader_id The id of the shader.
     * 
     * @return Ok if no error was found, a string with the error otherwise.
     */
    unsafe fn get_shader_err( &self, shader_id: u32 ) -> Result<String, String> {
        // Fetch log and success status
        let mut success = i32::from( gl::FALSE );
        let mut log = Vec::with_capacity( 512 );
        log.set_len( 512-1 );
        gl::GetShaderiv( shader_id, gl::COMPILE_STATUS, &mut success );

        // If successful, return Ok
        if success == i32::from(gl::TRUE) {
            return Ok( String::new() )
        }

        // Otherwise, get the log and return it as an error
        gl::GetShaderInfoLog(
            shader_id,
            512,
            ptr::null_mut(),
            log.as_mut_ptr() as *mut gl::types::GLchar
        );

        return Err( String::from_utf8_lossy( &log ).to_string() );
    }

    /**
     * Gets the error message from a link event, if it exists.
     * 
     * @return Ok if no error occurred, an error message otherwise.
     */
    unsafe fn get_linker_err( &self ) -> Result<String, String> {
        // Fetch log and success status
        let mut success = i32::from( gl::FALSE );
        let mut log = Vec::with_capacity( 512 );
        log.set_len( 512-1 );
        gl::GetProgramiv( self.pid, gl::LINK_STATUS, &mut success );

        // If successful, return Ok
        if success == i32::from(gl::TRUE) {
            return Ok( String::new() )
        }

        // Otherwise, get the log and return it as an error
        gl::GetProgramInfoLog(
            self.pid,
            512,
            ptr::null_mut(),
            log.as_mut_ptr() as *mut gl::types::GLchar
        );

        return Err( String::from_utf8_lossy( &log ).to_string() );
    }

    /**
     * Compiles a shader, adding it to the compiled shader program of the ShaderBuilder.
     * 
     * @param shader_src The shader.
     * @param shader_type The type of shader.
     */
    pub unsafe fn compile( mut self, shader_src: &str, shader_type: ShaderType ) -> ShaderBuilder {
        // Create and compile the shader
        let ( shader, shader_cstr ) = (
            gl::CreateShader( shader_type.into() ),
            CString::new( shader_src.as_bytes() ).unwrap(),
        );
        gl::ShaderSource( shader, 1, &shader_cstr.as_ptr(), ptr::null() );
        gl::CompileShader( shader );

        // Error handling
        if let Err(err) = self.get_shader_err( shader ) {
            panic!("ERROR::SHADER::COMPILATION_FAILED\n{}", err);
        }

        // Add compiled shader to pipeline and return
        self.shaders.push( shader );
        self
    }

    /**
     * Attaches a shader file to the ShaderBuilder pipeline.
     * 
     * @param shader_path Path to the shader file.
     */
    pub unsafe fn attach_shader( self, shader_path: &str ) -> ShaderBuilder {
        let path = Path::new( shader_path );
        if let Some(ext) = path.extension() {
            // Attempt getting shadertype from  extension
            let shader_type = ShaderType::from_ext( ext )
                .expect( &format!( "ERROR::SHADER::FAILED_TO_PARSE_EXTENSION\n{}" , ext.to_string_lossy().to_string()) );

            // Attempt reading contents of file
            let shader_src = std::fs::read_to_string( path )
                .expect( &format!( "ERROR:SHADER:FAILED_TO_READ_FILE\n{}", shader_path ) );

            // Compile and return
            self.compile( &shader_src, shader_type )
        } else {
            panic!( "ERROR::SHADER::FAILED_TO_READ_EXTENSION" );
        }
    }

    /**
     * Links and finalizes the shader pipeline.
     * 
     * @return The finished shader pipeline.
     */
    #[must_use = "The shader must be linked or it is useless."]
    pub unsafe fn link( self ) -> Shader {
        // Attach shaders
        for &shader in &self.shaders {
            gl::AttachShader( self.pid, shader );
        }

        // Link and errorhandle
        gl::LinkProgram( self.pid );
        if let Err(err) = self.get_linker_err() {
            panic!("ERROR::SHADER::COMPILATION_FAILED\n{}", err);
        }

        // Delete shaders as they are now part of the greater shader pipeline
        for &shader in &self.shaders {
            gl::DeleteShader( shader );
        }

        // Return
        Shader {
            pid: self.pid,
        }
    }
}

/**
 * Shader functions.
 */
impl Shader {
    /**
     * Activates the shader.
     */
    pub unsafe fn activate( &self ) {
        gl::UseProgram( self.pid );
    }

    /**
     * Gets the location of a uniform variable in a shader.
     * 
     * @param pid The shader program id.
     * @param name The name of the uniform variable.
     * 
     * @return The location of the uniform variable, or -1 if it does not exist.
     */
    pub unsafe fn get_uniform_location( &self, name: &str) -> gl::types::GLint {
        let name_cstring = CString::new(name).unwrap();
        let name_ptr: *const i8 = name_cstring.as_ptr() as *const i8;
        return gl::GetUniformLocation(self.pid, name_ptr);
    }

    /**
     * Sets a uniform mat4 in the shader.
     */
    pub unsafe fn set_uniform_mat4( &self, name: &str, value: glm::Mat4 ) {
        gl::UniformMatrix4fv( self.get_uniform_location( name ), 1, gl::FALSE, value.as_ptr());
    }
}

/**
 * SSBO - Shader Storage Buffer Object. Can store at least 128MB.
 * https://www.khronos.org/opengl/wiki/Shader_Storage_Buffer_Object.
 */
#[allow(dead_code)]
pub struct SSBO<T> {
    pid: u32,
    bid: u32,
    binding: u32,
    data: Vec<T>,
    data_size: isize,
}

/**
 * SSBO builder.
 * @see SSBO
 */
pub struct SSBOBuilder<T> {
    pid: u32,
    bid: u32,
    binding: u32,
    data: Vec<T>,
}

/**
 * SSBO builder functions.
 */
impl<T> SSBOBuilder<T> {
    /**
     * Creates an empty SSBO object.
     * Initializes its buffer.
     */
    #[must_use = "The SSBO must be initialized."]
    pub unsafe fn new() -> SSBOBuilder<T> {
        let mut buffer_id: gl::types::GLuint = 0;
        gl::GenBuffers(1, &mut buffer_id);

        SSBOBuilder {
            pid: 0,
            bid: buffer_id,
            binding: 0,
            data: Vec::new(),
        }
    }

    /**
     * Sets the data of the SSBO.
     * The SSBO object must be initialized through the new() method.
     * 
     * @param data The data.
     */
    #[must_use = "The SSBO must have data to be initialized."]
    pub unsafe fn set_data( self, data: Vec<T> ) -> SSBOBuilder<T> {
        //let data = &data[..];

        // Get data size and pointer reference
        let ( data_size, data_ref ) = (
            byte_size_of_array( &data ),
            pointer_to_array( &data ),
        );

        // Set buffer data
        gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, self.bid);
        gl::BufferData(gl::SHADER_STORAGE_BUFFER, data_size, data_ref, gl::DYNAMIC_COPY);
        gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, 0);

        // Return
        self
    }

    /**
     * Sets the shader details for the SSBO.
     * 
     * @param shader_pid The program ID of the compiled shader which uses the SSBO.
     * @param shader_binding The binding number of the SSBO within the shader.
     * @param shader_buffer_name The name of the SSBO/buffer within the shader.
     */
    #[must_use = "The SSBO must contain details about the shader it is used in to function."]
    pub unsafe fn set_shader_details( mut self, shader_pid: u32, shader_binding: u32, shader_buffer_name: &str ) -> SSBOBuilder<T> {
        // Set vars
        self.pid = shader_pid;
        self.binding = shader_binding;
        
        // Find block index and connect to it
        let name_c_str = CString::new( shader_buffer_name ).unwrap();
        let block_index: gl::types::GLuint = gl::GetProgramResourceIndex(
            shader_pid,
            gl::SHADER_STORAGE_BLOCK,
            name_c_str.as_ptr() as *const i8
        );
        
        gl::ShaderStorageBlockBinding( shader_pid, block_index, shader_binding );
        gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, shader_binding, self.bid);

        // Return
        self
    }

    /**
     * Links the SSBO, finalizing it.
     * The data can be changed, but the total size of the new data cannot be greater than the original data's size.
     * 
     * @return The fully initialized SSBO object.
     */
    #[must_use = "The SSBO must be linked to a shader or it is useless."]
    pub unsafe fn link ( self ) -> SSBO<T> {
        SSBO {
            pid: self.pid,
            bid: self.bid,
            binding: self.binding,
            data: self.data,
            data_size: 0,//byte_size_of_array( &self.data ),
        }
    }
}

/**
 * SSBO functions.
 */
impl<T> SSBO<T> {
    /**
     * Updates the data in the SSBO.
     * The new data size cannot exceed the original data size.
     * 
     * @param new_data The new data.
     */
    pub unsafe fn update_data( &mut self, new_data: Vec<T> ) -> &SSBO<T> {
        // Get data size and ref
        let ( new_data_size, new_data_ref ) = (
            byte_size_of_array( &new_data ),
            pointer_to_array( &new_data ),
        );

        // Copy new data into buffer
        gl::BindBuffer( gl::SHADER_STORAGE_BUFFER, self.bid );
        let p = gl::MapBuffer( gl::SHADER_STORAGE_BUFFER, gl::WRITE_ONLY );
        p.copy_from( new_data_ref, new_data_size as usize );
        gl::UnmapBuffer(gl::SHADER_STORAGE_BUFFER);
        gl::BindBuffer( gl::SHADER_STORAGE_BUFFER, 0 );

        // Return
        self
    }
}