use portablemc::forge::{self, Loader, Version as ForgeVersion};
use std::path::PathBuf;
use std::sync::Arc;
use crate::resources::ResourceManager;

/// Dossier où va vivre notre launcher, positionné au même niveau que le
/// dossier `.minecraft` du launcher officiel, selon l'OS :
/// - Windows : `%APPDATA%\idc-launcher`      (à côté de `%APPDATA%\.minecraft`)
/// - macOS   : `~/Library/Application Support/idc-launcher` (à côté de `.../minecraft`)
/// - Linux   : `~/idc-launcher`               (à côté de `~/.minecraft`)
fn dossier_jeu() -> Result<PathBuf, String> {
    #[cfg(target_os = "linux")]
    let base = dirs::home_dir();

    #[cfg(not(target_os = "linux"))]
    let base = dirs::config_dir();

    let base = base.ok_or_else(|| "Impossible de déterminer le dossier utilisateur".to_string())?;
    Ok(base.join("idc-launcher"))
}

pub fn lancer_jeu_bloquant(pseudo: &str) -> Result<(), String> {
    let dossier = dossier_jeu()?;

    // portablemc canonicalise ce chemin en interne (voir l'erreur
    // "No such file or directory @ canonicalize") : il DOIT déjà exister
    // sur le disque avant qu'on le passe à `set_mc_dir`.
    std::fs::create_dir_all(&dossier).map_err(|e| {
        format!("Impossible de créer le dossier du jeu ({}) : {e}", dossier.display())
    })?;

    let mut installer = forge::Installer::new(Loader::NeoForge, ForgeVersion::Name("21.1.232".to_string()));

    {
        let mojang = installer.mojang_mut();
        mojang.set_version("1.21.1");
        mojang.set_auth_offline_username(pseudo);
        mojang.base_mut().set_mc_dir(dossier);
    }

    let game = installer
        .install(())
        .map_err(|e| format!("Erreur lors de l'installation : {e}"))?;

    let status = game
        .spawn_and_wait()
        .map_err(|e| format!("Erreur lors du lancement du jeu : {e}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("Le jeu s'est fermé avec un code d'erreur : {status}"))
    }
}

/// Synchronise les ressources et lance le jeu (version asynchrone)
/// Cette fonction doit être appelée depuis un contexte async (comme une commande Tauri)
pub async fn lancer_jeu_avec_ressources(
    pseudo: String,
    resource_manager: Arc<ResourceManager>,
) -> Result<(), String> {
    // Synchroniser les ressources avant de lancer le jeu
    let sync_result = resource_manager.sync_game_resources().await
        .map_err(|e| format!("Erreur de synchronisation des ressources : {}", e))?;
    
    // Log des résultats de synchronisation
    if !sync_result.downloaded.is_empty() {
        println!("Fichiers téléchargés : {:?}", sync_result.downloaded);
    }
    if !sync_result.updated.is_empty() {
        println!("Fichiers mis à jour : {:?}", sync_result.updated);
    }
    if !sync_result.deleted.is_empty() {
        println!("Fichiers supprimés : {:?}", sync_result.deleted);
    }
    if !sync_result.errors.is_empty() {
        println!("Erreurs de synchronisation : {:?}", sync_result.errors);
    }
    
    // Lancer le jeu dans un thread bloquant
    tokio::task::spawn_blocking(move || lancer_jeu_bloquant(&pseudo))
        .await
        .map_err(|e| format!("Erreur interne lors du lancement : {}", e))??;
    
    Ok(())
}