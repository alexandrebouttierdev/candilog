//! Export CSV des entreprises (écran Relations, `reference_design/DECISIONS.md` E10).

use crate::core::errors::{AppError, AppResult};
use crate::core::utils::csv_export::{avec_bom, champ_sur};
use crate::features::companies::domain::Company;

/// État de la relation, selon les mêmes règles que les groupes de l'écran Relations.
fn relation_state(company: &Company) -> &'static str {
    if company.activity.open_applications > 0 {
        "En cours"
    } else if company.activity.applications == 0 {
        "Repérée"
    } else {
        "Clôturée"
    }
}

/// Sérialise les entreprises en CSV : 9 colonnes, séparateur point-virgule, marque d'ordre
/// d'octets, champs pouvant passer pour des formules neutralisés.
///
/// # Errors
/// Retourne `AppError::Serialization` si l'écriture échoue.
pub fn companies_csv(companies: &[Company]) -> AppResult<String> {
    let error = |context: &'static str| {
        move |error: csv::Error| AppError::Serialization(format!("{context} : {error}"))
    };
    let mut writer = csv::WriterBuilder::new()
        .delimiter(b';')
        .from_writer(Vec::new());
    writer
        .write_record([
            "nom",
            "secteur",
            "taille",
            "ville",
            "site",
            "statut_relation",
            "nb_candidatures",
            "derniere_interaction",
            "notes",
        ])
        .map_err(error("en-tête CSV"))?;
    for company in companies {
        writer
            .write_record(
                [
                    company.name.as_str(),
                    company.sector_name.as_deref().unwrap_or_default(),
                    &company.company_size.to_string(),
                    company.city.as_deref().unwrap_or_default(),
                    company.website.as_deref().unwrap_or_default(),
                    relation_state(company),
                    &company.activity.applications.to_string(),
                    company
                        .activity
                        .last_sent_date
                        .as_deref()
                        .unwrap_or_default(),
                    company.notes.as_deref().unwrap_or_default(),
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

/// Nom du fichier des contacts, posé à côté de celui des entreprises : `entreprises-…`
/// devient `contacts-…`, tout autre nom reçoit le suffixe `-contacts`.
#[must_use]
pub fn contacts_file_name(companies_file: &str) -> String {
    let stem = companies_file
        .strip_suffix(".csv")
        .or_else(|| companies_file.strip_suffix(".CSV"))
        .unwrap_or(companies_file);
    match stem.strip_prefix("entreprises") {
        Some(rest) => format!("contacts{rest}.csv"),
        None => format!("{stem}-contacts.csv"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::companies::domain::{CompanyActivity, CompanySize};

    fn company(name: &str, open: u32, total: u32) -> Company {
        Company {
            id: uuid::Uuid::nil(),
            name: name.into(),
            sector_id: None,
            sector_name: Some("Logiciel".into()),
            company_type_id: None,
            company_type_name: None,
            company_size: CompanySize::Pme,
            website: Some("https://linaia.fr".into()),
            city: Some("Rennes".into()),
            address: None,
            notes: Some("=HYPERLINK(\"x\")".into()),
            created_at: String::new(),
            updated_at: String::new(),
            activity: CompanyActivity {
                open_applications: open,
                applications: total,
                contacts: 0,
                last_reference_number: None,
                last_sent_date: (total > 0).then(|| "2026-09-01".to_owned()),
            },
        }
    }

    #[test]
    fn exporte_les_neuf_colonnes_et_l_etat_de_la_relation() {
        let csv = companies_csv(&[
            company("Linaïa", 1, 2),
            company("Sémaphore", 0, 0),
            company("Bréhat", 0, 1),
        ])
        .unwrap();
        let lignes: Vec<&str> = csv.trim_start_matches('\u{feff}').lines().collect();

        assert_eq!(lignes[0].split(';').count(), 9);
        assert!(lignes[1]
            .starts_with("Linaïa;Logiciel;PME;Rennes;https://linaia.fr;En cours;2;2026-09-01;"));
        assert!(lignes[2].contains(";Repérée;0;;"));
        assert!(lignes[3].contains(";Clôturée;1;"));
    }

    #[test]
    fn le_fichier_des_contacts_suit_celui_des_entreprises() {
        assert_eq!(
            contacts_file_name("entreprises-2026-09-27.csv"),
            "contacts-2026-09-27.csv"
        );
        assert_eq!(
            contacts_file_name("relations.csv"),
            "relations-contacts.csv"
        );
    }

    #[test]
    fn neutralise_une_note_qui_ouvrirait_une_formule() {
        let csv = companies_csv(&[company("Linaïa", 1, 1)]).unwrap();

        assert!(csv.contains("'=HYPERLINK"));
    }
}
