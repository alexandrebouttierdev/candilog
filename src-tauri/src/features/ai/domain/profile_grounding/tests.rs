//! Tests du recadrage d'un profil importé.
use super::*;
use crate::features::profile::domain::{
    Certification, Education, Experience, Identity, Language, Project, Skill,
};

/// Extrait du CV réellement analysé, tel que le lecteur PDF le restitue.
const CV: &str = "Alexandre Bouttier\n\n\
Technicien Supérieur Systèmes et Réseaux (TSSR) · Recherche contrat de professionnalisation\n\n\
Admis en formation TSSR à l'ENI de Chartres-de-Bretagne\n\n\
07 86 66 46 99 34 ans alexandrebouttier@gmail.com Saint-Jacques-de-la-Lande (35)\n\n\
linkedin.com/in/alexandrebouttier ↗ alexandrebouttier.fr ↗\n\n\
Projet professionnel · Technicien Supérieur Systèmes et Réseaux : admis en formation TSSR\n\
à l'ENI ; autoformation et mise en pratique sur un serveur VPS\n\n\
Oct. 2025 – Sept. 2026\n\n\
Projet personnel · entretienmx.fr ↗· OVH · Ubuntu Server\n\n\
Français · langue maternelle\n\n\
Anglais · lecture courante de documentation technique\n";

/// Les fragments observés dans la réponse réelle de `maternion/lfm2.5:350m`.
#[test]
fn efface_les_fragments_de_domaine_inventes_par_le_modele() {
    let mut profile = Profile {
        languages: vec![Language {
            name: ".com".into(),
            level: ".com".into(),
        }],
        projects: vec![Project {
            name: "Projet personnel · entretienmx.fr".into(),
            url: Some(".fr".into()),
            ..Project::default()
        }],
        certifications: vec![Certification {
            name: ".org".into(),
            url: Some(".com".into()),
            ..Certification::default()
        }],
        ..Profile::default()
    };

    ground_imported_profile(CV, &mut profile);

    assert_eq!(profile.languages[0].name, "");
    assert_eq!(profile.languages[0].level, "");
    assert_eq!(
        profile.projects[0].name,
        "Projet personnel · entretienmx.fr"
    );
    assert_eq!(profile.projects[0].url, None);
    assert_eq!(profile.certifications[0].name, "");
    assert_eq!(profile.certifications[0].url, None);
}

#[test]
fn efface_les_morceaux_de_mots_et_la_certification_absente_du_cv() {
    let mut profile = Profile {
        experiences: vec![Experience {
            title: "Projet professionnel".into(),
            company: "ENI".into(),
            location: Some(".chartres-de-bretagne".into()),
            ..Experience::default()
        }],
        languages: vec![
            Language {
                name: ".franc".into(),
                level: "Bac".into(),
            },
            Language {
                name: ".anglais".into(),
                level: "Anglais".into(),
            },
        ],
        projects: vec![Project {
            name: "Projet personnel".into(),
            url: Some(".github".into()),
            ..Project::default()
        }],
        certifications: vec![Certification {
            name: ".enf".into(),
            issuer: Some(".chartres-de-bretagne".into()),
            url: Some(".".into()),
            ..Certification::default()
        }],
        ..Profile::default()
    };

    ground_imported_profile(CV, &mut profile);

    // L'expérience reste, seul le lieu inventé disparaît.
    assert_eq!(profile.experiences[0].title, "Projet professionnel");
    assert_eq!(profile.experiences[0].company, "ENI");
    assert_eq!(profile.experiences[0].location, None);
    assert_eq!(profile.languages[0].name, "");
    assert_eq!(profile.languages[1].name, "");
    assert_eq!(profile.projects[0].url, None);
    assert_eq!(profile.certifications[0].name, "");
    assert_eq!(profile.certifications[0].issuer, None);
    assert_eq!(profile.certifications[0].url, None);
}

