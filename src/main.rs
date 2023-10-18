// Imports
use std::{ thread, ptr };
use std::sync::{Mutex, Arc, RwLock};

use glutin::event::{Event, WindowEvent, DeviceEvent, KeyboardInput, ElementState::{Pressed, Released}, VirtualKeyCode::{self, *}};
use glutin::event_loop::ControlFlow;

extern crate nalgebra_glm as glm;

mod util;
mod shader;
mod camera;

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
        let (vertices, indices) = util::create_triangle_triangle(8, 8);
        let my_vao = unsafe {util::create_vao(&vertices, &indices)};
        let simple_shader = unsafe {
            shader::ShaderBuilder::new()
                .attach_shader("shaders/simple.vert")
                .attach_shader("shaders/simple.frag")
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

                // Physics updates
                camera.set_vars(
                    Some( camera.pos() + movement ),
                    Some( camera.ang() + rotation ),
                    None,
                    None,
                    None
                );

                // Apply shader and camera transformations
                simple_shader.activate();
                simple_shader.set_uniform_mat4( "view", camera.view_transformation() );
                
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