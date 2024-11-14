// Disabilita la console su Windows, eseguendo l'app in modalità GUI
//#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod mouse;
mod window;
mod backup;

mod read_files;
mod types;
mod performance;
mod bootstrap;

use std::sync::{Arc, Mutex, Condvar};
use std::thread;
use std::env;
use std::path::{Path, PathBuf};
use sysinfo::Pid;
use crate::types::BackupState;
use std::fs::File;
use std::io::{self, BufRead};
use std::fs;

use toml::{self, Value};
use dirs::document_dir;
use crate::read_files::{read_config, BackupConfig};
use crate::bootstrap::set_bootstrap;

#[cfg(target_os = "windows")]
#use std::os::windows::fs::symlink_file;

#[cfg(target_os = "macos")]
use std::os::unix::fs::symlink;

fn main() {
    // get argument from command line to set the config file (if needed)
    let args: Vec<String> = env::args().collect();
    // Se il primo argomento (dopo il nome del programma) è presente, usalo come percorso,
    // altrimenti usa il percorso di default
    if args.len() == 2 {
        match read_lines_to_vec(&args[1]) {
            Ok(lines) => {
                /*
                for (index, line) in lines.iter().enumerate() {
                    println!("Line {}: {}", index + 1, line);
                }
                */
                if let Err(e) = update_config_file(lines, "src/utils/config.toml") {
                    eprintln!("Errore nell'aggiornamento del file di configurazione: {}", e);
                } else {
                    println!("File di configurazione aggiornato correttamente!");
                }
            }
            Err(e) => {
                eprintln!("Error reading file: {}", e);
            }
        }
    };

    //AGGIUNGERE IF PER SO
    // Chiama set_bootstrap e gestisce eventuali errori
    /*
    if let Err(e) = set_bootstrap() {
        eprintln!("Errore durante la creazione del link simbolico: {}", e);
    }
    */
    set_bootstrap();

    // Ottieni il PID del processo corrente
    let process_id = std::process::id();
    let pid = Pid::from(process_id as usize);
    println!("ID process = {}",pid);

    //lettura file configurazione
    let mut source_cpu_logging = String::from("C:\\Default\\LogPath\\log.txt"); // Valore predefinito per `source_cpu_logging`;
    let mut config_backup = BackupConfig::default(); // Usa il valore predefinito
    match read_config("src/utils/config.toml") {
        Ok(config) => {
            source_cpu_logging = config.cpu_logging.log_path.clone();
            config_backup = config.backup.clone();
        }, // Configurazione caricata con successo
        Err(e) => {
            eprintln!("Errore durante la lettura del file di configurazione: {}", e);
        }
    };

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
        if let Err(e) = backup::backup_files(state_clone, config_backup ) {
            eprintln!("Errore durante la creazione del backup {}", e);
        }
    });

    // Thread per il logging del consumo di CPU
    // Avvia un thread separato per registrare il consumo di CPU ogni 2 minuti
    let cpu_log_thread = thread::spawn( move || {
        performance::log_cpu_usage_periodically(pid, 120, &source_cpu_logging); // 120 secondi = 2 minuti
    });

    //finestra con loop per conferma backup
    let state_clone = Arc::clone(&state);
    window::make_window(state_clone);

    // Unisci tutti i thread al main thread
    mouse_thread.join().unwrap();
    backup_thread.join().unwrap();
    cpu_log_thread.join().expect("Errore nel thread di logging CPU.");
}


fn read_lines_to_vec(file_path: &str) -> io::Result<Vec<String>> {
    let path = Path::new(file_path);
    let file = File::open(&path)?;

    let buffered = io::BufReader::new(file);
    let lines: Vec<String> = buffered
        .lines()
        .collect::<Result<_, _>>()?;

    Ok(lines)
}

fn update_config_file(values: Vec<String>, config_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Verifica che `values` contenga esattamente 3 elementi
    if values.len() < 2 {
        return Err("Il vettore deve contenere esattamente 3 elementi".into());
    }

    // Carica il contenuto del file di configurazione
    let config_content = fs::read_to_string(config_path)?;
    let mut config: Value = config_content.parse()?;

    // Assegna i valori rispettivi
    if let Some(backup_section) = config.get_mut("backup") {
        backup_section["source_directory"] = Value::String(values[0].clone());

        // Divide il secondo valore in base alla virgola e rimuove spazi vuoti
        let file_types: Vec<Value> = values[1]
            .split(',')
            .map(|s| Value::String(s.trim().to_string()))
            .collect();

        backup_section["file_types"] = Value::Array(file_types);
    }

    if values.len() == 3 {
        if let Some(cpu_logging_section) = config.get_mut("cpu_logging") {
            cpu_logging_section["log_path"] = Value::String(values[2].clone());
        }
    }else{
        #[cfg(target_os = "windows")]
        if let Some(cpu_logging_section) = config.get_mut("cpu_logging") {
            // Ottieni il percorso della cartella Documenti e assegnalo a una variabile
            let mut documents_path: PathBuf = document_dir().expect("Impossibile trovare la cartella Documenti");
            // Aggiungi il nome del file "prova.txt" al percorso
            documents_path.push("performance_cpu.txt");
            // Converte `PathBuf` in `String` prima di assegnarlo
            cpu_logging_section["log_path"] = Value::String(documents_path.to_string_lossy().into_owned());
        }
        #[cfg(any(target_os = "macos", target_os = "linux"))]
        if let Some(cpu_logging_section) = config.get_mut("cpu_logging") {
            // Ottieni il percorso della cartella Documenti e assegnalo a una variabile
            let mut documents_path: PathBuf = document_dir().expect("Impossibile trovare la cartella Documenti");
            // Aggiungi il nome del file "prova.txt" al percorso
            documents_path.push("performance_cpu.txt");
            // Converte `PathBuf` in `String` prima di assegnarlo
            cpu_logging_section["log_path"] = Value::String(documents_path.to_string_lossy().into_owned());
        }

    }

    // Scrive il contenuto aggiornato nel file `config.toml`
    let updated_content = toml::to_string(&config)?;
    fs::write(config_path, updated_content)?;

    Ok(())
}