#[test]
fn conserve_les_valeurs_recopiees_du_cv() {
    let mut profile = Profile {
        identity: Identity {
            first_name: "Alexandre".into(),
            name: "Bouttier".into(),
            email: "alexandrebouttier@gmail.com".into(),
            phone: Some("07 86 66 46 99".into()),
            city: Some("Chartres-de-Bretagne".into()),
            title: Some("Technicien Supérieur Systèmes et Réseaux".into()),
            resume: Some("Recherche contrat de professionnalisation".into()),
            linkedin: Some("linkedin.com/in/alexandrebouttier".into()),
            website: Some("alexandrebouttier.fr".into()),
            ..Identity::default()
        },
        skills: vec![Skill {
            name: "Ubuntu Server".into(),
            description: None,
        }],
        languages: vec![Language {
            name: "Français".into(),
            level: "langue maternelle".into(),
        }],
        ..Profile::default()
    };
    let attendu = profile.clone();

    ground_imported_profile(CV, &mut profile);

    assert_eq!(profile, attendu);
}

/// La comparaison ignore la casse et les accents, comme partout ailleurs.
#[test]
fn accepte_une_recopie_a_la_casse_ou_aux_accents_pres() {
    let mut profile = Profile {
        languages: vec![Language {
            name: "FRANCAIS".into(),
            level: "Langue Maternelle".into(),
        }],
        ..Profile::default()
    };

    ground_imported_profile(CV, &mut profile);

    assert_eq!(profile.languages[0].name, "FRANCAIS");
    assert_eq!(profile.languages[0].level, "Langue Maternelle");
}

/// Une date déjà reformatée ne figure pas telle quelle dans le CV : la recopier serait
/// impossible, et le recadrage ne doit pas y toucher.
#[test]
fn ne_touche_pas_aux_dates_deja_normalisees() {
    let mut profile = Profile {
        experiences: vec![Experience {
            title: "Projet professionnel".into(),
            company: "ENI".into(),
            start_date: "2025-10".into(),
            end_date: Some("2026-09".into()),
            ..Experience::default()
        }],
        ..Profile::default()
    };

    ground_imported_profile(CV, &mut profile);

    assert_eq!(profile.experiences[0].start_date, "2025-10");
    assert_eq!(profile.experiences[0].end_date.as_deref(), Some("2026-09"));
}

/// Une description multi-puces n'apparaît pas d'un bloc : on retient chaque fragment
/// recopié plutôt que d'effacer toute la mission.
#[test]
fn conserve_les_fragments_de_description_presents_dans_le_cv() {
    let source = "\
Senior Full-Stack Developer — AlthéaRH
Conception d'applications React et Node.js
Encadrement d'une équipe de 4 développeurs
Mise en place de pipelines CI/CD";
    let mut profile = Profile {
        experiences: vec![Experience {
            title: "Senior Full-Stack Developer".into(),
            company: "AlthéaRH".into(),
            description: Some(
                "Conception d'applications React et Node.js\n\
                 Encadrement d'une équipe de 4 développeurs\n\
                 Mission inventée absente du CV"
                    .into(),
            ),
            ..Experience::default()
        }],
        ..Profile::default()
    };

    ground_imported_profile(source, &mut profile);

    let description = profile.experiences[0].description.as_deref().unwrap();
    assert!(description.contains("Conception d'applications React et Node.js"));
    assert!(description.contains("Encadrement d'une équipe de 4 développeurs"));
    assert!(!description.contains("Mission inventée"));
}

#[test]
fn conserve_les_descriptions_en_mode_vision_malgre_un_texte_partiel() {
    let source = "Thomas Candilog\nAlthéaRH\nReact";
    let mut profile = Profile {
        experiences: vec![Experience {
            title: "Senior Full-Stack Developer".into(),
            company: "AlthéaRH".into(),
            description: Some("Conception d'applications métier pour les RH".into()),
            ..Experience::default()
        }],
        projects: vec![Project {
            name: "Candiscore".into(),
            description: Some("Application desktop de suivi de candidatures".into()),
            ..Project::default()
        }],
        ..Profile::default()
    };

    profile.identity.first_name = "Thomas".into();
    profile.identity.name = "Candilog".into();
    ground_imported_profile_keep_free_text(source, &mut profile);

    assert_eq!(
        profile.experiences[0].description.as_deref(),
        Some("Conception d'applications métier pour les RH")
    );
    assert_eq!(
        profile.projects[0].description.as_deref(),
        Some("Application desktop de suivi de candidatures")
    );
    // Le titre absent du texte partiel est bien effacé.
    assert_eq!(profile.experiences[0].title, "");
    // Prénom et nom lus sur l'image restent, même si le PDF complémentaire est pauvre.
    assert_eq!(profile.identity.first_name, "Thomas");
    assert_eq!(profile.identity.name, "Candilog");
}

