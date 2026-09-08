//! Recadrage sur le CV d'un profil extrait par un modèle.
//!
//! L'import de profil demande au modèle de recopier le CV sans rien inventer. Les petits
//! modèles locaux ne s'y tiennent pas : ils renvoient des fragments de domaine (« .com »,
//! « .fr », « .org »), des morceaux de mots (« .franc ») et jusqu'à une certification
//! absente du document. Le nettoyage par vacuité ne rejetait que le vide, si bien que ces
//! valeurs arrivaient telles quelles dans l'écran de revue.
//!
//! On ne conserve donc qu'un texte réellement présent dans le CV analysé, comme
//! `ground_imported_resume` le fait déjà pour l'analyse de CV.

use super::normalization::contains_search_term;
use crate::features::profile::domain::Profile;

/// Efface d'un profil importé tout texte absent du CV analysé.
///
/// Les dates ne sont pas concernées : `normalize_profile_dates` les a déjà ramenées au
/// format `AAAA-MM`, que le CV n'écrit pas tel quel (« Oct. 2025 »). Les entrées vidées de
/// leur libellé sont retirées ensuite par le nettoyage par vacuité.
pub fn ground_imported_profile(source: &str, profile: &mut Profile) {
    let identity = &mut profile.identity;
    retenir(source, &mut identity.first_name);
    retenir(source, &mut identity.name);
    retenir(source, &mut identity.email);
    retenir_option(source, &mut identity.phone);
    retenir_option(source, &mut identity.address);
    retenir_option(source, &mut identity.city);
    retenir_option(source, &mut identity.title);
    retenir_option(source, &mut identity.resume);
    retenir_option(source, &mut identity.linkedin);
    retenir_option(source, &mut identity.github);
    retenir_option(source, &mut identity.website);

    for experience in &mut profile.experiences {
        retenir(source, &mut experience.title);
        retenir(source, &mut experience.company);
        retenir_option(source, &mut experience.location);
        retenir_option(source, &mut experience.description);
    }
    for skill in &mut profile.skills {
        retenir(source, &mut skill.name);
    }
    for education in &mut profile.education {
        retenir(source, &mut education.degree);
        retenir(source, &mut education.school);
        retenir_option(source, &mut education.location);
        retenir_option(source, &mut education.description);
    }
    for language in &mut profile.languages {
        retenir(source, &mut language.name);
        retenir(source, &mut language.level);
    }
    for project in &mut profile.projects {
        retenir(source, &mut project.name);
        retenir_option(source, &mut project.description);
        retenir_option(source, &mut project.url);
        retenir_option(source, &mut project.technologies);
    }
    for certification in &mut profile.certifications {
        retenir(source, &mut certification.name);
        retenir_option(source, &mut certification.issuer);
        retenir_option(source, &mut certification.url);
    }
}

/// Vide une valeur que le CV ne contient pas.
fn retenir(source: &str, value: &mut String) {
    if !est_recopie(source, value) {
        value.clear();
    }
}

/// Retire une valeur optionnelle que le CV ne contient pas.
fn retenir_option(source: &str, value: &mut Option<String>) {
    if !value
        .as_deref()
        .is_some_and(|texte| est_recopie(source, texte))
    {
        *value = None;
    }
}

/// Une valeur est recopiée si elle porte du texte et apparaît telle quelle dans le CV.
///
/// La recherche exige des frontières alphanumériques : c'est elle qui écarte « .com », que
/// le CV n'écrit qu'accolé à un domaine (`gmail.com`), sans rejeter une valeur légitime.
/// Le garde sur les caractères alphanumériques écarte en amont les valeurs sans contenu
/// (« . »), qu'une ponctuation quelconque du CV suffirait sinon à valider.
fn est_recopie(source: &str, value: &str) -> bool {
    value.chars().any(char::is_alphanumeric) && contains_search_term(source, value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::profile::domain::{
        Certification, Experience, Identity, Language, Project, Skill,
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
}
