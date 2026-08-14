//! État transversal de notification et de capture.

pub use crate::ui::components::notification::Kind as NotificationKind;

/// Le harnais de capture visuelle est-il demandé ?
///
/// Toujours faux sans la caractéristique Cargo `capture` : le binaire distribué ne doit ni lire
/// ces variables, ni écrire de fichier au chemin qu'elles désignent.
#[must_use]
pub fn capture_demandee() -> bool {
    cfg!(feature = "capture") && std::env::var_os("CANDILOG_CAPTURE_PATH").is_some()
}

/// Message adressé à l'utilisateur, **avec** sa nature.
///
/// La nature accompagne le texte au lieu d'être redevinée au moment du rendu : toutes les
/// erreurs étaient converties en `String` dès `update()`, puis reclassées par recherche de
/// mots-clés, avec `Success` pour cas par défaut — un échec sur deux s'affichait en vert.
#[derive(Debug, Clone, PartialEq)]
pub struct Notification {
    /// Nature, qui détermine le ton et l'icône du toast.
    pub kind: NotificationKind,
    /// Texte affiché, déjà destiné à l'utilisateur.
    pub message: String,
}