#[test]
fn separe_une_description_collee_dans_le_libelle() {
    let mut profile = Profile {
        skills: vec![Skill {
            name: "Rust — async Tokio et axum".into(),
            description: None,
        }],
        projects: vec![Project {
            name: "Candilog : suivi de candidatures desktop".into(),
            description: None,
            url: None,
            technologies: None,
        }],
        ..Profile::default()
    };
    completer_descriptions_depuis_libelles(&mut profile);
    assert_eq!(profile.skills[0].name, "Rust");
    assert_eq!(
        profile.skills[0].description.as_deref(),
        Some("async Tokio et axum")
    );
    assert_eq!(profile.projects[0].name, "Candilog");
    assert_eq!(
        profile.projects[0].description.as_deref(),
        Some("suivi de candidatures desktop")
    );
}

#[test]
fn l_import_conserve_les_descriptions_de_competences_et_de_projets() {
    const CV: &str = "Camille Martin\nCompétences\nRust — async Tokio et axum\nProjets\nCandilog\nSuivi de candidatures desktop avec Tauri";
    let mut profile = Profile {
        skills: vec![Skill {
            name: "Rust — async Tokio et axum".into(),
            description: None,
        }],
        projects: vec![Project {
            name: "Candilog".into(),
            description: Some("Suivi de candidatures desktop avec Tauri".into()),
            url: None,
            technologies: None,
        }],
        ..Profile::default()
    };
    completer_descriptions_depuis_libelles(&mut profile);
    ground_imported_profile(CV, &mut profile);
    assert_eq!(profile.skills[0].name, "Rust");
    assert_eq!(
        profile.skills[0].description.as_deref(),
        Some("async Tokio et axum")
    );
    assert_eq!(
        profile.projects[0].description.as_deref(),
        Some("Suivi de candidatures desktop avec Tauri")
    );
}

#[test]
fn decoupe_un_nom_complet_colle_dans_un_seul_champ() {
    let mut dans_le_nom = Profile {
        identity: Identity {
            name: "Alexandre Bouttier".into(),
            ..Identity::default()
        },
        ..Profile::default()
    };
    completer_identite_noms("", &mut dans_le_nom);
    assert_eq!(dans_le_nom.identity.first_name, "Alexandre");
    assert_eq!(dans_le_nom.identity.name, "Bouttier");

    let mut dans_le_prenom = Profile {
        identity: Identity {
            first_name: "Camille Martin".into(),
            ..Identity::default()
        },
        ..Profile::default()
    };
    completer_identite_noms("", &mut dans_le_prenom);
    assert_eq!(dans_le_prenom.identity.first_name, "Camille");
    assert_eq!(dans_le_prenom.identity.name, "Martin");
}

#[test]
fn complete_prenom_et_nom_vides_depuis_le_texte_source() {
    let source = "Alexandre Bouttier\nTechnicien systèmes\nalexandre@example.fr";
    let mut profile = identite_vide();
    completer_identite_noms(source, &mut profile);
    assert_eq!(profile.identity.first_name, "Alexandre");
    assert_eq!(profile.identity.name, "Bouttier");
}

#[test]
fn ne_decoupe_pas_quand_prenom_et_nom_sont_deja_remplis() {
    let mut profile = Profile {
        identity: Identity {
            first_name: "Alexandre".into(),
            name: "Bouttier".into(),
            ..Identity::default()
        },
        ..Profile::default()
    };
    completer_identite_noms("Autre Personne\n...", &mut profile);
    assert_eq!(profile.identity.first_name, "Alexandre");
    assert_eq!(profile.identity.name, "Bouttier");
}

