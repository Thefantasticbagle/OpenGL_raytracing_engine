extern crate nalgebra_glm as glm;

/**
 * Struct for a camera.
 */
pub struct Camera {
    // Set properties
    pos: glm::Vec3,
    ang: glm::Vec3,
    fov: f32,
    z_near: f32,
    z_far: f32,

    // Calculated properties
    rts: glm::Mat4,
    left: glm::Vec3,
    up: glm::Vec3,
    front: glm::Vec3,
}

/**
 * Camera functions.
 */
#[allow(dead_code)]
impl Camera {
    /**
     * Constructor.
     */
    pub fn new() -> Camera {
        Camera {
            pos: glm::zero(),
            ang: glm::zero(),
            fov: 90.0,
            z_near: 1.0,
            z_far: 1000.0,
            rts: glm::Mat4::identity(),
            left: glm::zero(),
            up: glm::zero(),
            front: glm::zero(),
        }
    }

    /**
     * Calculates the RTS of the camera.
     * Unlike the view transformation, this is not meant to transform the world around the camera. It's the camera's transformation relative to the world.
     * (view_transformation was removed as it is not required for raytracing, the transformations in raytracing are not necessarily affine.)
     */
    fn calculate_rts( &mut self ) -> &Camera {
        // Create translation matrix
        let translation_matrix = glm::translation( &self.pos );

        // Create rotation matrix
        let ( rotation_x, rotation_y, rotation_z ) = (
            glm::rotation(self.ang.x, &glm::vec3(1.0, 0.0, 0.0)),
            glm::rotation(self.ang.y, &glm::vec3(0.0, 1.0, 0.0)),
            glm::rotation(self.ang.z, &glm::vec3(0.0, 0.0, 1.0)),
        );
        let rotation_matrix = rotation_y * rotation_x * rotation_z;

        // Create scaling matrix
        let scale_matrix = glm::scaling( &glm::vec3(1.0, 1.0, 1.0) );

        // Combine into RTS
        self.rts = scale_matrix * translation_matrix * rotation_matrix;

        // Calculate left, up, and front of camera
        self.left = self.rts.column(0).xyz().normalize();
        self.up = self.rts.column(1).xyz().normalize();
        self.front = self.rts.column(2).xyz().normalize();

        // Return
        self
    }

    /**
     * Main setter.
     * Parameters can be left as "None", in which case they aren't updated.
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
    
        // Update RTS and return
        self.calculate_rts()
    }

    /**
     * Sets view parameters.
     */
    pub fn set_view_params( &mut self, position: glm::Vec3, angle: glm::Vec3, field_of_view: f32, near_clipping_plane: f32, far_clipping_plane: f32 ) -> &Camera {
        // Update variables
        self.pos = position;
        self.ang = angle;
        self.fov = field_of_view;
        self.z_near = near_clipping_plane;
        self.z_far = far_clipping_plane;

        // Update RTS and return
        self.calculate_rts()
    }

    // --- Getters
    pub fn pos( &self )     -> glm::Vec3 { self.pos }
    pub fn ang( &self )     -> glm::Vec3 { self.ang }
    pub fn fov( &self )     -> f32 { self.fov }
    pub fn z_near( &self )  -> f32 { self.z_near }
    pub fn z_far( &self )   -> f32 { self.z_far }
    pub fn rts( &self )     -> glm::Mat4 { self.rts }
    pub fn left( &self )    -> glm::Vec3 { self.left }
    pub fn front( &self )   -> glm::Vec3 { self.front }
    pub fn up( &self )      -> glm::Vec3 { self.up }

}