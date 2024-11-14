use std::env;
use auto_launch::{AutoLaunchBuilder};
use std::path::PathBuf;


pub(crate) fn set_bootstrap()  {
    // Recupera il percorso dell'eseguibile corrente
    let current_path: PathBuf = env::current_exe().expect("Impossibile recuperare il percorso dell'eseguibile");

    // Crea l'oggetto AutoLaunch con il percorso corrente
    let auto_launch = AutoLaunchBuilder::new()
        .set_app_name("backup_emergency")
        .set_app_path(current_path.to_str().unwrap())  // Converte il percorso in stringa
        .set_use_launch_agent(false)  // specifico per macOS
        .build()
        .unwrap();

    // Abilita l'avvio automatico
    auto_launch.enable().expect("Impossibile abilitare l'auto-avvio");

    // Verifica se è abilitato
    if auto_launch.is_enabled().unwrap() {
        println!("L'app si avvierà automaticamente al login.");
    } else {
        println!("L'avvio automatico non è abilitato.");
    }
}


// OLD attempt (windows works)
/*
use std::os::unix::fs::symlink;
fn set_bootstrap() -> std::io::Result<()>{
    #[cfg(target_os = "windows")]
    {
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

    #[cfg(target_os = "macos")]
    {
        // Path del file originale
        let target = env::current_exe().expect("Failed to get current exe path");
        // Path del collegamento simbolico
        let home_dir = env::var("HOME").expect("Could not find home directory");
        //let link = format!("/Library/LaunchAgents");
        //let link = format!("{}/Library/LaunchAgents/backup_emergency", home_dir);
        //let link = format!("/Users/paolomuccilli/Desktop/backup_emergency");
        let link = format!("/Library/LaunchAgents/backup_emergency");
        println!("Ti linko tutto: {:?}", link);

        // Creazione del link simbolico
        symlink(&target, &link)?;

        println!("Link simbolico creato con successo su macOS!");
        Ok(())
    }
}
*/