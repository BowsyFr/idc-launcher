mod database;
mod discord_auth;
mod callback_server;
mod game;
mod commands;

use database::Database;
use discord_auth::DiscordAuth;
use callback_server::CallbackServer;
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex;

/// État partagé entre toutes les commandes Tauri.
/// `db` est `None` tant que la connexion asynchrone n'est pas terminée
/// (voir le hook `.setup()` plus bas).
///
/// `discord_auth` est dans un `Arc` car il doit maintenant être partagé
/// avec le serveur de callback : ce dernier échange lui-même le code
/// contre un token dès qu'il arrive, pour pouvoir afficher le pseudo et
/// l'avatar directement sur la page de confirmation dans le navigateur.
pub struct AppState {
    pub db: Mutex<Option<Database>>,
    pub discord_auth: Arc<DiscordAuth>,
    pub callback_server: CallbackServer,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Plus besoin de dotenv::dotenv().ok() ici : les valeurs du .env sont
    // maintenant embarquées au moment de la compilation (voir build.rs +
    // discord_auth.rs / database.rs), donc disponibles au runtime sans
    // dépendre d'un fichier .env présent à côté du binaire.

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            db: Mutex::new(None),
            discord_auth: Arc::new(DiscordAuth::new()),
            callback_server: CallbackServer::new(),
        })
        .setup(|app| {
            let handle = app.handle().clone();

            // La connexion à MySQL est async : on la lance en tâche de fond
            // et on remplit `AppState.db` une fois prête, sans bloquer le
            // démarrage de la fenêtre.
            tauri::async_runtime::spawn(async move {
                match Database::new().await {
                    Ok(database) => {
                        if let Err(e) = database.create_tables().await {
                            eprintln!("Erreur lors de la création des tables : {e}");
                        }
                        let state = handle.state::<AppState>();
                        *state.db.lock().await = Some(database);
                        println!("Base de données connectée.");
                    }
                    Err(e) => {
                        eprintln!("Impossible de se connecter à la base de données : {e}");
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::start_discord_auth,
            commands::complete_discord_auth,
            commands::get_user_by_discord_id,
            commands::create_user,
            commands::launch_game,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}