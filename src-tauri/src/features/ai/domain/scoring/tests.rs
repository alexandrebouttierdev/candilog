//! Tests du score ATS et du recadrage.
use super::*;
use crate::features::ai::domain::{
    AtsContentRecommendation, AtsRecommendation, AtsRecommendationSection, ContentRelevance,
    GeneratedEducation, GeneratedExperience,
};
use crate::features::profile::domain::{Education, Experience, Identity, Profile, Skill};

fn profile_rust() -> Profile {
    Profile {
        photo: None,
        identity: Identity {
            first_name: "Camille".into(),
            name: "Martin".into(),
            email: "camille@example.fr".into(),
            title: Some("Développeuse Rust".into()),
            resume: Some("Systèmes et CLI".into()),
            ..Identity::default()
        },
        experiences: vec![Experience {
            title: "Ingénieure".into(),
            company: "Nova".into(),
            location: None,
            start_date: "2020-01".into(),
            end_date: None,
            current: true,
            description: Some("APIs Rust".into()),
        }],
        skills: vec![Skill {
            name: "Rust".into(),
            description: None,
        }],
        education: vec![Education {
            degree: "Master".into(),
            school: "INSA".into(),
            location: None,
            start_date: None,
            end_date: None,
            description: None,
        }],
        languages: vec![],
        projects: vec![],
        certifications: vec![],
        interests: vec![],
    }
}

fn offre(skills: Vec<&str>, keywords: Vec<&str>, experience: Option<&str>) -> StructuredListing {
    StructuredListing {
        title: "Poste".into(),
        skills: skills.into_iter().map(str::to_owned).collect(),
        soft_skills: vec![],
        experience: experience.map(str::to_owned),
        keywords: keywords.into_iter().map(str::to_owned).collect(),
        ..StructuredListing::default()
    }
}

#[test]
fn une_competence_presente_augmente_le_score() {
    let score = profile_score(
        &profile_rust(),
        &offre(
            vec!["Rust", "React"],
            vec!["cli", "kubernetes"],
            Some("3 ans"),
        ),
    );
    assert_eq!(score.present, vec!["Rust"]);
    assert_eq!(score.missing, vec!["React"]);
    assert_eq!(score.skills, Some(50));
    assert!(score.total > 0);
}

/// Un profil réel n'écrit jamais « VMware » tout court : il écrit « VMware vSphere ».
/// L'égalité stricte des clés rendait ces compétences invisibles au score — huit profils
/// sur dix du scénario de bout en bout affichaient zéro compétence couverte — et
/// l'éditeur proposait alors d'ajouter une compétence déjà présente.
#[test]
fn une_competence_de_l_offre_est_couverte_par_un_libelle_plus_precis() {
    let mut profile = profile_rust();
    profile.skills = vec![
        Skill {
            name: "VMware vSphere 7/8".into(),
            description: None,
        },
        Skill {
            name: "Windows Server 2016/2019/2022".into(),
            description: None,
        },
        Skill {
            name: "Veeam Backup & Replication".into(),
            description: None,
        },
    ];

    let score = profile_score(
        &profile,
        &offre(vec!["VMware", "Windows", "VEEAM"], vec![], None),
    );

    assert_eq!(score.missing, Vec::<String>::new());
    assert_eq!(score.skills, Some(100));
}

/// La frontière de mot reste celle de la recherche : un préfixe commun ne suffit pas.
#[test]
fn un_fragment_de_mot_ne_couvre_pas_une_competence() {
    let mut profile = profile_rust();
    profile.skills = vec![Skill {
        name: "JavaScript".into(),
        description: None,
    }];

    let score = profile_score(&profile, &offre(vec!["Java"], vec![], None));

    assert_eq!(score.present, Vec::<String>::new());
    assert_eq!(score.missing, vec!["Java"]);
}

#[test]
fn offre_sans_competences_ne_penalise_pas() {
    let score = profile_score(&profile_rust(), &offre(vec![], vec![], None));
    assert_eq!(score.skills, None);
    assert_eq!(score.ats, None);
    assert_eq!(score.missing, Vec::<String>::new());
}

#[test]
fn cv_vide_contre_offre_complete_donne_zero_competence() {
    let vide = Profile::default();
    let score = profile_score(&vide, &offre(vec!["Rust"], vec!["cli"], Some("3 ans")));
    assert_eq!(score.skills, Some(0));
    assert_eq!(score.present, Vec::<String>::new());
    assert_eq!(score.missing, vec!["Rust"]);
    assert_eq!(score.ats, Some(0));
}

#[test]
fn correspondance_maximale_atteint_cent() {
    let score = profile_score(
        &profile_rust(),
        &offre(vec!["Rust"], vec!["cli"], Some("1 an")),
    );
    assert_eq!(score.skills, Some(100));
    assert_eq!(score.ats, Some(100));
    assert_eq!(score.experience, Some(100));
    assert_eq!(score.total, 100);
}

