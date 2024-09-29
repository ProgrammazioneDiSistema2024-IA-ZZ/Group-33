use device_query::{DeviceState, DeviceQuery, MouseState};
use std::thread;
use std::time::Duration;
use scrap::Display;
use std::sync::{Arc, Mutex, Condvar};
use crate::types::BackupState;

const TRESHOLD: i32 = 50;

pub fn mouse_movements(state: Arc<(Mutex<BackupState>, Condvar)> ){
    // Otteniamo il monitor principale
    let display = Display::primary().expect("Impossibile ottenere il monitor principale");

    // Otteniamo le dimensioni del monitor
    let width = display.width() as i32;
    let height = display.height() as i32;

    println!("Dimensione dello schermo: {}x{}", width, height);
    
    let mut count = 0;
    
    //get the state of the device
    let device_state = DeviceState::new();
    loop {
        let mouse: MouseState = device_state.get_mouse();
        let position = mouse.coords; // Ottiene le coordinate del mouse
        //println!("Mouse position: {:?}", position);

        if count < 4 {
            if position.0 < TRESHOLD && position.1 < TRESHOLD {
                count = 1;
            } else if position.0 > (width - TRESHOLD) && position.1 < TRESHOLD {
                if count == 1 || count == 2 {
                    count = 2;
                } else {
                    count = 0;
                }
            } else if position.0 > (width - TRESHOLD) && position.1 > (height - TRESHOLD) {
                if count == 2 || count == 3 {
                    count = 3;
                } else {
                    count = 0;
                }
            } else if position.0 < TRESHOLD && position.1 > (height - TRESHOLD) {
                if count == 3 || count == 4 {
                    //increment count
                    count = 4;

                    let (lock, cvar) = &*state;
                    let mut state_guard = lock.lock().unwrap();
                    while *state_guard != BackupState::Idle {
                        state_guard = cvar.wait(state_guard).unwrap();
                    }
                    *state_guard = BackupState::Confirming;
                    cvar.notify_all();
                    println!("Confirming back up...");
                } else {
                    count = 0;
                }
            }
        }else{
            let (lock, cvar) = &*state;
            let mut state_guard = lock.lock().unwrap();
            if *state_guard == BackupState::Idle {
                count = 0;
            }else {
                //it we touch upper left angle
                if position.0 < TRESHOLD && position.1 < TRESHOLD {
                    //Funziona annulla
                    count = 0;
                    println!("Annulla Back-up");

                    *state_guard = BackupState::Idle;
                    cvar.notify_all();
                } else if position.0 > (width - TRESHOLD) && position.1 > (height - TRESHOLD) { //bottom right
                    //funzione accetta (fai backup)
                    count = 5;
                    println!("Conferma Back-up");

                    *state_guard = BackupState::Confirmed;
                    cvar.notify_all();
                }
            }
        }

        if count == 5 {
            let (lock, cvar) = &*state;
            let mut state_guard = lock.lock().unwrap();
            while *state_guard != BackupState::Idle {
                state_guard = cvar.wait(state_guard).unwrap();
            }
            count = 0;
        }

        thread::sleep(Duration::from_millis(100));
    }
}
