extern crate nalgebra_glm as glm;

/**
 * Struct for a camera.
 */
pub struct Camera {
    aspect_ratio: f32,
    pos: glm::Vec3,
    ang: glm::Vec3,
    fov: f32,
    z_near: f32,
    z_far: f32,
    view_transformation: glm::Mat4,
}

/**
 * Camera functions.
 */
impl Camera {
    /**
     * Constructor.
     */
    pub fn new() -> Camera {
        Camera {
            aspect_ratio: 16.0 / 9.0,
            pos: glm::zero(),
            ang: glm::zero(),
            fov: 90.0,
            z_near: 1.0,
            z_far: 1000.0,
            view_transformation: glm::Mat4::identity(),
        }
    }

    /**
     * Calculates the view transformation of the camera and sets it.
     */
    fn calculate_view_transformation( &mut self ) -> &Camera {
        // Create translation matrix
        let translation_matrix = glm::translation( &(glm::vec3(0.0,0.0,-5.0) - self.pos) );

        // Create rotation matrix
        let ( rotation_x, rotation_y, rotation_z ) = (
            glm::rotation(self.ang.x, &glm::vec3(1.0, 0.0, 0.0)),
            glm::rotation(self.ang.y, &glm::vec3(0.0, 1.0, 0.0)),
            glm::rotation(self.ang.z, &glm::vec3(0.0, 0.0, 1.0)),
        );
        let rotation_matrix = rotation_y * rotation_x * rotation_z;
        
        // Create perspective matrix
        let perspective_matrix = glm::perspective( self.aspect_ratio, self.fov, self.z_near, self.z_far );

        // Combine into view transformation and return
        self.view_transformation = perspective_matrix * rotation_matrix * translation_matrix;
        self
    }

    /**
     * Any setter function.
     * If no parameters are defined, only updates the view transformation.
     */
    pub fn set_vars(
        &mut self,
        position: Option<glm::Vec3>,
        angle: Option<glm::Vec3>,
        field_of_view: Option<f32>,
        near_clipping_plane: Option<f32>,
        far_clipping_plane: Option<f32>
    ) -> &Camera {
        // Set variables which are defined
        if let Some(position_defined) = position { self.pos = position_defined; }
        if let Some(angle_defined) = angle { self.ang = angle_defined; }
        if let Some(field_of_view_defined) = field_of_view { self.fov = field_of_view_defined; }
        if let Some(near_clipping_plane_defined) = near_clipping_plane { self.z_near = near_clipping_plane_defined; }
        if let Some(far_clipping_plane_defined) = far_clipping_plane { self.z_far = far_clipping_plane_defined; }
    
        // Update view transformation and return
        self.calculate_view_transformation()
    }

    /**
     * Main setter.
     */
    pub fn set_view_vars( &mut self, position: glm::Vec3, angle: glm::Vec3, field_of_view: f32, near_clipping_plane: f32, far_clipping_plane: f32 ) -> &Camera {
        // Update variables
        self.pos = position;
        self.ang = angle;
        self.fov = field_of_view;
        self.z_near = near_clipping_plane;
        self.z_far = far_clipping_plane;

        // Update view transformation and return
        self.calculate_view_transformation()
    }

    /**
     * Gets the camera's view transformation.
     */
    pub fn get_view_transformation( &self ) -> glm::Mat4 {
        self.view_transformation
    }
}