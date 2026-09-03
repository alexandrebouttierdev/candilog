//! Validation et complétion du profil professionnel.

use crate::core::errors::{AppError, AppResult};
use crate::core::utils::text::search_key;
use crate::core::utils::validation::validate_optional_http_url;
use crate::features::profile::domain::{
    apply_decisions, build_preview, normaliser, nouveau_nom_fichier, Identity,
    ImportProfilePreview, ImportProfileRequest, ImportProfileResult, Profile, ProfilePayload,
    ProfileRepository, Skill, MAX_SOURCE_BYTES,
};
use base64::Engine;
use std::path::{Path, PathBuf};

/// Service métier du profil, générique sur son dépôt.
pub struct ProfileService<R: ProfileRepository> {
    repo: R,
    /// Dossier des photos, sous le dossier de données de l'utilisateur.
    photos_dir: PathBuf,
}

impl<R: ProfileRepository> ProfileService<R> {
    #[must_use]
    pub const fn new(repo: R, photos_dir: PathBuf) -> Self {
        Self { repo, photos_dir }
    }

    /// Payload le profil avec les informations de complétion nécessaires à l'écran.
    pub fn load(&self) -> AppResult<ProfilePayload> {
        let (profile, updated_at) = self.repo.get()?;
        Ok(enrichir(profile, updated_at))
    }

    /// Valide et remplace le profil complet.
    ///
    /// La photo est **reprise du profil enregistré** et non du payload reçu : les formulaires
    /// de l'écran Profile n'en portent pas, et l'enregistrement d'une section ne doit pas
    /// effacer une image que personne n'a demandé de retirer.
    pub fn save(&self, profile: &Profile) -> AppResult<ProfilePayload> {
        valider(profile)?;
        let (actuel, _) = self.repo.get()?;
        let mut profile = profile.clone();
        profile.photo = actuel.photo;
        let (profile, updated_at) = self.repo.save(&profile)?;
        Ok(enrichir(profile, Some(updated_at)))
    }

    /// Remplace la photo du profil par le fichier choisi, après validation et normalisation.
    ///
    /// L'ancienne image est supprimée une fois la nouvelle écrite : un échec en cours de
    /// route laisse donc le profil sur sa photo précédente, jamais sans photo.
    ///
    /// # Errors
    /// `AppError::Validation` si le fichier est illisible, trop volumineux ou d'un format
    /// refusé ; `AppError::Database` si l'écriture dans le dossier de données échoue.
    pub fn set_photo(&self, source: &Path) -> AppResult<ProfilePayload> {
        let metadata = std::fs::metadata(source).map_err(|error| {
            tracing::warn!(%error, "photo de profil inaccessible");
            AppError::Validation("Le fichier sélectionné est introuvable.".into())
        })?;
        // Contrôle avant lecture : inutile de charger en mémoire un fichier qu'on refusera.
        if metadata.len() > MAX_SOURCE_BYTES as u64 {
            return Err(AppError::Validation(format!(
                "L'image ne doit pas dépasser {} Mo.",
                MAX_SOURCE_BYTES / (1024 * 1024)
            )));
        }
        let bytes = std::fs::read(source).map_err(|error| {
            tracing::warn!(%error, "photo de profil illisible");
            AppError::Validation("Le fichier sélectionné n'a pas pu être lu.".into())
        })?;

        let png = normaliser(&bytes)?;
        let nom = nouveau_nom_fichier();
        self.ecrire_photo(&nom, &png)?;

        let (mut profile, _) = self.repo.get()?;
        let precedente = profile.photo.replace(nom);
        let (profile, updated_at) = self.repo.save(&profile)?;
        if let Some(ancienne) = precedente {
            self.supprimer_fichier(&ancienne);
        }
        Ok(enrichir(profile, Some(updated_at)))
    }

    /// Retire la photo du profil et supprime son fichier.
    ///
    /// # Errors
    /// `AppError::Database` si le profil ne peut pas être relu ou réenregistré.
    pub fn remove_photo(&self) -> AppResult<ProfilePayload> {
        let (mut profile, _) = self.repo.get()?;
        let precedente = profile.photo.take();
        let (profile, updated_at) = self.repo.save(&profile)?;
        if let Some(ancienne) = precedente {
            self.supprimer_fichier(&ancienne);
        }
        Ok(enrichir(profile, Some(updated_at)))
    }

