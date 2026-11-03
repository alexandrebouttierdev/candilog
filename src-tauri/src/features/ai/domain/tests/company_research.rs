//! Tests de la recherche d'entreprise : correspondances, normalisation, choix et recadrage.

use super::*;

fn candidat(siret: &str, name: &str, city: &str, head_office: bool) -> CompanyCandidate {
    CompanyCandidate {
        siret: siret.into(),
        siren: siret[..9].into(),
        name: name.into(),
        trade_name: None,
        city: Some(city.into()),
        postal_code: None,
        address: None,
        head_office,
        activity_code: None,
        activity_label: None,
        legal_category: None,
        size_category: None,
        workforce_range: None,
        open_establishments: None,
        source_url: format!("https://annuaire-entreprises.data.gouv.fr/etablissement/{siret}"),
    }
}

#[test]
fn la_taille_suit_la_categorie_insee_puis_la_tranche_d_effectif() {
    assert_eq!(
        size_from_registry(Some("GE"), None, None),
        Some(CompanySize::Large)
    );
    assert_eq!(
        size_from_registry(Some("ETI"), Some("01"), None),
        Some(CompanySize::Eti)
    );
    assert_eq!(
        size_from_registry(Some("PME"), Some("21"), None),
        Some(CompanySize::Pme)
    );
    assert_eq!(
        size_from_registry(Some("PME"), Some("02"), None),
        Some(CompanySize::Tpe)
    );
    assert_eq!(
        size_from_registry(Some("PME"), Some("NN"), Some("1000")),
        Some(CompanySize::Micro)
    );
    assert_eq!(
        size_from_registry(None, Some("52"), None),
        Some(CompanySize::Large)
    );
    assert_eq!(size_from_registry(None, None, None), None);
    assert_eq!(size_from_registry(None, Some("??"), None), None);
}

#[test]
fn la_nature_juridique_prime_sur_l_activite() {
    // Une association de santé reste une association.
    assert_eq!(
        company_type_from_registry(Some("9220"), Some("86.10Z")),
        Some("ASSOCIATION")
    );
    assert_eq!(
        company_type_from_registry(Some("7210"), None),
        Some("LOCAL_AUTHORITY")
    );
    assert_eq!(
        company_type_from_registry(Some("9300"), None),
        Some("FOUNDATION")
    );
    assert_eq!(
        company_type_from_registry(Some("5710"), Some("78.20Z")),
        Some("TEMP_AGENCY")
    );
    assert_eq!(
        company_type_from_registry(Some("5710"), Some("64.19Z")),
        Some("BANK")
    );
}

#[test]
fn une_societe_de_conseil_informatique_reste_au_jugement_du_modele() {
    // ESN, cabinet de conseil ou éditeur : le code NAF seul ne tranche pas.
    assert_eq!(
        company_type_from_registry(Some("5710"), Some("62.02A")),
        None
    );
    assert_eq!(company_type_from_registry(None, None), None);
}

#[test]
fn le_secteur_ne_se_deduit_que_des_sections_sans_ambiguite() {
    assert_eq!(
        sector_from_activity("62.02A").as_deref(),
        Some("5ec70000-0000-4000-8000-00000000000d")
    );
    assert_eq!(
        sector_from_activity("93.12Z").as_deref(),
        Some("5ec70000-0000-4000-8000-000000000015")
    );
    // Section M : conseil, juridique, publicité… trop varié.
    assert_eq!(sector_from_activity("70.22Z"), None);
    assert_eq!(sector_from_activity("xx"), None);
    assert_eq!(naf_section("62.02A"), Some('J'));
    assert_eq!(naf_section_label('J'), Some("Information et communication"));
}

#[test]
fn les_capitales_de_l_annuaire_deviennent_lisibles() {
    assert_eq!(
        readable_case("78 RUE OLIVIER DE SERRES 75015 PARIS"),
        "78 Rue Olivier de Serres 75015 Paris"
    );
    assert_eq!(readable_case("SAINT-MALO"), "Saint-Malo");
    assert_eq!(readable_case("PLACE D'ARMES"), "Place d'Armes");
    assert_eq!(readable_case("L'HAY-LES-ROSES"), "L'Hay-les-Roses");
    // Une casse déjà mixte est respectée.
    assert_eq!(readable_case("Orange Business"), "Orange Business");
}