fn identite_vide() -> Profile {
    Profile::default()
}

#[test]
fn complete_email_et_telephone_vides_par_un_extrait_exact() {
    let source = "Camille Martin\n06 12 34 56 78\ncamille.martin@example.fr\nRennes";
    let mut profile = identite_vide();

    completer_contacts_vides(source, &mut profile);

    assert_eq!(profile.identity.email, "camille.martin@example.fr");
    assert_eq!(profile.identity.phone.as_deref(), Some("06 12 34 56 78"));
    assert!(source.contains(&profile.identity.email));
    assert!(source.contains(profile.identity.phone.as_deref().unwrap()));
}

#[test]
fn ne_recouvre_pas_un_email_ou_telephone_deja_recopie() {
    let source = "Nadia Leroy\nnadia.leroy@example.org\n+33 6 11 22 33 44\n06 99 88 77 66";
    let mut profile = Profile {
        identity: Identity {
            email: "nadia.leroy@example.org".into(),
            phone: Some("+33 6 11 22 33 44".into()),
            ..Identity::default()
        },
        ..Profile::default()
    };

    completer_contacts_vides(source, &mut profile);

    assert_eq!(profile.identity.email, "nadia.leroy@example.org");
    assert_eq!(profile.identity.phone.as_deref(), Some("+33 6 11 22 33 44"));
}

#[test]
fn n_invente_pas_un_contact_absent_du_texte() {
    let source = "Ines Bernard\nDeveloppeuse\n2019-2024\n75002 Paris";
    let mut profile = identite_vide();

    completer_contacts_vides(source, &mut profile);

    assert_eq!(profile.identity.email, "");
    assert_eq!(profile.identity.phone, None);
}

#[test]
fn prend_le_premier_email_et_le_premier_telephone_plausible() {
    let source = "Contact 01/02/2020 puis 06.12.34.56.78 puis 07-11-22-33-44\na@b et leo.martin+job@example.co.uk puis autre.personne@example.net";
    let mut profile = identite_vide();

    completer_contacts_vides(source, &mut profile);

    assert_eq!(profile.identity.email, "leo.martin+job@example.co.uk");
    assert_eq!(profile.identity.phone.as_deref(), Some("06.12.34.56.78"));
}

#[test]
fn accepte_le_format_international_francais_exact() {
    let source = "Portable +33 (0)6 12 34 56 78 et aussi 0033 7 00 11 22 33";
    let mut profile = identite_vide();

    completer_contacts_vides(source, &mut profile);

    assert_eq!(
        profile.identity.phone.as_deref(),
        Some("+33 (0)6 12 34 56 78")
    );
    assert!(source.contains(profile.identity.phone.as_deref().unwrap()));
}

#[test]
fn accepte_un_indicatif_international_hors_france() {
    let source = "Bureau +44 20 7946 0958\nhello.world@example.com";
    let mut profile = identite_vide();

    completer_contacts_vides(source, &mut profile);

    assert_eq!(profile.identity.phone.as_deref(), Some("+44 20 7946 0958"));
    assert_eq!(profile.identity.email, "hello.world@example.com");
}

#[test]
fn ignore_un_numero_colle_a_une_suite_de_chiffres_et_un_email_de_fichier() {
    let source = "Ref 061234567890123 logo@2x.png\n0033 6 12 34 56 78\nmaya.rossi@example.io";
    let mut profile = identite_vide();

    completer_contacts_vides(source, &mut profile);

    assert_eq!(
        profile.identity.phone.as_deref(),
        Some("0033 6 12 34 56 78")
    );
    assert_eq!(profile.identity.email, "maya.rossi@example.io");
}

