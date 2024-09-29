use std::{fs, thread, time::Duration};
use std::path::{Path, PathBuf};
use std::time::Instant;
use std::sync::{Arc, Condvar, Mutex};
use std::str;
use std::process::Command;
use std::fs::OpenOptions;
use std::io::Write;
use sysinfo::{System, Pid, ProcessesToUpdate};
use crate::read_files::read_config;
use crate::types::BackupState;

pub fn backup_files( state: Arc<(Mutex<BackupState>, Condvar)>  ) -> Result<(), Box<dyn std::error::Error>> {
    let (lock, cvar) = &*state;
    loop {
        let mut state = lock.lock().unwrap();
        while *state != BackupState::BackingUp {
            state = cvar.wait(state).unwrap();
        }

        let start_time = Instant::now();


        let config = read_config("src/utils/config.toml")?;
        let source = config.backup.source_directory.clone();
        let destination = config.backup.destination_directory.clone();
        let extensions: Vec<&str> = config.backup.file_types.iter().map(|s| s.as_str()).collect();

        let mut source_path = Path::new(&source);
        let mut destination_path = Path::new(&destination);;

        // Crea la directory di destinazione se non esiste
        if !destination_path.exists() {
            fs::create_dir_all(destination_path).unwrap();
        }

        // Avvia la copia ricorsiva dal percorso sorgente
        copy_dir_recursive(source_path, destination_path, &extensions[..]);

        let duration = start_time.elapsed();
        println!("Backup completato in {:?}", duration);

        *state = BackupState::Idle;
        cvar.notify_all();
    }
    Ok(())
}

// Funzione ricorsiva per copiare file e directory
fn copy_dir_recursive(source: &Path, destination: &Path, extensions: &[&str]) {
    // Itera attraverso gli elementi nella directory sorgente
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let entry_path = entry.path();
        let file_name = entry.file_name();
        let dest_path = destination.join(&file_name);

        if entry_path.is_dir() {
            // Se è una directory, creala e copia i contenuti ricorsivamente
            fs::create_dir_all(&dest_path).unwrap();
            copy_dir_recursive(&entry_path, &dest_path, extensions);
        } else {
            // Se è un file, copia solo se l'estensione è nell'elenco o se "*" è presente
            if should_copy_file(&entry_path, extensions) {
                fs::copy(&entry_path, &dest_path).unwrap();
            }
        }
    }
}

// Funzione che determina se il file deve essere copiato in base all'estensione
fn should_copy_file(file_path: &Path, extensions: &[&str]) -> bool {
    if extensions.contains(&"*") {
        return true; // Copia tutto se "*" è presente
    }

    if let Some(extension) = file_path.extension() {
        if let Some(ext_str) = extension.to_str() {
            return extensions.contains(&ext_str); // Copia solo se l'estensione è nell'elenco
        }
    }

    false // Non copiare se l'estensione non corrisponde
}

// Funzione per trovare il primo disco esterno fisico
fn find_external_disk() -> Option<String> {
    // Esegui il comando diskutil list
    let output = Command::new("diskutil")
        .arg("list")
        .output()
        .expect("Errore durante l'esecuzione di diskutil list");

    let stdout = String::from_utf8(output.stdout).unwrap();

    let mut disk_name: Option<String> = None;

    // Cerca la prima occorrenza di un disco esterno fisico
    for line in stdout.lines() {
        if line.contains("external, physical") {
            if let Some(disk) = line.split_whitespace().next() {
                disk_name = Some(disk.to_string());
                break;
            }
        }
    }
    disk_name
}