    /// Photo du profil encodée en `data:` URL, ou `None` si le profil n'en a pas.
    ///
    /// La webview n'a pas accès au dossier de données : l'image lui parvient par l'IPC comme
    /// toute autre donnée, sans ouvrir de protocole de fichiers.
    ///
    /// # Errors
    /// `AppError::Database` si le profil ne peut pas être lu.
    pub fn photo_data_url(&self) -> AppResult<Option<String>> {
        let Some(chemin) = self.photo_path()? else {
            return Ok(None);
        };
        match std::fs::read(&chemin) {
            Ok(bytes) => Ok(Some(format!(
                "data:image/png;base64,{}",
                base64::engine::general_purpose::STANDARD.encode(bytes)
            ))),
            // Fichier disparu — sauvegarde restaurée, ménage manuel : le profil s'affiche
            // sans photo plutôt que d'échouer entièrement.
            Err(error) => {
                tracing::warn!(%error, "photo de profil absente du dossier de données");
                Ok(None)
            }
        }
    }

    /// Path absolu de la photo, pour le moteur PDF.
    ///
    /// # Errors
    /// `AppError::Database` si le profil ne peut pas être lu.
    pub fn photo_path(&self) -> AppResult<Option<PathBuf>> {
        let (profile, _) = self.repo.get()?;
        Ok(profile.photo.as_deref().and_then(|nom| {
            let chemin = self.photos_dir.join(nom);
            chemin.is_file().then_some(chemin)
        }))
    }

    /// Réinitialise **le seul profil** : identité, sections et photo.
    ///
    /// Rien d'autre n'est touché. Candidatures, entreprises, contacts, entretiens, relances,
    /// documents, réglages et configuration IA vivent dans d'autres tables, qu'aucune requête
    /// d'ici n'atteint.
    ///
    /// # Errors
    /// `AppError::Database` si la ligne du profil ne peut pas être réécrite.
    pub fn reset(&self) -> AppResult<ProfilePayload> {
        let (actuel, _) = self.repo.get()?;
        let (profile, updated_at) = self.repo.save(&Profile::default())?;
        if let Some(ancienne) = actuel.photo {
            self.supprimer_fichier(&ancienne);
        }
        Ok(enrichir(profile, Some(updated_at)))
    }

    fn ecrire_photo(&self, nom: &str, png: &[u8]) -> AppResult<()> {
        std::fs::create_dir_all(&self.photos_dir).map_err(|error| {
            tracing::error!(%error, "dossier des photos non créé");
            AppError::Database("Le dossier des photos n'a pas pu être créé.".into())
        })?;
        let chemin = self.photos_dir.join(nom);
        std::fs::write(&chemin, png).map_err(|error| {
            tracing::error!(%error, "photo de profil non enregistrée");
            AppError::Database("La photo n'a pas pu être enregistrée.".into())
        })?;
        crate::core::config::restreindre_fichier(&chemin);
        Ok(())
    }

    /// Supprime un fichier photo devenu inutile, sans faire échouer l'opération appelante.
    fn supprimer_fichier(&self, nom: &str) {
        let chemin = self.photos_dir.join(nom);
        match std::fs::remove_file(&chemin) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => tracing::warn!(%error, "ancienne photo de profil non supprimée"),
        }
    }

    /// Compare un CV extrait au profil actuel sans rien écrire.
    pub fn preview_import(&self, extracted: &Profile) -> AppResult<ImportProfilePreview> {
        let (current, _) = self.repo.get()?;
        Ok(build_preview(&current, extracted))
    }

    /// Applique les décisions d'import en une seule écriture.
    pub fn apply_import(&self, request: &ImportProfileRequest) -> AppResult<ImportProfileResult> {
        let (current, _) = self.repo.get()?;
        let (merged, result) = apply_decisions(&current, request)?;
        valider(&merged)?;
        self.repo.save(&merged)?;
        Ok(result)
    }

    /// Ajoute une compétence au profil, sans doublon selon une comparaison normalisée
    /// (casse, accents, espaces) : l'éditeur de CV propose d'ajouter une compétence attendue
    /// par l'offre, et rejouer l'action ne doit jamais créer d'entrée en double.
    ///
    /// # Errors
    /// `AppError::Validation` si le nom, une fois retiré de ses espaces, est vide.
    pub fn add_skill(&self, name: &str) -> AppResult<ProfilePayload> {
        let name = name.trim();
        if name.is_empty() {
            return Err(AppError::Validation("La compétence est requise".into()));
        }
        let (mut profile, _) = self.repo.get()?;
        if !profile
            .skills
            .iter()
            .any(|skill| search_key(&skill.name) == search_key(name))
        {
            profile.skills.push(Skill { name: name.into() });
        }
        self.save(&profile)
    }
}