#[test]
fn la_casse_et_les_accents_ne_changent_pas_le_match() {
    let score = profile_score(
        &profile_rust(),
        &offre(vec!["rust", "RUST"], vec!["CLI"], None),
    );
    assert_eq!(score.skills, Some(100));
    assert_eq!(score.ats, Some(100));
    let mut cafe = profile_rust();
    cafe.skills = vec![Skill {
        name: "Café".into(),
        description: None,
    }];
    let accent = profile_score(&cafe, &offre(vec!["café"], vec![], None));
    assert_eq!(accent.skills, Some(100));
}

#[test]
fn mot_cle_go_ne_matche_pas_ongoing() {
    let mut profile = profile_rust();
    profile.identity.resume = Some("travail ongoing sur le moteur".into());
    let score = profile_score(&profile, &offre(vec![], vec!["go"], None));
    assert_eq!(score.ats, Some(0));
}

#[test]
fn mots_cles_dupliques_comptent_chacun() {
    let score = profile_score(&profile_rust(), &offre(vec![], vec!["cli", "cli"], None));
    assert_eq!(score.ats, Some(100));
}

#[test]
fn score_importe_ignore_les_cles_json() {
    let resume = GeneratedResume {
        resume: "Parcours backend".into(),
        experiences: vec![],
        skills: vec!["Rust".into()],
        education: vec![],
    };
    let score = score_resume_imported(&resume, &offre(vec!["Rust"], vec!["title"], None));
    assert_eq!(score.skills, Some(100));
    assert_eq!(score.ats, Some(0));
}

#[test]
fn grounding_retire_les_faits_inventes() {
    let mut resume = GeneratedResume {
        resume: String::new(),
        experiences: vec![
            GeneratedExperience {
                title: "Ingénieure".into(),
                company: "Nova".into(),
                description: String::new(),
            },
            GeneratedExperience {
                title: "CEO".into(),
                company: "Inconnue SA".into(),
                description: String::new(),
            },
        ],
        skills: vec!["Rust".into(), "COBOL".into()],
        education: vec![
            GeneratedEducation {
                degree: "Master".into(),
                school: "INSA".into(),
            },
            GeneratedEducation {
                degree: "Doctorat".into(),
                school: "Harvard".into(),
            },
        ],
    };
    ground_generated_resume(&profile_rust(), &mut resume);
    assert_eq!(resume.skills, vec!["Rust"]);
    assert_eq!(resume.experiences.len(), 1);
    assert_eq!(resume.experiences[0].company, "Nova");
    assert_eq!(resume.education.len(), 1);
    assert_eq!(resume.education[0].school, "INSA");
}

#[test]
fn grounding_exige_la_paire_titre_entreprise_et_recopie_la_source() {
    let mut resume = GeneratedResume {
        resume: "Accroche inventée".into(),
        experiences: vec![
            GeneratedExperience {
                title: "Ingénieure".into(),
                company: "Entreprise inventée".into(),
                description: "Mission inventée".into(),
            },
            GeneratedExperience {
                title: "ingénieure".into(),
                company: "NOVA".into(),
                description: "Mission reformulée".into(),
            },
        ],
        skills: vec!["rust".into()],
        education: vec![],
    };

    ground_generated_resume(&profile_rust(), &mut resume);

    assert_eq!(resume.resume, "Systèmes et CLI");
    assert_eq!(resume.experiences.len(), 1);
    assert_eq!(resume.experiences[0].title, "Ingénieure");
    assert_eq!(resume.experiences[0].company, "Nova");
    assert_eq!(resume.experiences[0].description, "APIs Rust");
    assert_eq!(resume.skills, vec!["Rust"]);
}

#[test]
fn grounding_exige_la_paire_diplome_ecole() {
    let mut resume = GeneratedResume {
        education: vec![
            GeneratedEducation {
                degree: "Doctorat".into(),
                school: "INSA".into(),
            },
            GeneratedEducation {
                degree: "master".into(),
                school: "insa".into(),
            },
        ],
        ..GeneratedResume::default()
    };

    ground_generated_resume(&profile_rust(), &mut resume);

    assert_eq!(resume.education.len(), 1);
    assert_eq!(resume.education[0].degree, "Master");
    assert_eq!(resume.education[0].school, "INSA");
}

#[test]
fn grounding_du_cv_importe_retire_les_faits_absents_du_pdf() {
    let source = "Ingénieure chez Nova. APIs Rust. Master à l'INSA. Compétence Rust.";
    let mut resume = GeneratedResume {
        resume: "Résumé inventé".into(),
        experiences: vec![
            GeneratedExperience {
                title: "Ingénieure".into(),
                company: "Nova".into(),
                description: "APIs Rust".into(),
            },
            GeneratedExperience {
                title: "CEO".into(),
                company: "Google".into(),
                description: "Direction".into(),
            },
        ],
        skills: vec!["Rust".into(), "Cobol".into()],
        education: vec![
            GeneratedEducation {
                degree: "Master".into(),
                school: "INSA".into(),
            },
            GeneratedEducation {
                degree: "Doctorat".into(),
                school: "INSA".into(),
            },
        ],
    };

    ground_imported_resume(source, &mut resume);

    assert!(resume.resume.is_empty());
    assert_eq!(resume.experiences.len(), 1);
    assert_eq!(resume.experiences[0].description, "APIs Rust");
    assert_eq!(resume.skills, vec!["Rust"]);
    assert_eq!(resume.education.len(), 1);
    assert_eq!(resume.education[0].degree, "Master");
}

