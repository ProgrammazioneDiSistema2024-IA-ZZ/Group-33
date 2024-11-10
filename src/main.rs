mod mouse;
mod window;
mod backup;
mod read_files;
mod types;

use std::sync::{Arc, Mutex, Condvar};
use std::{fs, thread, time::Duration};
use std::fs::OpenOptions;
use std::io::Write;
use sysinfo::{System, Pid, ProcessesToUpdate};
use crate::types::BackupState;

fn main() {
    // Ottieni il PID del processo corrente
    let process_id = std::process::id();
    let pid = Pid::from(process_id as usize);

    println!("ID porcesso = {}",pid);

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
        log_cpu_usage_periodically(pid, 5, "cpu_usage.log"); // 120 secondi = 2 minuti
    });

    //finestra con loop per conferma backup
    let state_clone = Arc::clone(&state);
    window::make_window(state_clone);

    // Unisci tutti i thread al main thread
    mouse_thread.join().unwrap();
    backup_thread.join().unwrap();
    cpu_log_thread.join().expect("Errore nel thread di logging CPU.");
}

// Funzione che registra l'utilizzo della CPU ogni `interval_seconds` secondi
fn log_cpu_usage_periodically(pid: Pid, interval_seconds: u64, log_file_path: &str) {
    let mut sys = System::new_all();

    loop {
        let cpu_usage = get_cpu_usage(&mut sys, pid);

        // Scrivi l'utilizzo della CPU nel file di log
        /*
        let log_entry = format!("CPU usage: {}%\n", cpu_usage);
        if let Err(e) = append_to_log(log_file_path, &log_entry) {
            eprintln!("Errore durante la scrittura nel file di log: {}", e);
        }

         */
        println!("{}", format!("CPU usage: {}%\n", cpu_usage));

        // Attendi per `interval_seconds` secondi
        thread::sleep(Duration::from_secs(interval_seconds));
    }
}

// Funzione per aggiungere una voce al file di log
fn append_to_log(file_path: &str, content: &str) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)    // Crea il file se non esiste
        .append(true)    // Aggiunge al file esistente
        .open(file_path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

// Function to get CPU usage of the current process
fn get_cpu_usage(sys: &mut System, pid: Pid) -> f32 {
    sys.refresh_processes(ProcessesToUpdate::All); // Refresh process info
    if let Some(process) = sys.process(pid) {
        process.cpu_usage() // Return the CPU usage of the process
    } else {
        0.0 // Process not found
    }
}

