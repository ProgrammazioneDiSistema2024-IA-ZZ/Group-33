use std::{fs, thread};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Instant;

pub fn backup_files( state: Arc<(Mutex<u32>, Condvar)> ) {
    let (lock, cvar) = &*state;
    loop {
        let mut state = lock.lock().unwrap();
        while *state != 2 {
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

        *state = 0;
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
