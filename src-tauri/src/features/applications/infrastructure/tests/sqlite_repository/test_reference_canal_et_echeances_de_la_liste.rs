//! Référence `CAN-xxx`, canal, échéances à venir, impact de suppression, historique.

use super::*;

fn aujourd_hui_plus(jours: i64) -> String {
    (chrono::Local::now().date_naive() + chrono::Duration::days(jours))
        .format("%Y-%m-%d")
        .to_string()
}

#[test]
fn chaque_candidature_recoit_une_reference_croissante() {
    let (repo, company_id) = context();
    let premiere = repo
        .create(&entree(company_id, "Premier", "2026-08-20"))
        .unwrap();
    let seconde = repo
        .create(&entree(company_id, "Second", "2026-08-21"))
        .unwrap();
    assert_eq!(premiere.reference_number, 1);
    assert_eq!(seconde.reference_number, 2);
}

#[test]
fn une_reference_supprimee_n_est_jamais_reattribuee() {
    let (repo, company_id) = context();
    repo.create(&entree(company_id, "Premier", "2026-08-20"))
        .unwrap();
    let derniere = repo
        .create(&entree(company_id, "Second", "2026-08-21"))
        .unwrap();
    repo.delete(derniere.id).unwrap();

    let suivante = repo
        .create(&entree(company_id, "Troisième", "2026-08-22"))
        .unwrap();
    // « CAN-002 » a désigné une candidature supprimée : il ne peut pas en désigner une autre.
    assert_eq!(suivante.reference_number, 3);
}

#[test]
fn apres_une_remise_a_zero_la_numerotation_repart_de_un() {
    let (repo, company_id) = context();
    repo.create(&entree(company_id, "Ancienne", "2026-08-20"))
        .unwrap();
    // `reset_data` vide aussi `app_kv`, compteur compris.
    connection(&repo.pool)
        .unwrap()
        .execute_batch("DELETE FROM applications; DELETE FROM app_kv;")
        .unwrap();

    let nouvelle = repo
        .create(&entree(company_id, "Nouvelle", "2026-08-21"))
        .unwrap();
    assert_eq!(nouvelle.reference_number, 1);
}

#[test]
fn le_canal_est_persiste_et_fixe_la_nature_de_la_demarche() {
    let (repo, company_id) = context();
    let mut reseau = entree(company_id, "Coopté", "2026-08-20");
    reseau.channel = ApplicationChannel::Network;
    reseau.job_url = None;
    let lue = repo.create(&reseau).unwrap();
    assert_eq!(lue.channel, ApplicationChannel::Network);
    assert_eq!(lue.application_type, ApplicationType::JobOffer);

    let mut spontanee = entree(company_id, "Spontanée", "2026-08-20");
    spontanee.channel = ApplicationChannel::Spontaneous;
    let lue = repo.create(&spontanee).unwrap();
    assert_eq!(lue.application_type, ApplicationType::Unsolicited);
    // Le dépôt n'écrit jamais de lien pour une démarche spontanée.
    assert_eq!(lue.job_url, None);
}

#[test]
fn la_liste_expose_la_prochaine_relance_et_le_prochain_entretien_a_venir() {
    let (repo, company_id) = context();
    let candidature = repo
        .create(&entree(company_id, "Ops", "2026-08-20"))
        .unwrap();
    let conn = connection(&repo.pool).unwrap();
    let passee = aujourd_hui_plus(-3);
    let proche = aujourd_hui_plus(2);
    let lointaine = aujourd_hui_plus(9);
    for (id, date) in [
        ("r-passee", &passee),
        ("r-proche", &proche),
        ("r-loin", &lointaine),
    ] {
        conn.execute(
            "INSERT INTO follow_ups (id, application_id, follow_up_date, created_at)
             VALUES (?1, ?2, ?3, '2026-01-01')",
            rusqlite::params![id, candidature.id.to_string(), date],
        )
        .unwrap();
    }
    let entretien = format!("{}T14:30:00", aujourd_hui_plus(0));
    conn.execute(
        "INSERT INTO interviews (id, application_id, interview_date, created_at, updated_at)
         VALUES ('e-1', ?1, ?2, '2026-01-01', '2026-01-01')",
        rusqlite::params![candidature.id.to_string(), entretien],
    )
    .unwrap();

    let lue = repo.get(candidature.id).unwrap();
    // La relance passée est ignorée : l'échéance affichée est la prochaine, pas la plus
    // ancienne. L'entretien du jour compte, même si son heure est déjà dépassée.
    assert_eq!(lue.next_follow_up_date.as_deref(), Some(proche.as_str()));
    assert_eq!(lue.next_interview_at.as_deref(), Some(entretien.as_str()));
}

#[test]
fn l_impact_de_suppression_compte_ce_qui_part_en_cascade() {
    let (repo, company_id) = context();
    let candidature = repo
        .create(&entree(company_id, "Ops", "2026-08-20"))
        .unwrap();
    repo.update_status(candidature.id, ApplicationStatus::FollowedUp)
        .unwrap();
    connection(&repo.pool)
        .unwrap()
        .execute(
            "INSERT INTO follow_ups (id, application_id, follow_up_date, created_at)
             VALUES ('r-1', ?1, '2026-09-01', '2026-01-01')",
            [candidature.id.to_string()],
        )
        .unwrap();

    let impact = repo.deletion_impact(candidature.id).unwrap();
    assert_eq!(impact.follow_ups, 1);
    assert_eq!(impact.interviews, 0);
    assert_eq!(impact.status_changes, 2);
    assert!(matches!(
        repo.deletion_impact(Uuid::new_v4()),
        Err(AppError::NotFound(_))
    ));
}

#[test]
fn l_historique_des_statuts_commence_par_le_plus_recent() {
    let (repo, company_id) = context();
    let candidature = repo
        .create(&entree(company_id, "Ops", "2026-08-20"))
        .unwrap();
    repo.update_status(candidature.id, ApplicationStatus::Interview)
        .unwrap();

    let etapes: Vec<_> = repo
        .status_history(candidature.id)
        .unwrap()
        .into_iter()
        .map(|etape| etape.status)
        .collect();
    assert_eq!(
        etapes,
        vec![ApplicationStatus::Interview, ApplicationStatus::Pending]
    );
    assert!(matches!(
        repo.status_history(Uuid::new_v4()),
        Err(AppError::NotFound(_))
    ));
}

#[test]
fn le_filtre_par_canal_ne_retient_que_les_canaux_coches() {
    let (repo, company_id) = context();
    let mut reseau = entree(company_id, "Coopté", "2026-08-20");
    reseau.channel = ApplicationChannel::Network;
    repo.create(&reseau).unwrap();
    repo.create(&entree(company_id, "Annonce", "2026-08-20"))
        .unwrap();

    let page = repo
        .list_page(
            1,
            20,
            &ApplicationFilter {
                channel: vec![ApplicationChannel::Network],
                ..ApplicationFilter::default()
            },
        )
        .unwrap();
    assert_eq!(page.total, 1);
    assert_eq!(page.items[0].job_title, "Coopté");
}
