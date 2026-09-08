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

/// Plafond d'écriture d'un fichier de journal, en octets (8 Mio).
///
/// La rotation a lieu au démarrage : une session qui dure des semaines, ou qu'on relance
/// avec `RUST_LOG=debug`, alimentait sinon un fichier que rien ne bornait. Huit mégaoctets
/// tiennent largement une session de diagnostic tout en restant joignables à un
/// signalement.
const MAX_LOG_BYTES: u64 = 8 * 1024 * 1024;

/// Ligne déposée à la place de la suite du journal une fois le plafond atteint.
const LIMIT_NOTICE: &[u8] =
    b"[journal] plafond de 8 Mio atteint : la suite de cette session n'est pas enregistree.\n";

/// Fichier de journal qui cesse d'écrire au-delà de [`MAX_LOG_BYTES`].
///
/// Se taire est ici préférable à échouer : perdre la fin d'un journal n'empêche pas
/// l'application de fonctionner, alors qu'une erreur d'écriture remontée à chaque trace
/// remplirait la sortie standard de bruit.
struct BoundedLogWriter {
    file: std::fs::File,
    written: u64,
    saturated: bool,
}

impl BoundedLogWriter {
    fn new(file: std::fs::File) -> Self {
        let written = file.metadata().map(|metadata| metadata.len()).unwrap_or(0);
        Self {
            file,
            written,
            saturated: false,
        }
    }
}

impl std::io::Write for BoundedLogWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if self.saturated {
            return Ok(buf.len());
        }
        if self.written >= MAX_LOG_BYTES {
            self.saturated = true;
            self.file.write_all(LIMIT_NOTICE)?;
            return Ok(buf.len());
        }
        let remaining = MAX_LOG_BYTES.saturating_sub(self.written) as usize;
        let written = self.file.write(&buf[..buf.len().min(remaining)])?;
        self.written = self.written.saturating_add(written as u64);
        Ok(written)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.file.flush()
    }
}

/// Guard de vidage du journal fichier. À conserver vivante jusqu'à l'arrêt du programme :
/// l'écriture étant tamponnée dans un fil dédié, la relâcher trop tôt perd les dernières
/// lignes — précisément celles qui décrivent un arrêt anormal.
pub struct GuardJournal(Option<tracing_appender::non_blocking::WorkerGuard>);

impl GuardJournal {
    /// Un journal fichier est-il actif ? Faux quand le dossier de données est inaccessible.
    #[must_use]
    pub const fn ecrit_dans_un_fichier(&self) -> bool {
        self.0.is_some()
    }
}

/// Installe le journal. Sans dossier de données accessible, la sortie standard seule est
/// utilisée : ne pas pouvoir journaliser ne doit jamais empêcher l'application de démarrer.
#[must_use]
pub fn init() -> GuardJournal {
    let filter = || {
        tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("candilog=info"))
    };

    let Some(file) = ouvrir_file() else {
        tracing_subscriber::fmt().with_env_filter(filter()).init();
        tracing::warn!("journal fichier indisponible : sortie standard seule");
        return GuardJournal(None);
    };

    let (ecriture, guard) = tracing_appender::non_blocking(BoundedLogWriter::new(file));
    tracing_subscriber::registry()
        .with(filter())
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(ecriture)
                .with_ansi(false),
        )
        .init();
    tracing::info!(version = env!("CARGO_PKG_VERSION"), "démarrage de Candilog");
    if let Ok(paths) = AppPaths::discover() {
        signaler_session_precedente(&paths);
    }
    GuardJournal(Some(guard))
}

/// Nom du marqueur de session vivante, déposé à côté du journal.
const MARQUEUR_SESSION: &str = "candilog.session";

/// Ouvre une session et indique si la précédente s'est terminée brutalement.
///
/// Un arrêt par le noyau (OOM killer) ne laisse écrire aucune ligne : le journal s'arrête
/// net, sans erreur. Le marqueur survivant est alors la seule preuve exploitable qu'une
/// session n'est pas allée à son terme.
fn ouvrir_session(marqueur: &std::path::Path) -> bool {
    let interrompue = marqueur.exists();
    let _ = std::fs::write(marqueur, "");
    interrompue
}

/// Retire le marqueur : la session s'est terminée normalement.
fn fermer_session(marqueur: &std::path::Path) {
    let _ = std::fs::remove_file(marqueur);
}

/// Signale dans le journal qu'une session précédente ne s'est pas terminée proprement.
pub fn signaler_session_precedente(paths: &AppPaths) {
    if ouvrir_session(&paths.data_dir.join(MARQUEUR_SESSION)) {
        tracing::warn!(
            "session précédente terminée brutalement : arrêt par le système (mémoire \
             insuffisante) ou plantage. Le journal précédent s'interrompt sans erreur."
        );
    }
}

/// Marque l'arrêt normal de la session courante.
pub fn cloturer_session() {
    let Ok(paths) = AppPaths::discover() else {
        return;
    };
    fermer_session(&paths.data_dir.join(MARQUEUR_SESSION));
}

/// Ouvre `candilog.log` sous le dossier de données, après avoir fait tourner les précédents.
///
/// L'écriture passe ensuite par [`BoundedLogWriter`], qui en plafonne la taille.
fn ouvrir_file() -> Option<std::fs::File> {
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
