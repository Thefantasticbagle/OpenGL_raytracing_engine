use std::{ mem, os::raw::c_void, ffi::CString };


/**
 * Gets the size of an array.
 * 
 * @param val The array.
 * @return The size of the array in bytes.
 */
pub fn byte_size_of_array<T>(val: &[T]) -> isize {
    std::mem::size_of_val(&val[..]) as isize
}

/**
 * Gets the pointer to an array.
 * 
 * @param val The array.
 * @return The c-style pointer to the array.
 */
pub fn pointer_to_array<T>(val: &[T]) -> *const c_void {
    &val[0] as *const T as *const c_void
}

/**
 * Gets the size of a given type.
 */
pub fn size_of<T>() -> i32 {
    mem::size_of::<T>() as i32
}

/**
 * Gets the offset for a given amount of a type.
 * 
 * @param n The amount.
 * @return The offset for the given amount of <type> objects as a c-style pointer.
 */
pub fn offset<T>(n: u32) -> *const c_void {
    (n * mem::size_of::<T>() as u32) as *const T as *const c_void
}

/**
 * Casts a rust string into C's i8 array.
 * 
 * @param s The string.
 * @return The string, casted to C's version (i8 array).
 */
pub fn str_as_i8(s: &str) -> *const i8 {
    let c_str = CString::new( s ).unwrap();
    c_str.as_ptr() as *const i8
}

/**
 * Creates a VAO from a list of vertices and indices.
 * For now the VAO only contains one attribute: position(xyz).
 * 
 * @param vertices The vertex positions.
 * @param indices The indices which form the triangles.
 * 
 * @return The id of the generated VAO.
 */
pub unsafe fn create_vao(vertices: &Vec<f32>, indices: &Vec<u32>) -> u32 {
    // Generate & bind VAO
    let mut vao: gl::types::GLuint = 0;
    gl::GenVertexArrays(1, &mut vao);
    gl::BindVertexArray(vao);

    // Generate & bind VBO
    let mut vbo: gl::types::GLuint = 0;
    gl::GenBuffers(1, &mut vbo);
    gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

    // Fill VBO with data
    gl::BufferData(
        gl::ARRAY_BUFFER,
        byte_size_of_array(vertices),
        pointer_to_array(vertices),
        gl::STATIC_DRAW,
    );

    // Configure VAP
    gl::EnableVertexAttribArray(0);
    gl::VertexAttribPointer(
        0, // Same as parameter of above line (location=0 in shader)
        3, // x, y, z => 3 values
        gl::FLOAT, // Values are of type float
        gl::FALSE, // Values does NOT need to be automatically normalized
        3 * size_of::<f32>(), // x, y, z is the only data in the vector and are f32s
        std::ptr::null(), // No offset
    );

    // Generate & bind IBO/EBO
    let mut ebo: gl::types::GLuint = 0;
    gl::GenBuffers(1, &mut ebo);
    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);

    // Fill IBO/EBO with data
    gl::BufferData(
        gl::ELEMENT_ARRAY_BUFFER,
        byte_size_of_array(indices),
        pointer_to_array(indices),
        gl::STATIC_DRAW,
    );

    // Unbind buffers
    // (This was not requested by the assignment, but is good practise)
    // (Also, the IBO/EBO should not be unbound as it is stored in the VAO)
    gl::BindBuffer(gl::ARRAY_BUFFER, 0);
    gl::BindVertexArray(0);

    // Return
    vao
}

/**
 * Creates the vertices and indices for a triangle made of smaller triangles.
 * 
 * @param triangle_width How many subtriangles wide the triangle is.
 * @param triangle_height How many subtriangles tall the triangle is.
 * 
 * @return Vertices and Indices as a vector of float32s and unsigned int32s, respectively.
 */
pub fn create_triangle_triangle(triangle_width: i32, triangle_height: i32) -> (Vec<f32>, Vec<u32>) {
    let mut vertices: Vec<f32> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut i: u32 = 0;

    for y in 0..triangle_height {
        let y_float: f32 = y as f32 / triangle_height as f32;
        let x_offset: f32 = -y_float / 2 as f32;
        for x in y..triangle_width {
            let x_float: f32 = x as f32 / triangle_width as f32;
            
            // Add vertex
            vertices.push(x_float + x_offset - 0.5);
            vertices.push(y_float - 0.5);
            vertices.push(0.0);
            
            // Add indices for this triangle
            i += 1;
            if x >= triangle_width - 1 { continue; } // (Unless we're on the right-most end of the triangle)
            indices.push(i-1);
            indices.push(i);
            indices.push(i+(triangle_width-y) as u32-1);
        }
    }

    (vertices, indices)
}

/**
 * Creates the vertices and indices for a simple billboard which covers the entire screen.
 * 
 * @return Vertices and Indices as a vector of float32s and unsigned int32s, respectively.
 */
pub fn create_billboard( ) -> (Vec<f32>, Vec<u32>) {
    let vertices: Vec<f32> = vec![
        1.0, 1.0, 0.0,
        -1.0, 1.0, 0.0,
        -1.0, -1.0, 0.0,
        1.0, -1.0, 0.0,
    ];

    let indices: Vec<u32> = vec![
        0, 1, 3,
        1, 2, 3,
    ];

    (vertices, indices)
}