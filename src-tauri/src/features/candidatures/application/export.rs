//! Export CSV des candidatures.

use crate::core::errors::{AppError, AppResult};
use crate::features::candidatures::domain::Candidature;

/// Sérialise des candidatures en CSV, séparateur point-virgule.
///
/// Le point-virgule et non la virgule : c'est ce qu'attend Excel en locale française, où un
/// fichier séparé par des virgules s'ouvre en une seule colonne. L'export vise d'abord une
/// relecture par l'utilisateur, pas un import machine.
///
/// # Errors
/// Retourne `AppError::Serialization` si l'écriture échoue.
pub fn vers_csv(candidatures: &[Candidature]) -> AppResult<String> {
    let mut writer = csv::WriterBuilder::new()
        .delimiter(b';')
        .from_writer(Vec::new());

    fn erreur(contexte: &'static str) -> impl Fn(csv::Error) -> AppError {
        move |error| AppError::Serialization(format!("{contexte} : {error}"))
    }

    writer
        .write_record([
            "poste",
            "entreprise",
            "ville",
            "contrat",
            "statut",
            "date_envoi",
            "lien_offre",
            "notes",
        ])
        .map_err(erreur("en-tête CSV"))?;

    for ligne in candidatures {
        writer
            .write_record([
                ligne.poste.as_str(),
                ligne.entreprise_nom.as_deref().unwrap_or_default(),
                ligne.entreprise_ville.as_deref().unwrap_or_default(),
                &ligne.type_contrat.to_string(),
                &ligne.statut.to_string(),
                ligne.date_envoi.as_str(),
                ligne.lien_offre.as_deref().unwrap_or_default(),
                ligne.notes.as_deref().unwrap_or_default(),
            ])
            .map_err(erreur("ligne CSV"))?;
    }

    let octets = writer
        .into_inner()
        .map_err(|error| AppError::Serialization(format!("clôture du CSV : {error}")))?;
    String::from_utf8(octets)
        .map_err(|error| AppError::Serialization(format!("encodage du CSV : {error}")))
}

#[cfg(test)]
#[path = "tests/export/mod.rs"]
mod tests;