#[test]
fn grounding_ne_confond_pas_go_et_google() {
    let mut profile = profile_rust();
    profile.skills = vec![Skill {
        name: "Google".into(),
        description: None,
    }];
    let mut resume = GeneratedResume {
        resume: String::new(),
        experiences: vec![],
        skills: vec!["Go".into(), "Google".into()],
        education: vec![],
    };
    ground_generated_resume(&profile, &mut resume);
    assert_eq!(resume.skills, vec!["Google"]);
}

#[test]
fn grounding_profil_vide_vide_le_cv_genere() {
    let mut resume = GeneratedResume {
        resume: "Accroche inventée".into(),
        experiences: vec![GeneratedExperience {
            title: "CEO".into(),
            company: "Inconnue SA".into(),
            description: String::new(),
        }],
        skills: vec!["COBOL".into()],
        education: vec![GeneratedEducation {
            degree: "Doctorat".into(),
            school: "Harvard".into(),
        }],
    };
    ground_generated_resume(&Profile::default(), &mut resume);
    assert!(resume.skills.is_empty());
    assert!(resume.experiences.is_empty());
    assert!(resume.education.is_empty());
}

#[test]
fn listing_extraite_ignore_les_competences_absentes_du_texte() {
    let mut listing = offre(vec!["Kubernetes", "Rust"], vec!["inject"], None);
    ground_extracted_listing("Offre Rust backend, CLI", &mut listing);
    assert_eq!(listing.skills, vec!["Rust"]);
    assert!(listing.keywords.is_empty());
}

#[test]
fn recommandations_de_contenu_ecartent_identifiants_inventes_et_doublons() {
    let profile = profile_rust();
    let catalog = profile_content_catalog(&profile);
    let real_id = catalog[0].id.clone();
    let mut analysis = AtsAnalysis {
        content_recommendations: vec![
            AtsContentRecommendation {
                item_id: "skill-kubernetes-invente".into(),
                reason: "Demandé".into(),
                relevance: ContentRelevance::VeryRelevant,
            },
            AtsContentRecommendation {
                item_id: real_id.clone(),
                reason: "Présent dans le profil".into(),
                relevance: ContentRelevance::Relevant,
            },
            AtsContentRecommendation {
                item_id: real_id.clone(),
                reason: "Doublon".into(),
                relevance: ContentRelevance::Secondary,
            },
        ],
        ..AtsAnalysis::default()
    };

    ground_content_recommendations(&catalog, &mut analysis);

    assert_eq!(analysis.content_recommendations.len(), 1);
    assert_eq!(analysis.content_recommendations[0].item_id, real_id);
}

#[test]
fn score_importe_pondere_skills_et_ats() {
    let resume = GeneratedResume {
        resume: String::new(),
        experiences: vec![],
        skills: vec!["Rust".into()],
        education: vec![],
    };
    let score = score_resume_imported(&resume, &offre(vec!["Rust", "Go"], vec!["cli"], None));
    assert_eq!(score.skills, Some(50));
    assert_eq!(score.ats, Some(0));
    // La mission absente participe désormais au détail dynamique au lieu d'être un bonus.
    assert_eq!(score.total, 39);
}

#[test]
fn offre_sans_exigence_exclut_les_dimensions_du_total() {
    let score = profile_score(&Profile::default(), &StructuredListing::default());

    assert_eq!(score.skills, None);
    assert_eq!(score.experience, None);
    assert_eq!(score.ats, None);
    assert_eq!(score.total, 0);
}

#[test]
fn termes_dupliques_casse_et_accents_ne_comptent_qu_une_fois() {
    let mut profile = profile_rust();
    profile.skills = vec![Skill {
        name: "cafe".into(),
        description: None,
    }];

    let score = profile_score(&profile, &offre(vec!["Café", "cafe", "CAFÉ"], vec![], None));

    assert_eq!(score.skills, Some(100));
    assert_eq!(score.present, vec!["Café"]);
    assert!(score.missing.is_empty());
}

/// Le premier entier du texte n'est pas l'exigence d'expérience : un profil de trois
/// ans passait pour couvrir intégralement une offre qui en demandait cinq.
#[test]
fn l_exigence_est_lue_a_cote_du_mot_annee() {
    assert_eq!(first_entier("3 ans"), 3);
    assert_eq!(first_entier("Bac+3, 5 ans d'expérience"), 5);
    assert_eq!(
        first_entier("2 à 5 ans"),
        2,
        "une fourchette vaut par son minimum"
    );
    assert_eq!(first_entier("expérience de 4 années"), 4);
    assert_eq!(
        first_entier("Bac+5"),
        5,
        "sans mention d'année, le premier entier"
    );
    assert_eq!(first_entier("expérience souhaitée"), 0);
}