#[test]
fn seuls_les_sites_d_entreprise_sont_retenus_comme_site_officiel() {
    assert_eq!(
        website_origin("https://www.orange.fr/portail?x=1").as_deref(),
        Some("https://www.orange.fr")
    );
    assert_eq!(website_origin("javascript:alert(1)"), None);
    assert_eq!(website_origin("https://intranet/"), None);
    assert!(is_directory_site(
        "https://www.societe.com/societe/orange-380129866.html"
    ));
    assert!(is_directory_site("https://fr.linkedin.com/company/orange"));
    assert!(!is_directory_site("https://www.orange.fr/"));
}

#[test]
fn plusieurs_etablissements_imposent_un_choix() {
    let resultat = rank_candidates(
        "Orange",
        None,
        vec![
            candidat("38012986600001", "ORANGE", "Paris", true),
            candidat("38012986600002", "ORANGE", "Rennes", false),
        ],
    );
    assert!(!resultat.confident);
    assert_eq!(resultat.candidates.len(), 2);
    assert!(resultat.candidates[0].head_office, "le siège passe en tête");
}

#[test]
fn la_ville_demandee_restreint_les_etablissements() {
    let resultat = rank_candidates(
        "orange",
        Some("rennes"),
        vec![
            candidat("38012986600001", "ORANGE", "Paris", true),
            candidat("38012986600002", "ORANGE", "Rennes", false),
        ],
    );
    assert_eq!(resultat.candidates.len(), 1);
    assert_eq!(resultat.candidates[0].city.as_deref(), Some("Rennes"));
    assert!(resultat.confident, "un seul établissement au nom identique");
}

#[test]
fn un_homonyme_approchant_n_est_jamais_juge_sur() {
    let resultat = rank_candidates(
        "Orange",
        None,
        vec![candidat(
            "12345678900011",
            "ORANGE BLEUE FITNESS",
            "Lyon",
            true,
        )],
    );
    assert!(!resultat.confident);
}

#[test]
fn les_doublons_et_l_exces_sont_ecartes() {
    let mut liste = vec![candidat("38012986600001", "ORANGE", "Paris", true); 2];
    for index in 0..30 {
        liste.push(candidat(
            &format!("380129866{index:05}"),
            "ORANGE",
            "Lyon",
            false,
        ));
    }
    let resultat = rank_candidates("orange", None, liste);
    assert_eq!(resultat.candidates.len(), MAX_CANDIDATES);
    assert!(resultat.truncated);
}

#[test]
fn le_recadrage_ecarte_tout_code_hors_des_listes() {
    let secteurs = vec!["id-informatique".to_owned(), "id-conseil".to_owned()];
    let types = vec!["IT_SERVICES_COMPANY".to_owned()];
    let urls = vec!["https://www.nova.fr".to_owned()];
    let retenu = ground_classification(
        &CompanyClassification {
            secteur: Some("s02".into()),
            company_type: Some("it_services_company".into()),
            site: Some("U1".into()),
        },
        &secteurs,
        &types,
        &urls,
    );
    assert_eq!(retenu.sector_id.as_deref(), Some("id-conseil"));
    assert_eq!(retenu.company_type.as_deref(), Some("IT_SERVICES_COMPANY"));
    assert_eq!(retenu.website.as_deref(), Some("https://www.nova.fr"));

    let invente = ground_classification(
        &CompanyClassification {
            secteur: Some("S09".into()),
            company_type: Some("LICORNE".into()),
            site: Some("https://site-invente.fr".into()),
        },
        &secteurs,
        &types,
        &urls,
    );
    assert_eq!(invente, GroundedClassification::default());
}

#[test]
fn les_entrees_sont_bornees() {
    assert!(validate_company_query("   ").is_err());
    assert_eq!(
        validate_company_query("  Orange   Business ").unwrap(),
        "Orange Business"
    );
    assert!(validate_company_query(&"x".repeat(MAX_COMPANY_QUERY_CHARS + 1)).is_err());
    assert_eq!(validate_location(Some("  ")).unwrap(), None);
    assert!(validate_location(Some(&"x".repeat(MAX_LOCATION_CHARS + 1))).is_err());
    assert!(validate_siret("38012986600001").is_ok());
    assert!(validate_siret("380129866").is_err());
    assert!(validate_siret("3801298660000a").is_err());
}