// Funzione per ottenere il percorso di montaggio del disco
fn get_mount_point(disk: &str) -> Option<String> {
    // Esegui il comando diskutil info per il disco specificato
    let output = Command::new("diskutil")
        .arg("info")
        .arg(format!("{}{}", disk, "s1"))
        .output()
        .expect("Errore durante l'esecuzione di diskutil info");

    let stdout = String::from_utf8(output.stdout).unwrap();

    // Cerca la linea che contiene "Mount Point"
    for line in stdout.lines() {
        if line.contains("Mount Point") {
            let parts: Vec<&str> = line.split(": ").collect();
            if parts.len() == 2 {
                return Some(parts[1].trim().to_string());
            }
        }
    }

    None
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


//old but gold
/*
pub fn backup_files( state: Arc<(Mutex<BackupState>, Condvar)> ) {
    let (lock, cvar) = &*state;
    loop {
        let mut state = lock.lock().unwrap();
        while *state != BackupState::BackingUp {
            state = cvar.wait(state).unwrap();
        }

        //prendere da file//
        let source = "C:\\Users\\maxim\\Desktop\\file_backup\\source_folder".to_string();
        let destination = "C:\\Users\\maxim\\Desktop\\file_backup\\destination_folder".to_string();
        let extensions = vec!["mm", "rs", "jpg", "*"]; // Lista di estensioni da copiare (esempio)

        let start_time = Instant::now();

        let source_path = Path::new(&source);
        let destination_path = Path::new(&destination);

        // Crea la directory di destinazione se non esiste
        if !destination_path.exists() {
            fs::create_dir_all(destination_path).unwrap();
        }

        // Avvia la copia ricorsiva dal percorso sorgente
        copy_dir_recursive(source_path, destination_path, &extensions);

        let duration = start_time.elapsed();

        println!("Backup completato in {:?}", duration);

        *state = BackupState::Idle;
        cvar.notify_all();
    }
}

// Funzione ricorsiva per copiare file e directory
fn copy_dir_recursive(source: &Path, destination: &Path, extensions: &[&str]) {
    // Itera attraverso gli elementi nella directory sorgente
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let entry_path = entry.path();
        let file_name = entry.file_name();
        let dest_path = destination.join(&file_name);

        if entry_path.is_dir() {
            // Se è una directory, creala e copia i contenuti ricorsivamente
            fs::create_dir_all(&dest_path).unwrap();
            copy_dir_recursive(&entry_path, &dest_path, extensions);
        } else {
            // Se è un file, copia solo se l'estensione è nell'elenco o se "*" è presente
            if should_copy_file(&entry_path, extensions) {
                fs::copy(&entry_path, &dest_path).unwrap();
            }
        }
    }
}

// Funzione che determina se il file deve essere copiato in base all'estensione
fn should_copy_file(file_path: &Path, extensions: &[&str]) -> bool {
    if extensions.contains(&"*") {
        return true; // Copia tutto se "*" è presente
    }

    if let Some(extension) = file_path.extension() {
        if let Some(ext_str) = extension.to_str() {
            return extensions.contains(&ext_str); // Copia solo se l'estensione è nell'elenco
        }
    }

    false // Non copiare se l'estensione non corrisponde
}
*/

//util match
/*
match config {
            Ok(config) => {
                println!("Configurazione letta con successo!");
                /*
                println!("Cartella sorgente: {}", config.backup.source_directory);
                println!("Cartella destinazione: {}", config.backup.destination_directory);
                println!("Tipi di file da salvare: {:?}", config.backup.file_types);
                println!("Modalità di backup: {}", config.backup.backup_mode);
                println!("Intervallo di log CPU: {} secondi", config.cpu_logging.interval_seconds);
                */
                source_path = Path::new(&config.backup.source_directory);
                destination_path = Path::new(&config.backup.destination_directory);
                // Convert Vec<String> to Vec<&str>
                let vec_of_str: Vec<&str> = config.backup.file_types.iter().map(|s| s.as_str()).collect();
                // Convert Vec<&str> to &[&str]
                extensions = &vec_of_str;
            },
            Err(e) => {
                println!("Errore nella lettura della configurazione: {}", e);
            }
        }
*/