use std::sync::{Arc, Condvar, Mutex};
use std::sync::mpsc;
use pixels::{Pixels, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use rusttype::{Font, Scale, point};
use crate::types::BackupState;

const WIDTH: u32 = 500;
const HEIGHT: u32 = 200;

pub fn make_window(state: Arc<(Mutex<BackupState>, Condvar)>) {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Warning")
        .with_inner_size(LogicalSize::new(WIDTH, HEIGHT))
        .with_resizable(false)
        .with_visible(false)
        .build(&event_loop)
        .unwrap();

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture).unwrap()
    };

    let font_data = include_bytes!("../fonts/DejaVuSans-Bold.ttf");
    let font = Font::try_from_bytes(font_data as &[u8]).expect("Errore nel caricamento del font.");

    // Creiamo un canale per comunicare tra il thread secondario e l'event loop principale
    let (tx, rx) = mpsc::channel();

    let state_clone = Arc::clone(&state);
    std::thread::spawn(move || {
        let (lock, cvar) = &*state_clone;
        loop {
            let mut state_guard = lock.lock().unwrap();
            while *state_guard != BackupState::Confirming {
                state_guard = cvar.wait(state_guard).unwrap();
            }

            // Invio comando per rendere visibile la finestra
            tx.send(true).unwrap();

            while *state_guard == BackupState::Confirming {
                state_guard = cvar.wait(state_guard).unwrap();
            }

            if *state_guard == BackupState::Confirmed {
                *state_guard = BackupState::BackingUp;
                cvar.notify_all(); // Notifica l’inizio del backup
            }

            // Invio comando per nascondere la finestra
            tx.send(false).unwrap();
        }
    });

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        // Gestione dei messaggi dal thread secondario
        if let Ok(visible) = rx.try_recv() {
            window.set_visible(visible);
            if visible {
                window.request_redraw();
            }
        }

        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                window.set_visible(false);
                let (lock, cvar) = &*state;
                let mut state_guard = lock.lock().unwrap();
                *state_guard = BackupState::Idle;
                cvar.notify_all();
            }
            Event::RedrawRequested(_) => {
                let frame = pixels.frame_mut();
                for pixel in frame.chunks_mut(4) {
                    pixel.copy_from_slice(&[0, 0, 0, 255]);
                }
                let scale = Scale::uniform(24.0);
                let text = "Close the window or move the mouse to\nthe top-left corner to cancel, move the\nmouse to the bottom-right corner to\nconfirm the backup.";
                let v_metrics = font.v_metrics(scale);
                let start_offset = point(5.0, 5.0 + v_metrics.ascent);
                let mut offset = start_offset.clone();

                for c in text.chars() {
                    if c == '\n' {
                        offset.x = start_offset.x;
                        offset.y += v_metrics.ascent + v_metrics.descent + 10.0;
                        continue;
                    }
                    for glyph in font.layout(&c.to_string(), scale, offset) {
                        if let Some(bounding_box) = glyph.pixel_bounding_box() {
                            glyph.draw(|x, y, v| {
                                let x = x as usize + bounding_box.min.x as usize;
                                let y = y as usize + bounding_box.min.y as usize;
                                let idx = (x + y * WIDTH as usize) * 4;
                                if idx < frame.len() {
                                    let alpha = (v * 255.0) as u8;
                                    frame[idx + 0] = 255;
                                    frame[idx + 1] = 255;
                                    frame[idx + 2] = 255;
                                    frame[idx + 3] = alpha;
                                }
                            });
                        }
                        offset.x += glyph.unpositioned().h_metrics().advance_width;
                    }
                }
                pixels.render().unwrap();
            }
            _ => {}
        }
    });
}