fn enrichir(profile: Profile, updated_at: Option<String>) -> ProfilePayload {
    let sections = sections_complete(&profile);
    let complete = sections.iter().filter(|(_, complete)| *complete).count() as u16;
    let completion = ((complete * 100 + 3) / 7) as u8;
    ProfilePayload {
        profile,
        completion,
        incomplete_sections: sections
            .into_iter()
            .filter(|(_, complete)| !complete)
            .map(|(label, _)| label.to_owned())
            .collect(),
        updated_at,
    }
}

fn sections_complete(profile: &Profile) -> [(&'static str, bool); 7] {
    [
        ("votre identité", identity_complete(&profile.identity)),
        (
            "une expérience",
            profile.experiences.iter().any(|item| {
                !item.title.trim().is_empty()
                    && !item.company.trim().is_empty()
                    && !item.start_date.trim().is_empty()
            }),
        ),
        (
            "vos compétences",
            profile
                .skills
                .iter()
                .any(|item| !item.name.trim().is_empty()),
        ),
        (
            "une formation",
            profile
                .education
                .iter()
                .any(|item| !item.degree.trim().is_empty() && !item.school.trim().is_empty()),
        ),
        (
            "une langue",
            profile
                .languages
                .iter()
                .any(|item| !item.name.trim().is_empty() && !item.level.trim().is_empty()),
        ),
        (
            "un projet",
            profile
                .projects
                .iter()
                .any(|item| !item.name.trim().is_empty()),
        ),
        (
            "une certification",
            profile
                .certifications
                .iter()
                .any(|item| !item.name.trim().is_empty()),
        ),
    ]
}

fn identity_complete(identity: &Identity) -> bool {
    !identity.first_name.trim().is_empty()
        && !identity.name.trim().is_empty()
        && !identity.email.trim().is_empty()
}

fn valider(profile: &Profile) -> AppResult<()> {
    let email = profile.identity.email.trim();
    if !email.is_empty() && !email_valide(email) {
        return Err(AppError::Validation("L'adresse e-mail est invalide".into()));
    }
    validate_optional_http_url(profile.identity.linkedin.as_deref(), "Le profil LinkedIn")?;
    validate_optional_http_url(profile.identity.github.as_deref(), "Le profil GitHub")?;
    validate_optional_http_url(profile.identity.website.as_deref(), "Le site web")?;

    for experience in &profile.experiences {
        if experience.title.trim().is_empty()
            || experience.company.trim().is_empty()
            || experience.start_date.trim().is_empty()
        {
            return Err(AppError::Validation(
                "Chaque expérience nécessite un intitulé, une entreprise et une date de début"
                    .into(),
            ));
        }
        if experience.current && experience.end_date.is_some() {
            return Err(AppError::Validation(
                "Un poste actuel ne peut pas avoir de date de fin".into(),
            ));
        }
    }
    if profile
        .skills
        .iter()
        .any(|item| item.name.trim().is_empty())
    {
        return Err(AppError::Validation(
            "Chaque compétence nécessite un nom".into(),
        ));
    }
    if profile
        .education
        .iter()
        .any(|item| item.degree.trim().is_empty() || item.school.trim().is_empty())
    {
        return Err(AppError::Validation(
            "Chaque formation nécessite un diplôme et un établissement".into(),
        ));
    }
    if profile
        .languages
        .iter()
        .any(|item| item.name.trim().is_empty() || item.level.trim().is_empty())
    {
        return Err(AppError::Validation(
            "Chaque langue nécessite un nom et un niveau".into(),
        ));
    }
    if profile
        .projects
        .iter()
        .any(|item| item.name.trim().is_empty())
    {
        return Err(AppError::Validation(
            "Chaque projet nécessite un nom".into(),
        ));
    }
    if profile
        .certifications
        .iter()
        .any(|item| item.name.trim().is_empty())
    {
        return Err(AppError::Validation(
            "Chaque certification nécessite un nom".into(),
        ));
    }
    for project in &profile.projects {
        validate_optional_http_url(project.url.as_deref(), "Le lien du projet")?;
    }
    for certification in &profile.certifications {
        validate_optional_http_url(certification.url.as_deref(), "Le lien de la certification")?;
    }
    Ok(())
}

fn email_valide(email: &str) -> bool {
    email.split_once('@').is_some_and(|(local, domaine)| {
        !local.is_empty()
            && domaine.contains('.')
            && !domaine.starts_with('.')
            && !domaine.ends_with('.')
    })
}

#[cfg(test)]
#[path = "tests/service/mod.rs"]
mod tests;

#[cfg(test)]
#[path = "tests/reset/mod.rs"]
mod tests_reset;
