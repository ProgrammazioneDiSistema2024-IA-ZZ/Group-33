mod mouse;
mod window;
mod backup;
mod read_files;
mod types;
mod performance;

use std::sync::{Arc, Mutex, Condvar};
use std::{fs, thread, time::Duration};
use std::env;
use std::path::Path;
use std::fs::OpenOptions;
use std::io::Write;
use sysinfo::{System, Pid, ProcessesToUpdate};
use crate::types::BackupState;

use std::os::windows::fs::symlink_file;


fn main() {
    //AGGIUNGERE IF PER SO
    // Chiama set_bootstrap e gestisce eventuali errori
    if let Err(e) = set_bootstrap() {
        eprintln!("Errore durante la creazione del link simbolico: {}", e);
    }

    // Ottieni il PID del processo corrente
    let process_id = std::process::id();
    let pid = Pid::from(process_id as usize);

    println!("ID process = {}",pid);

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

    // Thread per il logging del consumo di CPU
    // Avvia un thread separato per registrare il consumo di CPU ogni 2 minuti
    let cpu_log_thread = thread::spawn( move || {
        performance::log_cpu_usage_periodically(pid, 5, "cpu_usage.log"); // 120 secondi = 2 minuti
    });

    //finestra con loop per conferma backup
    let state_clone = Arc::clone(&state);
    window::make_window(state_clone);

    // Unisci tutti i thread al main thread
    mouse_thread.join().unwrap();
    backup_thread.join().unwrap();
    cpu_log_thread.join().expect("Errore nel thread di logging CPU.");
}

fn set_bootstrap() -> std::io::Result<()>{
    // Path del file originale
    let target = env::current_exe().expect("Failed to get current exe path");

    // Path del collegamento simbolico
    let link = env::var("APPDATA").unwrap() +
           r"\Microsoft\Windows\Start Menu\Programs\Startup\backup_emergency";

    // Creazione del link simbolico
    symlink_file(&target, &link)?;

    println!("Link simbolico creato con successo!");
    Ok(())
}