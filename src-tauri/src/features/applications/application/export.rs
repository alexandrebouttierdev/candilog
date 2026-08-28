//! Export CSV des candidatures.

use crate::core::errors::{AppError, AppResult};
use crate::features::applications::domain::Application;

/// Sérialise des candidatures en CSV, séparateur point-virgule.
///
/// Le point-virgule et non la virgule : c'est ce qu'attend Excel en locale française, où un
/// fichier séparé par des virgules s'ouvre en une seule colonne. L'export vise d'abord une
/// relecture par l'utilisateur, pas un import machine.
///
/// # Errors
/// Retourne `AppError::Serialization` si l'écriture échoue.
pub fn vers_csv(applications: &[Application]) -> AppResult<String> {
    let mut writer = csv::WriterBuilder::new()
        .delimiter(b';')
        .from_writer(Vec::new());

    fn error(context: &'static str) -> impl Fn(csv::Error) -> AppError {
        move |error| AppError::Serialization(format!("{context} : {error}"))
    }

    writer
        .write_record([
            "poste",
            "entreprise",
            "ville",
            "contrat",
            "statut",
            "sent_date",
            "job_url",
            "notes",
        ])
        .map_err(error("en-tête CSV"))?;

    for row in applications {
        writer
            .write_record([
                row.job_title.as_str(),
                row.company_name.as_deref().unwrap_or_default(),
                row.company_city.as_deref().unwrap_or_default(),
                &row.contract_type.to_string(),
                &row.status.to_string(),
                row.sent_date.as_str(),
                row.job_url.as_deref().unwrap_or_default(),
                row.notes.as_deref().unwrap_or_default(),
            ])
            .map_err(error("ligne CSV"))?;
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
