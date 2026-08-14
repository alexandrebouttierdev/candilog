//! Helpers communs et déclaration des cas de test.
use super::*;
use crate::shared::profile::{
    Certification, Education, Experience, Language, PersonalInfo, Profile, Project, Skill,
};
use std::sync::Mutex;

struct StubRepo {
    store: Mutex<Option<Profile>>,
}
impl ProfilRepository for StubRepo {
    fn get(&self) -> AppResult<Profile> {
        Ok(self.store.lock().unwrap().clone().unwrap_or_default())
    }
    fn upsert(&self, p: &Profile) -> AppResult<Profile> {
        *self.store.lock().unwrap() = Some(p.clone());
        Ok(p.clone())
    }
}
fn service() -> ProfilService<StubRepo> {
    ProfilService::new(StubRepo {
        store: Mutex::new(None),
    })
}

/// Profil dont exactement `count` sections sont complètes.
fn profile_avec_sections(count: usize) -> Profile {
    let mut profile = Profile::default();
    if count >= 1 {
        profile.personal = PersonalInfo {
            first_name: "Alice".into(),
            last_name: "Dupont".into(),
            email: "alice@dupont.fr".into(),
            ..PersonalInfo::default()
        };
    }
    if count >= 2 {
        profile.experiences.push(Experience {
            title: "Développeuse".into(),
            company: "ACME".into(),
            ..Experience::default()
        });
    }
    if count >= 3 {
        profile.skills.push(Skill {
            name: "Rust".into(),
        });
    }
    if count >= 4 {
        profile.education.push(Education {
            degree: "Master".into(),
            school: "Université".into(),
            ..Education::default()
        });
    }
    if count >= 5 {
        profile.languages.push(Language {
            name: "Français".into(),
            level: "Natif".into(),
        });
    }
    if count >= 6 {
        profile.projects.push(Project {
            name: "Candilog".into(),
            ..Project::default()
        });
    }
    if count >= 7 {
        profile.certifications.push(Certification {
            name: "Certification".into(),
            ..Certification::default()
        });
    }
    profile
}

mod test_l_identite_n_est_complete_qu_avec_nom_prenom_et_email;
mod test_un_profil_a_moitie_score_proche_de_cinquante;
mod test_un_profil_complet_score_cent;
mod test_un_profil_vide_score_zero;
mod test_update_email_invalide_retourne_erreur;
mod test_update_experience_sans_titre_retourne_erreur;
mod test_update_formation_vide_retourne_erreur;
mod test_update_profil_valide_persiste;
