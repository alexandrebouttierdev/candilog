//! Fusion du profil et de la lettre pour l'export PDF.

use crate::features::documents::domain::ExportLettre;
use crate::features::profil::domain::Profil;
use crate::infrastructure::pdf::LettrePdf;

/// Identité du profil + objet et corps de la lettre.
#[must_use]
pub fn construire_lettre(profil: &Profil, lettre: &ExportLettre) -> LettrePdf {
    let identite = &profil.identite;
    let nom = format!("{} {}", identite.prenom, identite.nom)
        .trim()
        .to_owned();
    let objet = match (lettre.poste.as_deref(), lettre.entreprise.as_deref()) {
        (Some(poste), Some(entreprise))
            if !poste.trim().is_empty() && !entreprise.trim().is_empty() =>
        {
            format!("Objet : candidature au poste de {poste} — {entreprise}")
        }
        (Some(poste), _) if !poste.trim().is_empty() => {
            format!("Objet : candidature au poste de {poste}")
        }
        _ => "Objet : candidature".into(),
    };
    LettrePdf {
        nom,
        ville: identite.ville.clone(),
        email: identite.email.clone(),
        objet,
        corps: lettre.contenu.clone(),
    }
}

#[cfg(test)]
#[path = "tests/lettre_document/mod.rs"]
mod tests;
