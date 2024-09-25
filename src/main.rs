
mod mouse;
mod window;
mod backup;
use std::sync::{Arc, Mutex, Condvar};
use std::thread;

fn main() {
    // Variabile condivisa tra i thread con Mutex e Condvar
    let count = Arc::new((Mutex::new(0), Condvar::new()));

    // Thread per il monitoraggio del mouse
    let count_clone = Arc::clone(&count);
    let mouse_thread = thread::spawn(move || {
        mouse::mouse_movements(count_clone);
    });


    /*
    // Thread per la gestione della finestra di conferma
    let count_clone = Arc::clone(&count);
    let conferma_thread = thread::spawn(move || {
        conferma_backup(count_clone);
    });
    */



    // Thread per il backup
    let count_clone = Arc::clone(&count);
    let backup_thread = thread::spawn(move || {
        backup::backup_files(count_clone);
    });

    mouse_thread.join().unwrap();
    /*
    // Thread per il logging del consumo di CPU
    let log_thread = thread::spawn(|| {
        log_cpu();
    });

    // Unisci tutti i thread al main thread
    mouse_thread.join().unwrap();
    conferma_thread.join().unwrap();
    backup_thread.join().unwrap();
    log_thread.join().unwrap();
     */
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