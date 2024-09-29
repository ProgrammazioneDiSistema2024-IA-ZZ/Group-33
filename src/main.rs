mod mouse;
mod window;
mod backup;
mod read_config;
mod types;


use std::sync::{Arc, Mutex, Condvar};
use std::thread;
use crate::types::BackupState;


fn main() {
    // Variabile condivisa tra i thread con Mutex e Condvar
    let state = Arc::new((Mutex::new(BackupState::Idle), Condvar::new()));

    // Thread per il monitoraggio del mouse
    let state_clone = Arc::clone(&state);
    let mouse_thread = thread::spawn(move || {
        mouse::mouse_movements(state_clone);
    });

    // Thread per il backup
    let state_clone = Arc::clone(&state);
    let backup_thread = thread::spawn(move || {
        backup::backup_files(state_clone);
    });

    //finestra con loop per conferma backup
    let state_clone = Arc::clone(&state);
    window::make_window(state_clone);

    /*
    // Thread per la gestione della finestra di conferma
    let state_clone = Arc::clone(&state);
    let conferma_thread = thread::spawn(move || {
        window::get_window(state_clone);
    });
     */




    /*
    // Thread per il logging del consumo di CPU
    let log_thread = thread::spawn(|| {
        log_cpu();
    });
    */

    // Unisci tutti i thread al main thread
    mouse_thread.join().unwrap();
    //conferma_thread.join().unwrap();
    backup_thread.join().unwrap();
    //log_thread.join().unwrap();
}


/*
fn main() {
    // Avvio del monitoraggio del mouse su un thread separato
    let mouse_thread = thread::spawn(|| {
        mouse::monitor_mouse_movements();
    });
    mouse_thread.join().expect("Il thread del monitoraggio del mouse ha avuto un errore.");
    /*
    let source = "C:\\Users\\maxim\\Desktop\\file_backup\\source_folder".to_string();
    let destination = "C:\\Users\\maxim\\Desktop\\file_backup\\destination_folder".to_string();
    let extensions = vec!["mm", "rs", "jpg", "*"]; // Lista di estensioni da copiare (esempio)
    backup_files(&source, &destination, &extensions);
    */
}
*/