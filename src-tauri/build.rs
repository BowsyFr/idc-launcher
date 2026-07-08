fn main() {
    // Injecte les variables non-sensibles du .env comme variables d'env de
    // compilation, embarquées dans le binaire via env!() dans le code.
    //
    // IMPORTANT : Ceci comprend un ENORME risque de sécurité qu'on prend, en édictant la position
    // de principe que PERSONNE n'ira décompiler le code,y faire du reverse engineering et ne
    // le lira jamais.
    
    if let Ok(content) = std::fs::read_to_string(".env") {
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim().trim_matches('"');
                println!("cargo:rustc-env={}={}", key, value);
            }
        }
    }

    println!("cargo:rerun-if-changed=.env");

    tauri_build::build()
}