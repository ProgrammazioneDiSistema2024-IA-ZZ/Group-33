use std::{fs, thread, time::Duration};
use std::path::{Path, PathBuf};
use std::time::Instant;
use std::sync::{Arc, Condvar, Mutex};
use std::str;
use std::process::Command;
use std::fs::OpenOptions;
use std::io::Write;
use sysinfo::{System, Pid, ProcessesToUpdate};
use regex::Regex;
use chrono::Local;
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
        let destination = if cfg!(target_os = "windows") {
            find_external_disk_win(&source).unwrap_or(config.backup.destination_directory.clone())  // Richiama la funzione per Windows
        } else if cfg!(target_os = "macos") {
            get_mount_point_macos(&find_external_disk_macos().unwrap()).unwrap_or(config.backup.destination_directory.clone())  // Richiama la funzione per macOS
        } else if cfg!(target_os = "linux") {
            find_usb_partition_mountpoint().unwrap_or(config.backup.destination_directory.clone())  // Richiama la funzione per Ubuntu
        }
         else {
            panic!("Unsupported operating system!");
        };

        println!("{}", &destination);

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


// WINDOWS
fn find_external_disk_win(source_path:&str) -> Option<String> {
    let output = Command::new("wmic")
        .arg("diskdrive")
        .arg("get")
        .arg("deviceid,mediatype")
        .output()
        .expect("Errore durante l'esecuzione di wmic diskdrive");

    let stdout = String::from_utf8(output.stdout).unwrap();

    let mut tmp_device_id: Option<String> = None;

    for line in stdout.lines() {
        if line.contains("Removable Media") {
            if let Some(id) = line.split_whitespace().nth(0) {
                tmp_device_id = Some(id.to_string());
                break;
            }
        }
    }

    let device_id;
    if(tmp_device_id.is_none()){
        return None;
    }else {
        device_id = tmp_device_id.unwrap();
    }

    let output = Command::new("wmic")
        .arg("path")
        .arg("Win32_DiskDriveToDiskPartition")
        .output()
        .expect("Errore durante l'esecuzione di wmic DiskDriveToDiskPartition");

    let stdout = String::from_utf8(output.stdout).unwrap();

    let mut partition_id: String = "NONE".to_string();

    for line in stdout.lines() {
        if line.contains(&device_id) {
            let mut iter = line.split_whitespace();
            let first = iter.next().unwrap();
            // Ottieni il resto della stringa dopo il primo spazio
            partition_id = iter.collect::<Vec<&str>>().join(" ");
        }
    }

    let output = Command::new("wmic")
        .arg("path")
        .arg("Win32_LogicalDiskToPartition")
        .output()
        .expect("Errore durante l'esecuzione di wmic LogicalDiskToPartition");


    let stdout = String::from_utf8(output.stdout).unwrap();
    let re = Regex::new(r#"DeviceID="([A-Z]:)""#).unwrap();

    for line in stdout.lines() {
        if line.contains(&partition_id) {
            if let Some(caps) = re.captures(line) {
                // Ottieni la lettera del disco
                let disk_letter = &caps[1];
                let mut full_path = disk_letter.to_string();
                full_path.push_str("\\backup (");
                let today = Local::now();
                let original_folder = source_path.split("\\").last().unwrap();
                let formatted_date = today.format("%Y-%m-%d").to_string();

                // Concatenala alla stringa
                full_path.push_str(&original_folder);
                full_path.push_str(") ");
                full_path.push_str(&formatted_date);
                return Some(full_path);
            } else {
                println!("Lettera del disco non trovata.");
                return None;
            }

        }
    }

    None
}
// MACOS
// Funzione per ottenere il percorso di montaggio del disco
fn get_mount_point_macos(disk: &str) -> Option<String> {
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
fn find_external_disk_macos() -> Option<String> {
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
// LINUX
fn get_mount_point_linux(disk: &str) -> Option<String> {
    // Esegui il comando lsblk per ottenere il punto di montaggio del disco specificato
    let output = Command::new("lsblk")
        .arg("-o")
        .arg("MOUNTPOINT")
        .arg(format!("/dev/{}", disk))
        .output()
        .expect("Errore durante l'esecuzione di lsblk");

    let stdout = String::from_utf8(output.stdout).unwrap();

    // Cerca la linea che contiene il mount point
    for line in stdout.lines() {
        println!("{}", line);
        let trimmed_line = line.trim();
        if !trimmed_line.is_empty() {
            return Some(trimmed_line.to_string());
        }
    }

    None
}
fn find_usb_partition_mountpoint() -> Option<String> {
    // Esegui il comando lsblk con l'opzione -o NAME,TYPE,TRAN,MOUNTPOINT per ottenere i dispositivi, i tipi e i punti di mount
    let output = Command::new("lsblk")
        .arg("-o")
        .arg("NAME,TYPE,TRAN,MOUNTPOINT")
        .output()
        .expect("Errore durante l'esecuzione di lsblk");

    let stdout = String::from_utf8(output.stdout).unwrap();


    let mut current_device_is_usb = false;

    // Cerchiamo tutti i dispositivi esterni (collegati via USB, TRAN == "usb") e le partizioni montate
    for line in stdout.lines() {
        // Split the line into columns (NAME, TYPE, TRAN, MOUNTPOINT)
        let columns: Vec<&str> = line.split_whitespace().collect();

        // Controlla se la linea rappresenta un "disk" di tipo "usb"
        if columns.len() >= 3 && columns[1] == "disk" && columns[2] == "usb" {
            current_device_is_usb = true;  // Imposta un flag per indicare che il disco è USB
        }

        // Se la linea rappresenta una partizione e appartiene a un disco USB, prendi il mountpoint
        if columns.len() == 3 && columns[1] == "part" && current_device_is_usb && !columns[2].is_empty() {
            // Ritorna subito il primo mountpoint trovato
            return Some(columns[2].to_string());
        }

        // Resetta il flag per dispositivi non USB
        if columns.len() >= 3 && columns[1] == "disk" && columns[2] != "usb" {
            current_device_is_usb = false;
        }
    }

    // Nessun mountpoint USB trovato
    None
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

