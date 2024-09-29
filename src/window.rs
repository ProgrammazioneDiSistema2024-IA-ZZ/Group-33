use std::sync::{Arc, Condvar, Mutex};
//import for the window
use pixels::{Pixels, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    //platform::run_return::EventLoopExtRunReturn,
    window::WindowBuilder,
};
use rusttype::{Font, Scale, point};
use crate::types::BackupState;

const WIDTH: u32 = 400;
const HEIGHT: u32 = 200;

pub fn make_window( state: Arc<(Mutex<BackupState>, Condvar)> ){
    // Create an event loop
    let event_loop = EventLoop::new();

    // Create a window with title
    let window = WindowBuilder::new()
        .with_title("attenzione: richiesta")
        .with_inner_size(LogicalSize::new(WIDTH, HEIGHT))
        .with_resizable(false)
        .with_visible(false)
        .build(&event_loop)
        .unwrap();

    // Configure the pixel buffer for rendering
    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture).unwrap()
    };

    // Load a default font
    let font_data = include_bytes!("../fonts/DejaVuSans-Bold.ttf"); // Change path if necessary
    let font = Font::try_from_bytes(font_data as &[u8]).expect("Errore nel caricamento del font.");


    // Loop of events
    event_loop.run(move |event, _, control_flow| {
        match event {
            // Close the window when the user presses "X"
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                //*control_flow = ControlFlow::Exit;
                window.set_visible(false);
                //put state to 0
                let (lock, cvar) = &*state;
                let mut state_guard = lock.lock().unwrap();
                while *state_guard != BackupState::Confirming {
                    state_guard = cvar.wait(state_guard).unwrap();
                }
                *state_guard = BackupState::Idle;
                cvar.notify_all();
            }

            // Draw the content of the window
            Event::RedrawRequested(_) => {
                // Clear the screen
                let frame = pixels.frame_mut(); // Get the frame

                for pixel in frame.chunks_mut(4) {
                    pixel.copy_from_slice(&[0, 0, 0, 255]); // Set background color to black
                }

                // Draw the text
                let scale = Scale::uniform(24.0); // Set font size
                let text = "descrizione";
                let v_metrics = font.v_metrics(scale);
                let offset = point(10.0, 50.0 + v_metrics.ascent); // Position of the text

                for glyph in font.layout(text, scale, offset) {
                    let bounding_box = glyph.pixel_bounding_box().unwrap();
                    glyph.draw(|x, y, v| {
                        let x = x as usize + bounding_box.min.x as usize;
                        let y = y as usize + bounding_box.min.y as usize;
                        let idx = (x + y * WIDTH as usize) * 4;

                        if idx < frame.len() {
                            let alpha = (v * 255.0) as u8;
                            frame[idx + 0] = 255; // Red
                            frame[idx + 1] = 255; // Green
                            frame[idx + 2] = 255; // Blue
                            frame[idx + 3] = alpha; // Alpha
                        }
                    });
                }

                // Show the frame
                pixels.render().unwrap();
            }

            // Check the to_close variable to close the window
            Event::MainEventsCleared => {
                let (lock, cvar) = &*state;
                let mut state_guard = lock.lock().unwrap();
                while *state_guard != BackupState::Confirming {
                    //check if the backup need to start
                    if *state_guard == BackupState::Confirmed {
                        *state_guard = BackupState::BackingUp;
                        cvar.notify_all();
                    }
                    window.set_visible(false);
                    state_guard = cvar.wait(state_guard).unwrap();
                }
                window.set_visible(true);
                window.request_redraw();
            }

            _ => *control_flow = ControlFlow::Wait,
        }
    });
}