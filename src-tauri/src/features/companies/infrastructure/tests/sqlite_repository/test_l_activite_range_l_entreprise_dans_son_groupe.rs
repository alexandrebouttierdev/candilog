//! Activité de la relation (écran Relations) : décomptes et groupes En cours / Repérées /
//! Clôturées.

use super::*;
use crate::core::database::helpers::connection;
use crate::features::companies::domain::RelationState;

/// Insère une candidature brute : le dépôt des entreprises ne connaît pas celui des
/// candidatures, et le déclencheur attribue la référence.
fn candidature(
    repo: &SqliteCompanyRepository,
    company: uuid::Uuid,
    id: &str,
    status: &str,
    sent: &str,
) {
    connection(&repo.pool)
        .unwrap()
        .execute(
            "INSERT INTO applications (id, company_id, job_title, contract_type_code, status,
                sent_date, job_url, created_at, updated_at)
             VALUES (?1, ?2, 'Poste', 'CDI', ?3, ?4, 'https://exemple.fr', ?4, ?4)",
            rusqlite::params![id, company.to_string(), status, sent],
        )
        .unwrap();
}

fn noms(repo: &SqliteCompanyRepository, state: RelationState) -> Vec<String> {
    repo.list_page(
        1,
        20,
        &CompanyFilter {
            relation_state: Some(state),
            ..CompanyFilter::default()
        },
    )
    .unwrap()
    .items
    .into_iter()
    .map(|company| company.name)
    .collect()
}

#[test]
fn l_activite_compte_les_candidatures_ouvertes_et_les_contacts() {
    let repo = repo();
    let nova = repo.create(&entree("Nova")).unwrap();
    candidature(&repo, nova.id, "c-1", "EN_ATTENTE", "2026-08-01");
    candidature(&repo, nova.id, "c-2", "REFUS", "2026-09-01");
    connection(&repo.pool)
        .unwrap()
        .execute(
            "INSERT INTO contacts (id, company_id, first_name, name, created_at, updated_at)
             VALUES ('k-1', ?1, 'Claire', 'Ménard', '2026-01-01', '2026-01-01')",
            [nova.id.to_string()],
        )
        .unwrap();

    let activite = repo.get(nova.id).unwrap().activity;
    assert_eq!(activite.open_applications, 1);
    assert_eq!(activite.applications, 2);
    assert_eq!(activite.contacts, 1);
    // La plus récente est la seconde, envoyée en septembre : référence 2.
    assert_eq!(activite.last_reference_number, Some(2));
    assert_eq!(activite.last_sent_date.as_deref(), Some("2026-09-01"));
}

#[test]
fn les_groupes_separent_en_cours_reperees_et_cloturees() {
    let repo = repo();
    let active = repo.create(&entree("Active")).unwrap();
    candidature(&repo, active.id, "c-1", "ENTRETIEN", "2026-08-01");
    let close = repo.create(&entree("Close")).unwrap();
    candidature(&repo, close.id, "c-2", "REFUS", "2026-08-01");
    repo.create(&entree("Reperee")).unwrap();

    assert_eq!(noms(&repo, RelationState::Active), vec!["Active"]);
    assert_eq!(noms(&repo, RelationState::Watch), vec!["Reperee"]);
    assert_eq!(noms(&repo, RelationState::Closed), vec!["Close"]);
}
