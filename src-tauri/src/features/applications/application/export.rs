//! Export CSV des candidatures.

use crate::core::errors::{AppError, AppResult};
use crate::core::utils::csv_export::{avec_bom, champ_sur};
use crate::features::applications::domain::Application;

/// Sérialise des candidatures en CSV, séparateur point-virgule.
///
/// Le point-virgule et non la virgule : c'est ce qu'attend Excel en locale française, où un
/// fichier séparé par des virgules s'ouvre en une seule colonne. L'export vise d'abord une
/// relecture par l'utilisateur, pas un import machine. Pour la même raison le fichier
/// commence par une marque d'ordre d'octets, et les champs pouvant passer pour des formules
/// sont neutralisés (`core::utils::csv_export`).
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
            "type_candidature",
            "contrat",
            "duree_hebdomadaire",
            "heures_par_semaine",
            "domaine_professionnel",
            "type_entreprise",
            "taille_entreprise",
            "ville",
            "adresse",
            "statut",
            "date_envoi",
            "lien_offre",
            "notes",
        ])
        .map_err(error("en-tête CSV"))?;

    for row in applications {
        // Ville, adresse et type d'entreprise sont exportés dans leur valeur **effective** :
        // le CSV est relu hors de l'application, où la règle d'héritage n'existe plus.
        writer
            .write_record(
                [
                    row.job_title.as_str(),
                    row.company_name.as_deref().unwrap_or_default(),
                    &row.application_type.to_string(),
                    row.contract_type_name
                        .as_deref()
                        .unwrap_or(row.contract_type_code.as_str()),
                    &row.weekly_work_schedule.to_string(),
                    &row.weekly_hours.map(|h| h.to_string()).unwrap_or_default(),
                    row.professional_domain_name.as_deref().unwrap_or_default(),
                    row.effective_company_type_name
                        .as_deref()
                        .unwrap_or_default(),
                    &row.company_size.to_string(),
                    row.effective_city.as_deref().unwrap_or_default(),
                    row.effective_address.as_deref().unwrap_or_default(),
                    &row.status.to_string(),
                    row.sent_date.as_str(),
                    row.job_url.as_deref().unwrap_or_default(),
                    row.notes.as_deref().unwrap_or_default(),
                ]
                .map(champ_sur),
            )
            .map_err(error("ligne CSV"))?;
    }

    let octets = writer
        .into_inner()
        .map_err(|error| AppError::Serialization(format!("clôture du CSV : {error}")))?;
    let texte = String::from_utf8(octets)
        .map_err(|error| AppError::Serialization(format!("encodage du CSV : {error}")))?;
    Ok(avec_bom(&texte))
}

#[cfg(test)]
#[path = "tests/export/mod.rs"]
mod tests;
