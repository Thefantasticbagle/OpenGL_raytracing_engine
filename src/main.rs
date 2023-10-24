// Imports
use std::{ thread, ptr };
use std::sync::{Mutex, Arc, RwLock};

use glutin::event::{Event, WindowEvent, KeyboardInput, ElementState::{Pressed, Released}, VirtualKeyCode::{self}};
use glutin::event_loop::ControlFlow;
use raytracing::{RTSphere, RTMaterial, RTSettings, RTCamera};

extern crate nalgebra_glm as glm;

mod util;
mod shader;
mod camera;
mod raytracing;

// Initial window size
const INITIAL_SCREEN_W: u32 = 720;
const INITIAL_SCREEN_H: u32 = 400;

/**
 * The main function.
 */
fn main() {
    // --- Create contexted window
    // Create context builder
    let context_builder = glutin::ContextBuilder::new()
        .with_vsync ( true );

    // Create window builder
    let window_builder = glutin::window::WindowBuilder::new()
        .with_title ( "OpenGL Raytracing Engine" )
        .with_resizable ( true )
        .with_inner_size ( glutin::dpi::LogicalSize::new(INITIAL_SCREEN_W, INITIAL_SCREEN_H) );

    // Create event loop
    let event_loop = glutin::event_loop::EventLoop::new();

    // Assemble
    let context_pre = context_builder
        .build_windowed ( window_builder, &event_loop ).unwrap();

    // --- Set up event listeners
    let arc_keys_mainthread = Arc::new( Mutex::new( Vec::<VirtualKeyCode>::with_capacity(10) ) );
    let arc_keys_renderthread = Arc::clone( &arc_keys_mainthread );
    
    // --- Start render thread
    // Spawn thread
    let render_thread = thread::spawn ( move || {
        // Load OpenGL context and functions
        let context = unsafe {
            let context_pre_enabled = context_pre.make_current().unwrap();
            gl::load_with ( | symbol | context_pre_enabled.get_proc_address ( symbol ) as *const _ );
            context_pre_enabled
        };

        // OpenGL Settings
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
            gl::Enable(gl::CULL_FACE);
            gl::Disable(gl::MULTISAMPLE);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
            // TODO: Include or exclude this?
            //gl::DebugMessageCallback(Some(util::debug_callback), ptr::null());
        }

        // Set up camera
        let mut camera = camera::Camera::new();
        camera.set_view_params(
            glm::zero(),
            glm::zero(),
            90.0,
            1.0,
            10.0,
        );

        let (
            camera_move_speed,
            camera_rotation_speed,
        ) = (
            5.0,
            3.0,
        );

        // Set up game objects
        let (vertices, indices) = util::create_billboard();
        let my_vao = unsafe {util::create_vao(&vertices, &indices)};
        let simple_shader = unsafe {
            shader::ShaderBuilder::new()
                .attach_shader("shaders/raytracing.vert")
                .attach_shader("shaders/raytracing.frag")
                .link()
        };

        // Set shader settings
        let settings = RTSettings {
            max_bounces: 4,
            rays_per_frag: 16,
            diverge_strength: 0.03,
        };

        unsafe {
            settings.send_uniform( &simple_shader, "settings" );
        }

        // Create SSBO for spheres
        // For now the data is left blank, as it is immidiately overwritten in the gameloop.
        // However, the amount of objects must be the same so the correct amount of space is reserved.
        let spheres_count = 5;
        let mut spheres = Vec::new();
        for _ in 0..spheres_count {
            spheres.push( RTSphere::new() )
        }

        let mut ssbo = unsafe {
            shader::SSBOBuilder::new()
                .set_data( spheres )
                .set_shader_details( simple_shader.pid, 0, "MaterialBuffer" )
                .link()
        };

        

        // ------------------------------------------ //
        // --------------- Gameloop ----------------- //
        // ------------------------------------------ //

        // Start time
        let ( time_start, mut time_prev ) = (
            std::time::Instant::now(),
            std::time::Instant::now()
        );
        
        loop {
            // Elapsed and delta time
            let time = std::time::Instant::now();
            let ( time_elapsed, dt ) = (
                time.duration_since( time_start ).as_secs_f32(),
                time.duration_since(time_prev).as_secs_f32(),
            );
            time_prev = time;

            // TODO: Resize events
            let ( mut screen_width, mut screen_height ) = ( INITIAL_SCREEN_W, INITIAL_SCREEN_H );

            // --- Key events
            let ( mut movement, mut rotation ) = ( glm::Vec3::zeros(), glm::Vec3::zeros() );

            if let Ok( keys ) = arc_keys_renderthread.lock() {
                for key in keys.iter() { match key {

                    // Movement
                    VirtualKeyCode::A => {
                        movement -= camera.left() * dt * camera_move_speed;
                    }
                    VirtualKeyCode::D => {
                        movement += camera.left() * dt * camera_move_speed;
                    }
                    VirtualKeyCode::W => {
                        movement += camera.front() * dt * camera_move_speed;
                    }
                    VirtualKeyCode::S => {
                        movement -= camera.front() * dt * camera_move_speed;
                    }
                    VirtualKeyCode::Space => {
                        movement += camera.up() * dt * camera_move_speed;
                    }
                    VirtualKeyCode::LShift => {
                        movement -= camera.up() * dt * camera_move_speed;
                    }

                    // Rotation
                    VirtualKeyCode::Right => {
                        rotation.y += dt * camera_rotation_speed;
                    }
                    VirtualKeyCode::Left => {
                        rotation.y -= dt * camera_rotation_speed;
                    }
                    VirtualKeyCode::Up => {
                        if rotation.x > -glm::pi::<f32>() / 2.0 {
                            rotation.x -= dt * camera_rotation_speed;
                        }
                    }
                    VirtualKeyCode::Down => {
                        if rotation.x < glm::pi::<f32>() / 2.0 {
                            rotation.x += dt * camera_rotation_speed;
                        }
                    }

                    _ => { }
                } }
            }

            // --- OpenGL
            unsafe {
                // Clear color and depth buffers
                gl::ClearColor(0.04, 0.05, 0.09, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

                // Activate shader
                simple_shader.activate();

                // Update camera with player movement
                camera.set_vars(
                    Some( camera.pos() + movement ),
                    Some( camera.ang() + rotation ),
                    None,
                    None,
                    None
                );

                // Create RTCamera and pass to shader
                // This camera is a lot like the normal Camera, but only carries the necessary variables for the shader to use
                let rtcamera = RTCamera {
                    screen_size: glm::vec2( screen_width as f32, screen_height as f32 ),
                    fov: 60.0,
                    focus_distance: 1.0,
                    pos: camera.pos(),
                    local_to_world: camera.rts(),
                };
                rtcamera.send_uniform( &simple_shader, "camera" );

                // Update sphere objects
                ssbo.update_data(
                    vec![
                        RTSphere {
                            center: glm::vec3((time_elapsed*0.5).sin() * 100.0 , time_elapsed.cos() * 100.0, 0.0),
                            radius: 50.0,
                            material: RTMaterial {
                                color: glm::vec4(1.0, 0.7, 0.3, 0.0),
                                emission_color: glm::vec4(1.0, 0.7, 0.3, 1.0),
                                specular_color: glm::vec4(1.0, 1.0, 1.0, 0.0),
                                smoothness: 0.5,
                            }
                        },
                        RTSphere {
                            center: glm::vec3(0.0, 0.0, 0.0),
                            radius: 2.0,
                            material: RTMaterial {
                                color: glm::vec4(1.0, 1.0, 1.0, 1.0),
                                emission_color: glm::vec4(1.0, 1.0, 0.0, 0.0),
                                specular_color: glm::vec4(1.0, 1.0, 1.0, 0.2),
                                smoothness: 0.3,
                            }
                        },
                        RTSphere {
                            center: glm::vec3(0.0, 0.0, 3.0),
                            radius: 1.0,
                            material: RTMaterial {
                                color: glm::vec4(1.0, 0.0, 0.0, 1.0),
                                emission_color: glm::vec4(1.0, 0.0, 0.0, 1.0),
                                specular_color: glm::vec4(1.0, 0.0, 0.0, 0.2),
                                smoothness: 0.3,
                            }
                        },
                        RTSphere {
                            center: glm::vec3(3.0, 0.0, 0.0),
                            radius: 2.0,
                            material: RTMaterial {
                                color: glm::vec4(0.0, 1.0, 0.0, 1.0),
                                emission_color: glm::vec4(0.0, 1.0, 0.0, 1.0),
                                specular_color: glm::vec4(0.0, 1.0, 0.0, 0.2),
                                smoothness: 0.3,
                            }
                        },
                        RTSphere {
                            center: glm::vec3(2.5, -0.5, 2.5),
                            radius: 2.0,
                            material: RTMaterial {
                                color: glm::vec4(0.0, 0.0, 1.0, 1.0),
                                emission_color: glm::vec4(0.0, 0.0, 1.0, 0.6),
                                specular_color: glm::vec4(0.0, 1.0, 1.0, 0.5),
                                smoothness: 0.6,
                            }
                        },
                    ]
                );
                gl::Uniform1i( simple_shader.get_uniform_location( "spheresCount" ), spheres_count as i32);

                // Draw
                gl::BindVertexArray(my_vao);
                gl::DrawElements(
                    gl::TRIANGLES, 
                    indices.len() as gl::types::GLint,
                    gl::UNSIGNED_INT,
                    ptr::null()
                );
            }

            // "Flip" screen
            context.swap_buffers().unwrap(); // we use "double buffering" to avoid artifacts
        }
    } );

    // Spawn another thread for error handling
    let render_thread_healthy = Arc::new ( RwLock::new(true) );
    let render_thread_watcher = Arc::clone ( &render_thread_healthy );
    thread::spawn ( move || {
        if !render_thread.join().is_ok() {
            if let Ok ( mut health ) = render_thread_watcher.write() {
                println! ( "An error occured in the render thread" );
                *health = false;
            }
        }
    } );

    // --- Start event loop in the main thread
    event_loop.run ( move | event, _, control_flow | {
        *control_flow = ControlFlow::Wait;

        // Break loop if an error occurs in the render thread
        if let Ok ( health ) = render_thread_healthy.read() {
            // TODO: Add "!"
            if *health == false {
                *control_flow = ControlFlow::Exit;
            }
        }

        // Handle events
        match event {
            //close window
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit;
            }

            //keyboard input
            Event::WindowEvent { event: WindowEvent::KeyboardInput {
                input: KeyboardInput { state: key_state, virtual_keycode: Some(key_code), .. }, .. 
            }, .. } => {
                if let Ok( mut keys ) = arc_keys_mainthread.lock() {
                    match key_state {
                        Pressed => {
                            if !keys.contains( &key_code ) {
                                keys.push( key_code );
                            }
                        },
                        Released => {
                            if keys.contains( &key_code ) {
                                let key_index = keys.iter().position( |&k| k == key_code ).unwrap();
                                keys.remove( key_index );
                            }
                        },
                    }
                }
            }

            //default
            _ => { }
        }
    } );
}