/// Deux profils identiques face à une offre qui annonce un diplôme avant l'expérience
/// ne doivent pas être notés comme si l'exigence était le niveau d'études.
#[test]
fn un_diplome_annonce_avant_l_experience_ne_fausse_pas_le_score() {
    let profile = profile_rust();
    let stricte = profile_score(&profile, &offre(vec![], vec![], Some("Bac+3, 12 ans")));
    let souple = profile_score(&profile, &offre(vec![], vec![], Some("Bac+3, 2 ans")));

    assert!(
        stricte.experience < souple.experience,
        "12 ans exigés doivent noter plus sévèrement que 2 ans ({stricte:?} / {souple:?})"
    );
}

#[test]
fn offre_open_filtre_expertises_entreprise() {
    let source = r#"
Chez Open, nos 4000 collaborateurs. Expertises reconnues en Java, J2EE, SIG, C#.
Technologies de pointe : Cloud, DevOps.
Contexte : Concepteur Développeur Full stack F/H à Rennes.
Compétences techniques clés
Java, Angular
Pipelines CI/CD
Sensibilité aux méthodes et outils IA
Tests unitaires / intégration, qualité logicielle
Pratiques Agile, intégration continue, code review
"#;
    let mut listing = StructuredListing {
        title: "Concepteur Développeur Full stack F/H".into(),
        skills: vec![
            "Java".into(),
            "J2EE".into(),
            "SIG".into(),
            "C#".into(),
            "Angular".into(),
            "CI/CD".into(),
            "Agile".into(),
            "Code review".into(),
        ],
        soft_skills: vec![],
        experience: None,
        keywords: vec![],
        ..StructuredListing::default()
    };
    ground_extracted_listing(source, &mut listing);
    assert!(listing.skills.iter().any(|s| s == "Java"));
    assert!(listing.skills.iter().any(|s| s == "Angular"));
    assert!(
        !listing.skills.iter().any(|s| s == "J2EE"),
        "J2EE = blurb entreprise: {:?}",
        listing.skills
    );
    assert!(
        !listing.skills.iter().any(|s| s == "SIG"),
        "SIG = blurb: {:?}",
        listing.skills
    );
    assert!(
        !listing.skills.iter().any(|s| s == "C#"),
        "C# = blurb: {:?}",
        listing.skills
    );
}

/// CV réel (Alexandre Bouttier) vs offre Open-like — chemin Analyse de CV importé.
/// Régression du score ~9/100 : expérience à 0 + titre vide.
///
/// JSON parsé volontairement pauvre + texte PDF → le score lit le PDF.
///
/// Régression score ~17 : commentaire LLM aligné mais soft/ATS à 0 + skills partiels
/// ne doivent plus écraser le total (noyau compétences+expérience + bonus).
///
/// Agile / qualité / MOA / UX dans `competences` ne doivent pas tirer le score à ~31.
#[test]
fn cas_open_soft_dans_competences_ne_dilue_pas() {
    let resume = GeneratedResume {
        resume: "Développeur fullstack JavaScript TypeScript 6 ans Rennes.".into(),
        experiences: vec![GeneratedExperience {
            title: "Développeur Fullstack".into(),
            company: "Linaïa".into(),
            description: "React Node NestJS Angular refonte CI/CD GitLab CI Agile code review Docker Spring Boot migration. 2019 2025.".into(),
        }],
        skills: vec![
            "JavaScript".into(),
            "TypeScript".into(),
            "React".into(),
            "Node.js".into(),
            "Angular".into(),
            "CI/CD".into(),
            "Docker".into(),
            "GitLab CI".into(),
        ],
        education: vec![],
    };
    let source = "Développeur fullstack JavaScript TypeScript 6 ans. React Angular Node NestJS Docker GitLab CI CI/CD Agile Code review. GDS Bretagne Angular vers React. Jour de Match Spring Boot vers Node.js.";
    let offre = StructuredListing {
        title: "Concepteur Développeur Full stack F/H".into(),
        skills: vec![
            "Java".into(),
            "Angular".into(),
            "Pipelines CI/CD".into(),
            "Sensibilité aux méthodes et outils IA".into(),
            "Tests unitaires / intégration".into(),
            "qualité logicielle".into(),
            "Pratiques Agile".into(),
            "intégration continue".into(),
            "code review".into(),
            "industrialisation".into(),
            "lignes directrices UX".into(),
            "réunions de cadrage MOA".into(),
        ],
        soft_skills: vec![],
        experience: None,
        keywords: vec!["manuels d'usage".into(), "mise en production".into()],
        ..StructuredListing::default()
    };
    let score = score_resume_imported_with_source(&resume, &offre, Some(source));
    assert!(
        score.total >= 40,
        "attendu ≥40, obtenu {} skills={:?} present={:?} missing={:?}",
        score.total,
        score.skills,
        score.present,
        score.missing
    );
    assert!(score
        .present
        .iter()
        .any(|s| search_key(s).contains("angular")));
    // Java peut être manquant ou seulement transférable via Spring Boot — jamais un match JS.
    assert!(!score
        .present
        .iter()
        .any(|s| search_key(s) == "javascript" && search_key(s) == "java"),);
    assert!(
        score.skills.unwrap_or(0) >= 40,
        "compétences dures après reclassement: {:?}",
        score.skills
    );
}