#[test]
fn remplit_un_telephone_vide_sans_ecraser_l_email_et_inversement() {
    let source = "Sam Ortega sam.ortega@example.net 07 45 67 89 01";
    let mut email_seul = Profile {
        identity: Identity {
            email: "sam.ortega@example.net".into(),
            phone: None,
            ..Identity::default()
        },
        ..Profile::default()
    };
    completer_contacts_vides(source, &mut email_seul);
    assert_eq!(email_seul.identity.email, "sam.ortega@example.net");
    assert_eq!(email_seul.identity.phone.as_deref(), Some("07 45 67 89 01"));

    let mut tel_seul = Profile {
        identity: Identity {
            email: "   ".into(),
            phone: Some("07 45 67 89 01".into()),
            ..Identity::default()
        },
        ..Profile::default()
    };
    completer_contacts_vides(source, &mut tel_seul);
    assert_eq!(tel_seul.identity.email, "sam.ortega@example.net");
    assert_eq!(tel_seul.identity.phone.as_deref(), Some("07 45 67 89 01"));
}

#[test]
fn separe_un_email_colle_a_un_telephone() {
    let source = "Ref 12lea.martin@example.fr0612345678 et 06 12 34 56 78 90";
    let mut profile = identite_vide();

    completer_contacts_vides(source, &mut profile);

    assert_eq!(profile.identity.email, "12lea.martin@example.fr0612345678");
    assert_eq!(profile.identity.phone.as_deref(), Some("0612345678"));
    assert!(source.contains(profile.identity.email.as_str()));
    assert!(source.contains(profile.identity.phone.as_deref().unwrap()));
}

#[test]
fn accepte_un_telephone_entre_parentheses() {
    let source = "Ligne (07) 45 67 89 01 — ines.bernard@example.com";
    let mut profile = identite_vide();

    completer_contacts_vides(source, &mut profile);

    assert_eq!(profile.identity.phone.as_deref(), Some("(07) 45 67 89 01"));
    assert!(source.contains(profile.identity.phone.as_deref().unwrap()));
}

#[test]
fn ajoute_un_diplome_unique_sans_ecraser_la_formation_existante() {
    let source = "\
Formations\n\
Licence Histoire\n\
Nébula Université\n\
\n\
Master Informatique\n\
Orion Institut\n";
    let mut profile = Profile {
        education: vec![Education {
            degree: "Licence Histoire".into(),
            school: "Nébula Université".into(),
            ..Education::default()
        }],
        ..Profile::default()
    };
    let avant = profile.education[0].clone();

    completer_formations_manquantes(source, &mut profile);

    assert_eq!(profile.education.len(), 2);
    assert_eq!(profile.education[0], avant);
    assert_eq!(profile.education[1].degree, "Master Informatique");
    assert!(source.contains(profile.education[1].degree.as_str()));
    assert_eq!(profile.education[1].school, "Orion Institut");
    assert!(source.contains(profile.education[1].school.as_str()));
}

#[test]
fn ignore_une_fenetre_de_formation_ambigue() {
    let source = "\
Formations\n\
J'ai suivi un master puis une autre voie, voir Orion Institut et Pétale École.\n";
    let mut profile = Profile::default();

    completer_formations_manquantes(source, &mut profile);

    assert!(profile.education.is_empty());
}

#[test]
fn ne_duplique_pas_un_diplome_deja_couvert() {
    let source = "Master Informatique\nOrion Institut\n";
    let mut profile = Profile {
        education: vec![Education {
            degree: "Master Informatique".into(),
            school: "Orion Institut".into(),
            ..Education::default()
        }],
        ..Profile::default()
    };

    completer_formations_manquantes(source, &mut profile);

    assert_eq!(profile.education.len(), 1);
    assert_eq!(profile.education[0].degree, "Master Informatique");
    assert_eq!(profile.education[0].school, "Orion Institut");
}

#[test]
fn ajoute_plusieurs_formations_sans_paniquer_sur_la_couverture() {
    let source = "Formations\nLicence Histoire\nMaster Informatique\n";
    let mut profile = Profile::default();

    completer_formations_manquantes(source, &mut profile);

    assert_eq!(profile.education.len(), 2);
    assert!(profile
        .education
        .iter()
        .any(|formation| formation.degree == "Licence Histoire"));
    assert!(profile
        .education
        .iter()
        .any(|formation| formation.degree == "Master Informatique"));
}
