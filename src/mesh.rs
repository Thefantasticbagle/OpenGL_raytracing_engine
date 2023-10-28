use crate::raytracing::{RTTriangle, RTMeshInfo, RTMaterial};

/**
 * Struct for holding a mesh.
 */
pub struct Mesh {
    pub vertices: Vec<f32>,
    pub normals: Vec<f32>,
    pub colors: Vec<f32>,
    pub indices: Vec<u32>,
    pub index_count: i32,
}

/**
 * Struct for holding a model.
 */
pub struct Model {
    pub meshes: Vec<Mesh>,
}

/**
 * Model functions.
 */
impl Model {
    /**
     * Creates a new, empty model.
     */
    pub fn new() -> Model {
        return Model { meshes: Vec::new() }
    }

    /**
     * Loads a .obj file into the model.
     * 
     * @param path The path for the .obj file.
     */
    pub fn load_from_file( mut self, path: &str ) -> Model {
        let (parts, _materials)
        = tobj::load_obj(path,
            &tobj::LoadOptions{
                triangulate: true,
                single_index: true,
                ..Default::default()
            }
        ).expect("Failed to load model");

        for part in parts {
            let ( positions, indices ) = ( part.mesh.positions, part.mesh.indices );
            let ( positions_len, indices_len ) = ( positions.len(), indices.len() );
            self.meshes.push( 
                Mesh {
                    vertices: positions,
                    normals: part.mesh.normals,
                    indices: indices,
                    colors: [1.0, 0.0, 0.0, 1.0].iter().cloned().cycle().take(positions_len*4).collect(),
                    index_count: indices_len as i32,
                }
            );
        }

        self
    }

    /**
     * Generates the necessary raytracing structs to render the model.
     * Each part of the model becomes its own mesh, and triangles are dumped into a global triangle vector.
     * 
     * @return Two vectors containing raytracing triangles and meshes, respectively.
     */
    pub fn generate_raytracing_structs( self ) -> ( Vec<RTTriangle>, Vec<RTMeshInfo> ) {
        // Set up buffers and counters
        let ( mut triangles, mut meshes, mut start_index ) = (
            Vec::<RTTriangle>::new(),
            Vec::<RTMeshInfo>::new(),
            0,
        );

        // Iterate parts, adding each as its own mesh in `meshes`
        for part in self.meshes {
            // Set up buffers required for each individual mesh
            let ( mut vertices_vec3, mut boundingbox_min, mut boundingbox_max ) = (
                Vec::<glm::Vec3>::new(),
                glm::Vec3::zeros(),
                glm::Vec3::zeros(),
            );

            // Iterate vertices of part, creating glm::vec3 for each and noting down the min/max point
            for i in 0..part.vertices.len()/3 {
                let vec = glm::vec3(
                    part.vertices[i*3],
                    part.vertices[i*3+1],
                    part.vertices[i*3+2]
                ) / 80.0 + glm::vec3(-1.0, 1.0, 3.0);

                vertices_vec3.push( vec );
                if i == 0 {
                    boundingbox_min = vec; boundingbox_max = vec;
                }
                boundingbox_min = glm::min2( &vec, &boundingbox_min );
                boundingbox_max = glm::max2( &vec, &boundingbox_max );
            }

            // Iterate normals, creating glm::vec3 for each
            let mut normals_vec3 = Vec::<glm::Vec3>::new();
            for i in 0..part.normals.len()/3 {
                normals_vec3.push( glm::vec3(part.normals[i*3], part.normals[i*3+1], part.normals[i*3+2]) );
            }

            // Iterate colors, creating glm::vec4 for each
            let mut colors_vec4 = Vec::<glm::Vec4>::new();
            for i in 0..part.colors.len()/4 {
                colors_vec4.push( glm::vec4(part.colors[i*4], part.colors[i*4+1], part.colors[i*4+2], part.colors[i*4+3]) );
            }

            // Iterate triangles of part, creating raytracing triangles and adding them to `triangles` vector
            for i in 0..part.index_count/3 {
                //if i > 10 { break }
                let ( i0, i1, i2 ) = (
                    part.indices[(i*3) as usize],
                    part.indices[(i*3+1) as usize],
                    part.indices[(i*3+2) as usize],
                );
                let triangle = RTTriangle {
                    p0: vertices_vec3[i0 as usize].into(),
                    p1: vertices_vec3[i1 as usize].into(),
                    p2: vertices_vec3[i2 as usize].into(),
                    normal0: normals_vec3[i0 as usize].into(),
                    normal1: normals_vec3[i1 as usize].into(),
                    normal2: normals_vec3[i2 as usize].into(),
                    material: RTMaterial {
                        color: colors_vec4[i0 as usize],
                        emission_color: glm::vec4(colors_vec4[i0 as usize].x, colors_vec4[i0 as usize].y, colors_vec4[i0 as usize].z, 0.5),
                        specular_color: glm::Vec4::zeros(),
                        smoothness: 0.5,
                    }
                };
                triangles.push( triangle );
            }

            // Create and push raytracing mesh to `meshes`
            meshes.push( RTMeshInfo {
                start_index: start_index,
                count: triangles.len() as u32 - start_index,
                boundingbox_min: boundingbox_min.into(),
                boundingbox_max: boundingbox_max.into(),
            } );

            // Set start index for next part
            start_index = triangles.len() as u32 - 1;
        }

        // Return triangles and meshes
        ( triangles, meshes ) 
    }
}