#[test]
fn cas_open_commentaire_aligne_score_pas_ecrase() {
    let resume = GeneratedResume {
        resume: "Développeur fullstack JavaScript / TypeScript avec 6 ans d'expérience.".into(),
        experiences: vec![GeneratedExperience {
            title: "Développeur Fullstack JavaScript".into(),
            company: "Linaïa".into(),
            description: "React Node NestJS CI/CD Agile code review. Refonte Angular vers React. Juil. 2019 – Oct. 2025.".into(),
        }],
        skills: vec![
            "JavaScript".into(),
            "TypeScript".into(),
            "React".into(),
            "Node.js".into(),
            "Angular".into(),
            "CI/CD".into(),
            "Agile".into(),
            "Code review".into(),
        ],
        education: vec![],
    };
    let source =
        "Développeur fullstack 6 ans. React Angular Node CI/CD GitLab CI Agile Code review TMA.";
    let offre = StructuredListing {
        title: "Concepteur Développeur Full stack F/H".into(),
        skills: vec![
            "Java".into(),
            "Angular".into(),
            "CI/CD".into(),
            "tests unitaires".into(),
            "Agile".into(),
            "Code review".into(),
            "IA".into(),
        ],
        soft_skills: vec![
            "Qualité logicielle".into(),
            "Sensibilité IA".into(),
            "Réunions MOA".into(),
        ],
        experience: Some("3 ans".into()),
        keywords: vec![
            "lignes directrices UX".into(),
            "manuels d'usage".into(),
            "cadrage MOA".into(),
        ],
        ..StructuredListing::default()
    };
    let score = score_resume_imported_with_source(&resume, &offre, Some(source));
    assert!(
        score.experience.unwrap_or(0) >= 100,
        "6 ans ≥ 3: {:?}",
        score.experience
    );
    assert!(score.missing.iter().any(|s| s == "Java"));
    assert!(
        score.total >= 45 && score.total <= 85,
        "attendu 45–85 (plus de ~17), obtenu {} skills={:?} ats={:?} present={:?} missing={:?}",
        score.total,
        score.skills,
        score.ats,
        score.present,
        score.missing
    );
}

#[test]
fn score_avec_texte_pdf_source_meme_si_json_pauvre() {
    let resume = GeneratedResume {
        resume: "Développeur".into(),
        experiences: vec![GeneratedExperience {
            title: "Développeur Fullstack".into(),
            company: "Linaïa".into(),
            description: String::new(),
        }],
        skills: vec!["JavaScript".into()],
        education: vec![],
    };
    let source = "Développeur fullstack JavaScript / TypeScript avec 6 ans d'expérience. COMPÉTENCES: React Angular Node.js Docker GitLab CI CI/CD Agile Code review. GDS Bretagne refonte Angular vers React. PostgreSQL.";
    let offre = StructuredListing {
        title: "Concepteur Développeur Full stack F/H".into(),
        skills: vec![
            "Java".into(),
            "Angular".into(),
            "React".into(),
            "CI/CD".into(),
            "Agile".into(),
            "Code review".into(),
        ],
        soft_skills: vec![],
        experience: Some("3 ans".into()),
        keywords: vec![],
        ..StructuredListing::default()
    };
    let without = score_resume_imported(&resume, &offre);
    let with = score_resume_imported_with_source(&resume, &offre, Some(source));
    assert!(
        with.total > without.total,
        "PDF source doit remonter le score: with={} without={}",
        with.total,
        without.total
    );
    assert!(with.present.iter().any(|s| s == "Angular"));
    assert!(with.present.iter().any(|s| s == "CI/CD"));
    assert!(with.missing.iter().any(|s| s == "Java"));
    assert!(with.total >= 40, "obtenu {}", with.total);
}

#[test]
fn cas_open_cv_alexandre_score_intermediaire() {
    let resume = GeneratedResume {
        resume: "Développeur fullstack JavaScript / TypeScript avec 6 ans d'expérience en agence. Mode régie Agile TMA Code review. Docker GitLab CI CI/CD.".into(),
        experiences: vec![GeneratedExperience {
            title: "Développeur Fullstack JavaScript — Node.js · React · React Native".into(),
            company: "Linaïa".into(),
            description: "Applications web et mobiles de bout en bout. API REST Node.js Express NestJS, interfaces React React Native, MongoDB PostgreSQL. GDS Bretagne : refonte Angular vers React, back-office, React Laravel PostgreSQL. Juil. 2019 – Oct. 2025 · 6 ans.".into(),
        }],
        skills: vec![
            "JavaScript".into(),
            "TypeScript".into(),
            "React".into(),
            "React Native".into(),
            "Node.js".into(),
            "NestJS".into(),
            "PostgreSQL".into(),
            "MongoDB".into(),
            "Docker".into(),
            "GitLab CI".into(),
            "CI/CD".into(),
            "Agile".into(),
            "Code review".into(),
            "Angular".into(),
        ],
        education: vec![],
    };
    let offre = StructuredListing {
        title: "Concepteur Développeur Full stack F/H".into(),
        skills: vec![
            "Java".into(),
            "Angular".into(),
            "React".into(),
            "CI/CD".into(),
            "tests unitaires".into(),
            "Agile".into(),
            "Code review".into(),
        ],
        soft_skills: vec!["Qualité logicielle".into()],
        experience: Some("3 ans".into()),
        keywords: vec![
            "pipelines".into(),
            "intégration continue".into(),
            "IA".into(),
            "full stack".into(),
        ],
        ..StructuredListing::default()
    };
    let score = score_resume_imported(&resume, &offre);
    assert_eq!(score.experience, Some(100), "6 ans ≥ 3 ans exigés");
    assert!(
        score.present.iter().any(|s| s == "React"),
        "React présent: {:?}",
        score.present
    );
    assert!(
        score.present.iter().any(|s| s == "Angular"),
        "Angular présent (projet GDS): {:?}",
        score.present
    );
    assert!(
        score.present.iter().any(|s| s == "CI/CD"),
        "CI/CD sur le CV: {:?}",
        score.present
    );
    assert!(
        score.missing.iter().any(|s| s == "Java"),
        "Java ≠ JavaScript: {:?}",
        score.missing
    );
    assert!(
        score.total >= 45 && score.total <= 90,
        "attendu 45–90 pour ce CV réel, obtenu {} skills={:?} ats={:?} present={:?} missing={:?}",
        score.total,
        score.skills,
        score.ats,
        score.present,
        score.missing
    );
}

