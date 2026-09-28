//! Export CSV des contacts (écran Relations, `reference_design/DECISIONS.md` E10).

use crate::core::errors::{AppError, AppResult};
use crate::core::utils::csv_export::{avec_bom, champ_sur};
use crate::features::contacts::domain::Contact;

/// Sérialise les contacts en CSV : 9 colonnes, séparateur point-virgule, marque d'ordre
/// d'octets. Le type suit l'écran Relations : un contact rattaché à une candidature est un
/// recruteur (ou un manager), les autres relèvent du réseau.
///
/// # Errors
/// Retourne `AppError::Serialization` si l'écriture échoue.
pub fn contacts_csv(contacts: &[Contact]) -> AppResult<String> {
    let error = |context: &'static str| {
        move |error: csv::Error| AppError::Serialization(format!("{context} : {error}"))
    };
    let mut writer = csv::WriterBuilder::new()
        .delimiter(b';')
        .from_writer(Vec::new());
    writer
        .write_record([
            "prenom",
            "nom",
            "role",
            "entreprise",
            "email",
            "telephone",
            "type",
            "derniere_interaction",
            "notes",
        ])
        .map_err(error("en-tête CSV"))?;
    for contact in contacts {
        let role = [
            contact.job_title.as_deref(),
            contact.tracking_role.as_deref(),
        ]
        .into_iter()
        .flatten()
        .filter(|value| !value.trim().is_empty())
        .collect::<Vec<_>>()
        .join(" · ");
        writer
            .write_record(
                [
                    contact.first_name.as_str(),
                    contact.name.as_str(),
                    role.as_str(),
                    contact.company_name.as_deref().unwrap_or_default(),
                    contact.email.as_deref().unwrap_or_default(),
                    contact.phone.as_deref().unwrap_or_default(),
                    if contact.activity.applications > 0 {
                        "Recruteur"
                    } else {
                        "Réseau"
                    },
                    contact
                        .activity
                        .last_sent_date
                        .as_deref()
                        .unwrap_or_default(),
                    contact.notes.as_deref().unwrap_or_default(),
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
mod tests {
    use super::*;
    use crate::features::contacts::domain::ContactActivity;

    fn contact(applications: u32) -> Contact {
        Contact {
            id: uuid::Uuid::nil(),
            company_id: None,
            company_name: Some("Vallis Conseil".into()),
            first_name: "Claire".into(),
            name: "Ménard".into(),
            job_title: Some("Chargée de recrutement".into()),
            tracking_role: Some("recruteur".into()),
            email: Some("claire@vallis.fr".into()),
            phone: Some("+33 6 12 34 56 78".into()),
            linkedin: None,
            notes: None,
            created_at: String::new(),
            updated_at: String::new(),
            activity: ContactActivity {
                applications,
                last_reference_number: None,
                last_sent_date: (applications > 0).then(|| "2026-08-29".to_owned()),
            },
        }
    }

    #[test]
    fn exporte_les_neuf_colonnes_et_distingue_recruteur_et_reseau() {
        let csv = contacts_csv(&[contact(1), contact(0)]).unwrap();
        let lignes: Vec<&str> = csv.trim_start_matches('\u{feff}').lines().collect();

        assert_eq!(lignes[0].split(';').count(), 9);
        // Le téléphone commençant par « + » est neutralisé pour le tableur.
        assert_eq!(
            lignes[1],
            "Claire;Ménard;Chargée de recrutement · recruteur;Vallis Conseil;claire@vallis.fr;'+33 6 12 34 56 78;Recruteur;2026-08-29;"
        );
        assert!(lignes[2].contains(";Réseau;;"));
    }
}
