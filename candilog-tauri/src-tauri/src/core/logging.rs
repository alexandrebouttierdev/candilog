//! Journal applicatif : sortie standard **et** fichier tournant.
//!
//! Une application de bureau lancée depuis un menu n'a pas de terminal visible : un journal
//! qui n'existe que sur la sortie standard n'existe pas. Le fichier, lui, permet à
//! l'utilisateur de joindre un diagnostic après un incident, et au mainteneur de reconstituer
//! une séquence d'événements — migrations appliquées, échecs d'écriture, appels IA, erreurs
//! remontées à l'écran.

use crate::core::config::AppPaths;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

/// Nombre de fichiers de journal conservés, le courant compris.
const JOURNAUX_CONSERVES: usize = 5;

/// Garde de vidage du journal fichier. À conserver vivante jusqu'à l'arrêt du programme :
/// l'écriture étant tamponnée dans un fil dédié, la relâcher trop tôt perd les dernières
/// lignes — précisément celles qui décrivent un arrêt anormal.
pub struct GardeJournal(Option<tracing_appender::non_blocking::WorkerGuard>);

impl GardeJournal {
    /// Un journal fichier est-il actif ? Faux quand le dossier de données est inaccessible.
    #[must_use]
    pub const fn ecrit_dans_un_fichier(&self) -> bool {
        self.0.is_some()
    }
}

/// Installe le journal. Sans dossier de données accessible, la sortie standard seule est
/// utilisée : ne pas pouvoir journaliser ne doit jamais empêcher l'application de démarrer.
#[must_use]
pub fn initialiser() -> GardeJournal {
    let filtre = || {
        tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("candilog=info"))
    };

    let Some(fichier) = ouvrir_fichier() else {
        tracing_subscriber::fmt().with_env_filter(filtre()).init();
        tracing::warn!("journal fichier indisponible : sortie standard seule");
        return GardeJournal(None);
    };

    let (ecriture, garde) = tracing_appender::non_blocking(fichier);
    tracing_subscriber::registry()
        .with(filtre())
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(ecriture)
                .with_ansi(false),
        )
        .init();
    tracing::info!(version = env!("CARGO_PKG_VERSION"), "démarrage de Candilog");
    GardeJournal(Some(garde))
}

/// Ouvre `candilog.log` sous le dossier de données, après avoir fait tourner les précédents.
fn ouvrir_fichier() -> Option<std::fs::File> {
    let paths = AppPaths::discover().ok()?;
    let journal = paths.data_dir.join("candilog.log");
    faire_tourner(&journal);
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&journal)
        .ok()
}

/// Décale `candilog.log` vers `candilog.log.1`, `.1` vers `.2`, et supprime le plus ancien.
///
/// Rotation au démarrage plutôt qu'à la taille : une session correspond ainsi à un fichier,
/// ce qui est la granularité utile pour joindre un journal à un signalement.
fn faire_tourner(journal: &std::path::Path) {
    if !journal.exists() {
        return;
    }
    let numerote = |rang: usize| journal.with_extension(format!("log.{rang}"));
    let _ = std::fs::remove_file(numerote(JOURNAUX_CONSERVES - 1));
    for rang in (1..JOURNAUX_CONSERVES - 1).rev() {
        let _ = std::fs::rename(numerote(rang), numerote(rang + 1));
    }
    let _ = std::fs::rename(journal, numerote(1));
}

#[cfg(test)]
#[path = "tests/logging/mod.rs"]
mod tests;