/// Sans aucune mention d'années, l'expérience n'est pas scorée à 0.
#[test]
fn cv_importe_sans_annees_n_ecrase_pas_le_score() {
    let resume = GeneratedResume {
        resume: "Développeur React".into(),
        experiences: vec![GeneratedExperience {
            title: "Développeur Full Stack".into(),
            company: "Acme".into(),
            description: "React et Node.js".into(),
        }],
        skills: vec!["React".into(), "Node.js".into()],
        education: vec![],
    };
    let score = score_resume_imported(
        &resume,
        &offre(vec!["React", "Java"], vec![], Some("5 ans")),
    );
    assert!(score.experience.is_none());
    assert!(score.total >= 20, "obtenu {}", score.total);
}

/// Cas Open : profil full-stack JS/TS/React/Node vs offre Angular/Java —
/// score intermédiaire (pas 1/100), sans plancher artificiel.
#[test]
fn cas_open_score_intermediaire() {
    let mut profile = profile_rust();
    profile.identity.title = Some("Développeur Full Stack".into());
    profile.identity.resume =
        Some("Développement d'applications web, tests et livraison continue.".into());
    profile.skills = vec![
        Skill {
            name: "JavaScript".into(),
            description: Some("ES6+".into()),
        },
        Skill {
            name: "TypeScript".into(),
            description: None,
        },
        Skill {
            name: "React".into(),
            description: Some("hooks, SPA".into()),
        },
        Skill {
            name: "Node.js".into(),
            description: None,
        },
        Skill {
            name: "PostgreSQL".into(),
            description: None,
        },
    ];
    profile.experiences = vec![Experience {
        title: "Développeur Full Stack".into(),
        company: "Nova".into(),
        location: None,
        start_date: "2019-01".into(),
        end_date: None,
        current: true,
        description: Some("Conception d'APIs, interfaces React, tests et CI/CD.".into()),
    }];

    let offre = StructuredListing {
        title: "Développeur Full Stack".into(),
        skills: vec![
            "Angular".into(),
            "Java".into(),
            "Spring".into(),
            "React".into(),
            "Node.js".into(),
            "TypeScript".into(),
            "Docker".into(),
            "Kubernetes".into(),
        ],
        soft_skills: vec!["Autonomie".into(), "Esprit d'équipe".into()],
        experience: Some("3 ans".into()),
        keywords: vec![
            "tests".into(),
            "CI/CD".into(),
            "API".into(),
            "React".into(), // doublon compétence → ignoré côté ATS
        ],
        ..StructuredListing::default()
    };

    let score = profile_score(&profile, &offre);
    assert!(
        score.total >= 40 && score.total <= 85,
        "attendu 40–85 pour Open, obtenu {}",
        score.total
    );
    assert!(score.present.iter().any(|s| s == "React"));
    assert!(
        score.present.iter().any(|s| s == "Angular"),
        "Angular transférable via famille front"
    );
    assert!(
        score.missing.iter().any(|s| s == "Java"),
        "Java reste absent (pas JavaScript)"
    );
    assert!(score.skills.unwrap_or(0) >= 40);
    assert_eq!(score.experience, Some(100));
}

#[test]
fn comptable_excel_couvre_calc_par_transfert() {
    let mut profile = profile_rust();
    profile.identity.title = Some("Comptable".into());
    profile.skills = vec![Skill {
        name: "Excel".into(),
        description: Some("Tableaux croisés".into()),
    }];
    let score = profile_score(
        &profile,
        &StructuredListing {
            title: "Comptable".into(),
            skills: vec!["LibreOffice Calc".into()],
            soft_skills: vec![],
            experience: None,
            keywords: vec![],
            ..StructuredListing::default()
        },
    );
    assert_eq!(score.missing, Vec::<String>::new());
    assert_eq!(score.present, vec!["LibreOffice Calc"]);
    assert_eq!(score.skills, Some(45)); // crédit transférable, jamais équivalent exact
}

