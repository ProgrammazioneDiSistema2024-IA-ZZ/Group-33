use std::fs::OpenOptions;
use std::thread;
use std::time::Duration;
use sysinfo::{Pid, ProcessesToUpdate, System};
use std::io::Write;

// Funzione che registra l'utilizzo della CPU ogni `interval_seconds` secondi
pub fn log_cpu_usage_periodically(pid: Pid, interval_seconds: u64, log_file_path: &str) {
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

