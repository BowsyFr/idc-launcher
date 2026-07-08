const { invoke } = window.__TAURI__.core;
const { openUrl } = window.__TAURI__.opener;
const tauriWindow = window.__TAURI__.window;

const ecranConnexion = document.getElementById("ecran-connexion");
const ecranPseudo = document.getElementById("ecran-pseudo");
const ecranConnecte = document.getElementById("ecran-connecte");
const statutAuth = document.getElementById("statut-auth");
const erreur = document.getElementById("erreur");
const avatarInitiale = document.getElementById("avatar-initiale");

const btnDiscord = document.getElementById("btn-discord");
const btnValiderPseudo = document.getElementById("btn-valider-pseudo");
const btnJouer = document.getElementById("btn-jouer");

let discordUserCourant = null;
let usernameCourant = null;

// Écrans de connexion/pseudo : format vertical.
// Écran de jeu : format fenêtre classique, comme un launcher normal.
const TAILLE_VERTICALE = { largeur: 440, hauteur: 680 };
const TAILLE_JEU = { largeur: 1100, hauteur: 650 };

async function definirTailleFenetre(largeur, hauteur) {
  try {
    if (!tauriWindow) return;
    const fenetre = tauriWindow.getCurrentWindow();
    await fenetre.setSize(new tauriWindow.LogicalSize(largeur, hauteur));
    await fenetre.center();
  } catch (e) {
    console.warn("Impossible de redimensionner la fenêtre :", e);
  }
}

function afficherEcran(ecran) {
  [ecranConnexion, ecranPseudo, ecranConnecte].forEach((e) => e.classList.add("cache"));
  ecran.classList.remove("cache");

  if (ecran === ecranConnecte) {
    definirTailleFenetre(TAILLE_JEU.largeur, TAILLE_JEU.hauteur);
  } else {
    definirTailleFenetre(TAILLE_VERTICALE.largeur, TAILLE_VERTICALE.hauteur);
  }
}

function afficherErreur(message) {
  erreur.textContent = message;
  erreur.classList.remove("cache");
}

function definirProfil(nom) {
  document.getElementById("texte-connecte").textContent = `Connecté en tant que ${nom}`;
  avatarInitiale.textContent = (nom[0] || "?").toUpperCase();
}

// La fenêtre démarre au format vertical (écran de connexion).
definirTailleFenetre(TAILLE_VERTICALE.largeur, TAILLE_VERTICALE.hauteur);

btnDiscord.addEventListener("click", async () => {
  erreur.classList.add("cache");
  statutAuth.textContent = "Ouverture du navigateur...";

  try {
    const authUrl = await invoke("start_discord_auth");
    await openUrl(authUrl);

    statutAuth.textContent = "En attente de la connexion dans le navigateur...";
    const discordUser = await invoke("complete_discord_auth");
    discordUserCourant = discordUser;

    const existingUser = await invoke("get_user_by_discord_id", {
      discordId: discordUser.id,
    });

    if (existingUser) {
      usernameCourant = existingUser.username;
      definirProfil(existingUser.username);
      afficherEcran(ecranConnecte);
    } else {
      document.getElementById("bienvenue-discord").textContent =
          `Bienvenue, ${discordUser.username} ! Choisis ton pseudo Minecraft :`;
      afficherEcran(ecranPseudo);
    }
  } catch (e) {
    afficherErreur(`Erreur : ${e}`);
    statutAuth.textContent = "";
  }
});

btnValiderPseudo.addEventListener("click", async () => {
  const pseudo = document.getElementById("input-pseudo").value.trim();
  if (!pseudo) return;

  try {
    const user = await invoke("create_user", {
      discordId: discordUserCourant.id,
      username: pseudo,
    });
    usernameCourant = user.username;
    definirProfil(user.username);
    afficherEcran(ecranConnecte);
  } catch (e) {
    afficherErreur(`Erreur : ${e}`);
  }
});

btnJouer.addEventListener("click", async () => {
  if (!usernameCourant) return;

  erreur.classList.add("cache");
  btnJouer.disabled = true;
  const contenuOriginal = btnJouer.innerHTML;
  btnJouer.textContent = "Installation / Lancement...";

  try {
    await invoke("launch_game", { username: usernameCourant });
  } catch (e) {
    afficherErreur(`Erreur : ${e}`);
  } finally {
    btnJouer.disabled = false;
    btnJouer.innerHTML = contenuOriginal;
  }
});