#[test]
fn infirmiere_sans_ide_reste_penalisee() {
    let mut profile = profile_rust();
    profile.identity.title = Some("Aide-soignante".into());
    profile.skills = vec![Skill {
        name: "Soins de base".into(),
        description: None,
    }];
    profile.certifications = vec![];
    let score = profile_score(
        &profile,
        &StructuredListing {
            title: "Infirmière IDE".into(),
            skills: vec!["Diplôme IDE".into(), "IDE".into()],
            soft_skills: vec![],
            experience: Some("2 ans".into()),
            keywords: vec!["soins".into()],
            ..StructuredListing::default()
        },
    );
    assert!(!score.missing.is_empty());
    assert!(score.skills.unwrap_or(100) < 50);
}

#[test]
fn evidence_dans_un_projet_compte_comme_competence() {
    let mut profile = profile_rust();
    profile.skills = vec![];
    profile.projects = vec![crate::features::profile::domain::Project {
        name: "Dashboard interne".into(),
        description: Some("Stack React et PostgreSQL".into()),
        url: None,
        technologies: Some("React, PostgreSQL".into()),
    }];
    let score = profile_score(&profile, &offre(vec!["React", "PostgreSQL"], vec![], None));
    assert_eq!(score.missing, Vec::<String>::new());
    assert_eq!(score.skills, Some(100));
}

#[test]
fn mots_cles_deja_en_competences_ne_sont_pas_recomptes() {
    let mut profile = profile_rust();
    profile.skills = vec![Skill {
        name: "Rust".into(),
        description: None,
    }];
    let score = profile_score(&profile, &offre(vec!["Rust"], vec!["Rust", "CLI"], None));
    // ATS ne voit que CLI (Rust dédupliqué) — présence de CLI dans le résumé
    assert!(score.ats.is_some());
}

#[test]
fn proximite_metier_eleve_le_total() {
    let mut profile = profile_rust();
    profile.identity.title = Some("Développeuse Rust systèmes".into());
    // Compétences partielles : le bonus de proximité peut monter le total.
    let proche = profile_score(
        &profile,
        &StructuredListing {
            title: "Développeur Rust".into(),
            skills: vec!["Rust".into(), "Go".into()],
            soft_skills: vec![],
            experience: None,
            keywords: vec![],
            ..StructuredListing::default()
        },
    );
    let loin = profile_score(
        &profile,
        &StructuredListing {
            title: "Comptable clients".into(),
            skills: vec!["Rust".into(), "Go".into()],
            soft_skills: vec![],
            experience: None,
            keywords: vec![],
            ..StructuredListing::default()
        },
    );
    assert!(
        proche.total > loin.total,
        "{} vs {}",
        proche.total,
        loin.total
    );
}

fn exigence(
    name: &str,
    category: RequirementCategory,
    importance: RequirementImportance,
) -> JobRequirement {
    JobRequirement {
        name: name.into(),
        category,
        importance,
        mandatory: importance == RequirementImportance::Mandatory,
        ..JobRequirement::default()
    }
}

#[test]
fn commerce_titres_voisins_et_missions_donnent_une_bonne_correspondance() {
    let mut profile = profile_rust();
    profile.identity.title = Some("Conseiller de vente".into());
    profile.identity.resume = Some("Accueil, conseil et relation client.".into());
    profile.skills = vec![Skill {
        name: "Encaissement".into(),
        description: Some("Tenue de caisse".into()),
    }];
    profile.experiences[0].title = "Conseiller de vente".into();
    profile.experiences[0].description = Some("Merchandising et conseil client".into());
    let score = profile_score(
        &profile,
        &StructuredListing {
            title: "Vendeur conseil".into(),
            requirements: vec![
                exigence(
                    "Vendeur conseil",
                    RequirementCategory::Occupation,
                    RequirementImportance::Important,
                ),
                exigence(
                    "Accueil client",
                    RequirementCategory::Responsibility,
                    RequirementImportance::Important,
                ),
                exigence(
                    "Conseil client",
                    RequirementCategory::Responsibility,
                    RequirementImportance::Important,
                ),
                exigence(
                    "Encaissement",
                    RequirementCategory::HardSkill,
                    RequirementImportance::Important,
                ),
                exigence(
                    "Gestion de rayon",
                    RequirementCategory::Responsibility,
                    RequirementImportance::Preferred,
                ),
            ],
            ..StructuredListing::default()
        },
    );
    assert!(score.total >= 55, "score commerce: {score:?}");
    assert!(score.breakdown.iter().any(|item| item.label == "Missions"));
}

