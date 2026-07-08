use portablemc::forge::{self, Loader, Version as ForgeVersion};
use std::path::PathBuf;

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

/// Installe (si besoin) et lance Minecraft 1.21.1 + NeoForge 21.1.232
/// pour le pseudo donné.
///
/// ATTENTION : cette fonction est bloquante (installation + attente de
/// fermeture du process Java). Elle doit toujours être appelée depuis
/// `tokio::task::spawn_blocking`, jamais directement dans une commande
/// `async` — sinon elle gèlerait le runtime tokio partagé avec le reste
/// de l'app (auth Discord, DB...).
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