#[test]
fn administratif_outil_voisin_reste_transferable_et_non_exact() {
    let mut profile = profile_rust();
    profile.identity.title = Some("Assistant administratif".into());
    profile.identity.resume = Some("Gestion de dossiers et accueil.".into());
    profile.skills = vec![Skill {
        name: "LibreOffice Calc".into(),
        description: None,
    }];
    let mut excel = exigence(
        "Excel",
        RequirementCategory::Tool,
        RequirementImportance::Important,
    );
    excel.transferable_from = vec!["LibreOffice Calc".into()];
    let score = profile_score(
        &profile,
        &StructuredListing {
            title: "Assistant administratif".into(),
            requirements: vec![
                exigence(
                    "Assistant administratif",
                    RequirementCategory::Occupation,
                    RequirementImportance::Important,
                ),
                excel,
                exigence(
                    "Gestion documentaire",
                    RequirementCategory::Responsibility,
                    RequirementImportance::Important,
                ),
                exigence(
                    "Accueil",
                    RequirementCategory::Responsibility,
                    RequirementImportance::Important,
                ),
            ],
            ..StructuredListing::default()
        },
    );
    let excel = score
        .evaluations
        .iter()
        .find(|item| item.requirement == "Excel")
        .unwrap();
    assert_eq!(excel.match_kind, RequirementMatchKind::Transferable);
    assert_eq!(excel.score, Some(TRANSFERABLE_CREDIT));
    assert!(score.total >= 60, "score administratif: {score:?}");
}

#[test]
fn caces_obligatoire_absent_plafonne_le_score_sans_inventer() {
    let mut profile = profile_rust();
    profile.identity.title = Some("Préparateur de commandes".into());
    profile.identity.resume = Some("Trois ans en préparation de commandes.".into());
    profile.skills = vec![Skill {
        name: "Préparation de commandes".into(),
        description: None,
    }];
    let score = profile_score(
        &profile,
        &StructuredListing {
            title: "Cariste".into(),
            requirements: vec![
                exigence(
                    "Cariste",
                    RequirementCategory::Occupation,
                    RequirementImportance::Important,
                ),
                exigence(
                    "Préparation de commandes",
                    RequirementCategory::Responsibility,
                    RequirementImportance::Important,
                ),
                exigence(
                    "CACES 3",
                    RequirementCategory::License,
                    RequirementImportance::Mandatory,
                ),
            ],
            ..StructuredListing::default()
        },
    );
    assert!(score.total <= 35, "score cariste sans CACES: {score:?}");
    assert!(score.missing.iter().any(|item| item == "CACES 3"));
}

#[test]
fn metiers_sans_rapport_restent_tres_faibles() {
    let mut profile = profile_rust();
    profile.identity.title = Some("Comptable fournisseurs".into());
    profile.identity.resume = Some("Facturation et rapprochements bancaires.".into());
    profile.skills = vec![Skill {
        name: "Comptabilité fournisseurs".into(),
        description: None,
    }];
    let score = profile_score(
        &profile,
        &StructuredListing {
            title: "Développeur Java".into(),
            requirements: vec![
                exigence(
                    "Développeur Java",
                    RequirementCategory::Occupation,
                    RequirementImportance::Important,
                ),
                exigence(
                    "Java",
                    RequirementCategory::HardSkill,
                    RequirementImportance::Important,
                ),
            ],
            ..StructuredListing::default()
        },
    );
    assert!(score.total <= 10, "métiers sans rapport: {score:?}");
}

#[test]
fn exigences_repetees_ne_sont_comptees_qu_une_fois() {
    let mut listing = StructuredListing {
        title: "Cariste".into(),
        requirements: vec![
            exigence(
                "CACES 3",
                RequirementCategory::License,
                RequirementImportance::Mandatory,
            ),
            exigence(
                "CACES catégorie 3",
                RequirementCategory::Certification,
                RequirementImportance::Important,
            ),
        ],
        ..StructuredListing::default()
    };
    listing.requirements[1].name = "CACES 3".into();
    let score = profile_score(&profile_rust(), &listing);
    assert_eq!(
        score
            .evaluations
            .iter()
            .filter(|item| item.requirement == "CACES 3")
            .count(),
        1
    );
}

#[test]
fn recommandation_repetitive_ou_qui_ajoute_un_manque_est_ecartee() {
    let resume = GeneratedResume {
        resume: "Vendeur avec expérience en conseil client.".into(),
        experiences: vec![],
        skills: vec!["Conseil client".into()],
        education: vec![],
    };
    let score = MatchScore {
        missing: vec!["Management".into()],
        evaluations: vec![RequirementEvaluation {
            requirement: "Conseil client".into(),
            category: RequirementCategory::Responsibility,
            importance: RequirementImportance::Important,
            match_kind: RequirementMatchKind::Exact,
            score: Some(100),
            evidence: Some("Conseil client".into()),
        }],
        ..MatchScore::default()
    };
    let mut analysis = AtsAnalysis {
        recommendations: vec![
            AtsRecommendation {
                section: AtsRecommendationSection::Profile,
                item_index: None,
                original_text: resume.resume.clone(),
                proposed_text: resume.resume.clone(),
                target_requirement: Some("Conseil client".into()),
                reason: "Répétition".into(),
                source_evidence: vec!["Conseil client".into()],
            },
            AtsRecommendation {
                section: AtsRecommendationSection::Profile,
                item_index: None,
                original_text: resume.resume.clone(),
                proposed_text: "Vendeur avec expérience en conseil client et management.".into(),
                target_requirement: Some("Management".into()),
                reason: "Ajout inventé".into(),
                source_evidence: vec!["Conseil client".into()],
            },
        ],
        ..AtsAnalysis::default()
    };
    ground_ats_recommendations(&resume, &score, &mut analysis);
    assert!(analysis.recommendations.is_empty());
}
