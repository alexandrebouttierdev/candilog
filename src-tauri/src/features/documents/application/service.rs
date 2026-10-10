//! Validation des CV et lettres avant persistance, et orchestration des exports PDF.

use crate::core::errors::{AppError, AppResult};
use crate::core::files::atomic_write;
use crate::core::pagination::Page;
use crate::features::ai::domain::{profile_without, ProfileSection, ResumeGeneration};
use crate::features::documents::application::{
    apply_proposal, build, build_cover_letter, compose_base_resume, prepare_workspace, recalculate,
    reject_proposal, validate_document,
};
use crate::features::documents::domain::{
    sanitize_letter, BaseResume, CoverLetter, CoverLetterExport, CoverLetterRepository,
    DocumentVersion, NewCoverLetter, NewResume, ResumeDocument, ResumeRepository, ResumeSummary,
    ResumeVersion, ResumeWorkspace, RESUME_BASE_KIND, RESUME_WORKSPACE_VERSION,
};
use crate::features::profile::application::ProfileService;
use crate::features::profile::domain::ProfileRepository;
use std::path::Path;
use std::sync::Arc;
use uuid::Uuid;

/// Borne du JSON d'une version de CV.
///
/// `content` est volontairement extensible — le modèle de génération évolue — mais rester
/// non borné laissait entrer en base un blob arbitraire par un simple appel IPC. La valeur
/// couvre très largement un CV complet sérialisé.
pub const MAX_CONTENT_CHARS: usize = 250_000;

/// Borne du texte d'une lettre, cohérente avec ce qu'un PDF d'une page peut porter.
pub const MAX_LETTER_CHARS: usize = 20_000;

/// Tons acceptés, alignés sur ceux que le rendu de lettre sait interpréter.
const TONES: [&str; 3] = ["formal", "casual", "creative"];

/// Longueurs acceptées, alignées sur celles que le rendu de lettre sait interpréter.
const LENGTHS: [&str; 3] = ["short", "medium", "long"];

pub struct DocumentsService<C: ResumeRepository, L: CoverLetterRepository, P: ProfileRepository> {
    resume: C,
    cover_letters: L,
    profile: Arc<ProfileService<P>>,
}

impl<C: ResumeRepository, L: CoverLetterRepository, P: ProfileRepository>
    DocumentsService<C, L, P>
{
    #[must_use]
    pub fn new(resume: C, cover_letters: L, profile: Arc<ProfileService<P>>) -> Self {
        Self {
            resume,
            cover_letters,
            profile,
        }
    }

    /// Fige le profil et une génération IA dans un document de travail autonome.
    ///
    /// Les sections que l'utilisateur a retirées de la génération ne reviennent ni dans le
    /// document ni dans la bibliothèque de contenu proposée ensuite.
    pub fn resume_prepare(
        &self,
        generation: ResumeGeneration,
        excluded_sections: &[ProfileSection],
    ) -> AppResult<ResumeWorkspace> {
        let payload = self.profile.load()?;
        let photo = self.profile.photo_bytes()?;
        prepare_workspace(
            &profile_without(&payload.profile, excluded_sections),
            generation,
            photo,
        )
    }

    /// Compose un CV de base depuis le profil enregistré.
    ///
    /// # Errors
    /// Retourne une erreur de lecture du profil, ou une validation si le document composé
    /// dépasse les bornes d'édition.
    pub fn compose_base_resume(
        &self,
        excluded_sections: Vec<ProfileSection>,
    ) -> AppResult<ResumeDocument> {
        let payload = self.profile.load()?;
        compose_base_resume(&payload.profile, &excluded_sections)
    }

    /// Revalide le document puis recalcule score et propositions après une édition manuelle.
    pub fn resume_recalculate(&self, workspace: ResumeWorkspace) -> AppResult<ResumeWorkspace> {
        let photo = self.profile.photo_bytes()?;
        recalculate(workspace, photo)
    }

    /// Applique une proposition puis recalcule le poste de travail.
    pub fn resume_apply_proposal(
        &self,
        workspace: ResumeWorkspace,
        proposal_id: &str,
    ) -> AppResult<ResumeWorkspace> {
        let photo = self.profile.photo_bytes()?;
        apply_proposal(workspace, proposal_id, photo)
    }

    /// Refuse une proposition sans modifier le document, puis recalcule le poste de travail.
    pub fn resume_reject_proposal(
        &self,
        workspace: ResumeWorkspace,
        proposal_id: &str,
    ) -> AppResult<ResumeWorkspace> {
        let photo = self.profile.photo_bytes()?;
        reject_proposal(workspace, proposal_id, photo)
    }

    /// Exporte un document CV autonome au chemin indiqué.
    ///
    /// La photo suit le profil courant, pas la version de CV enregistrée : un CV rouvert
    /// après suppression de la photo s'exporte sans elle, sans laisser de cadre vide.
    pub fn resume_export_pdf(
        &self,
        document: &ResumeDocument,
        destination: &Path,
    ) -> AppResult<()> {
        let photo = self.profile.photo_bytes()?;
        let bytes = build(document, photo).render_bytes()?;
        atomic_write(destination, "pdf", |temporaire| {
            std::fs::write(temporaire, &bytes).map_err(|error| {
                tracing::error!(%error, "export PDF impossible");
                AppError::Database(format!("Écriture du PDF impossible : {error}"))
            })
        })
    }

    /// Exporte une lettre au chemin indiqué, avec l'identité du profil en en-tête.
    pub fn cover_letter_export_pdf(
        &self,
        cover_letter: &CoverLetterExport,
        destination: &Path,
    ) -> AppResult<()> {
        let payload = self.profile.load()?;
        let bytes = build_cover_letter(&payload.profile, cover_letter).render_bytes()?;
        atomic_write(destination, "pdf", |temporaire| {
            std::fs::write(temporaire, &bytes).map_err(|error| {
                tracing::error!(%error, "export PDF de lettre impossible");
                AppError::Database(format!("Écriture du PDF de lettre impossible : {error}"))
            })
        })
    }

    /// Valide puis enregistre une version de CV.
    ///
    /// # Errors
    /// `AppError::Validation` si le nom est vide ou trop long, si le contenu n'est pas un
    /// objet JSON, ou s'il dépasse [`MAX_CONTENT_CHARS`].
    pub fn resume_save(&self, input: &NewResume) -> AppResult<ResumeVersion> {
        let name = input.name.trim();
        if name.is_empty() {
            return Err(AppError::Validation(
                "Le nom de la version est requis".into(),
            ));
        }
        if name.chars().count() > 120 {
            return Err(AppError::Validation(
                "Le nom de la version est trop long (120 max)".into(),
            ));
        }
        valider_contenu(&input.content)?;
        self.resume.save(&NewResume {
            name: name.into(),
            content: input.content.clone(),
            revises: input.revises,
            version_note: version_note(input.version_note.as_deref())?,
        })
    }

    /// Versions du CV auquel appartient `id`, de la plus récente à la plus ancienne.
    ///
    /// # Errors
    /// `NotFound` si la version n'existe pas.
    pub fn resume_versions(&self, id: Uuid) -> AppResult<Vec<DocumentVersion>> {
        self.resume.versions(id)
    }

    /// Fait de la version `id` la version courante de son CV.
    ///
    /// # Errors
    /// `NotFound` si la version n'existe pas.
    pub fn resume_restore(&self, id: Uuid) -> AppResult<()> {
        self.resume.restore(id)
    }

    pub fn resume_list_page(
        &self,
        page: u64,
        page_size: u64,
        search: &str,
        scored_only: bool,
    ) -> AppResult<Page<ResumeSummary>> {
        self.resume.list_page(page, page_size, search, scored_only)
    }
    pub fn resume_get(&self, id: Uuid) -> AppResult<ResumeVersion> {
        self.resume.get(id)
    }
    pub fn resume_delete(&self, id: Uuid) -> AppResult<()> {
        self.resume.delete(id)
    }

    /// Valide puis enregistre une lettre.
    ///
    /// Le ton et la longueur sont vérifiés ici comme ils le sont au rendu : les accepter
    /// librement à la persistance laissait la même règle produire deux résultats selon la
    /// couche traversée, et une lettre au ton inconnu échouait ensuite à la régénération.
    ///
    /// # Errors
    /// `AppError::Validation` si le nom, le contenu, le ton ou la longueur sont invalides.
    pub fn cover_letter_save(&self, input: &NewCoverLetter) -> AppResult<CoverLetter> {
        let name = input.name.trim();
        if name.is_empty() {
            return Err(AppError::Validation(
                "Le nom de la lettre est requis".into(),
            ));
        }
        if name.chars().count() > 140 {
            return Err(AppError::Validation(
                "Le nom de la lettre est trop long".into(),
            ));
        }
        if input.content.trim().is_empty() {
            return Err(AppError::Validation(
                "Générez une lettre avant de l'enregistrer".into(),
            ));
        }
        if input.content.chars().count() > MAX_LETTER_CHARS {
            return Err(AppError::Validation(
                "Le contenu de la lettre est trop long".into(),
            ));
        }
        valider_valeur(&input.tone, &TONES, "Le ton de la lettre")?;
        valider_valeur(&input.length, &LENGTHS, "La longueur de la lettre")?;
        let mut nettoyee = input.clone();
        nettoyee.name = name.into();
        // Le corps porte la mise en forme de l'éditeur : il est ramené au balisage canonique
        // avant d'entrer en base, seul endroit où l'on peut garantir que rien d'autre n'y est.
        nettoyee.content = sanitize_letter(&input.content);
        nettoyee.version_note = version_note(input.version_note.as_deref())?;
        self.cover_letters.save(&nettoyee)
    }

    /// Versions de la lettre à laquelle appartient `id`, de la plus récente à la plus ancienne.
    ///
    /// # Errors
    /// `NotFound` si la version n'existe pas.
    pub fn cover_letter_versions(&self, id: Uuid) -> AppResult<Vec<DocumentVersion>> {
        self.cover_letters.versions(id)
    }

    /// Fait de la version `id` la version courante de sa lettre.
    ///
    /// # Errors
    /// `NotFound` si la version n'existe pas.
    pub fn cover_letter_restore(&self, id: Uuid) -> AppResult<()> {
        self.cover_letters.restore(id)
    }

    pub fn cover_letters_list_page(
        &self,
        page: u64,
        page_size: u64,
        search: &str,
    ) -> AppResult<Page<CoverLetter>> {
        self.cover_letters.list_page(page, page_size, search)
    }
    pub fn cover_letter_get(&self, id: Uuid) -> AppResult<CoverLetter> {
        self.cover_letters.get(id)
    }
    pub fn cover_letter_delete(&self, id: Uuid) -> AppResult<()> {
        self.cover_letters.delete(id)
    }
}

/// Refuse un contenu de CV qui n'est pas un objet JSON borné.
///
/// Trois branches, évaluées dans cet ordre :
/// - `kind == "base"` : CV de base en `schema_version: 1`, désérialisé en [`BaseResume`] et
///   son document revalidé. Elle est testée avant `schema_version`, car un CV de base et un
///   CV ciblé partagent tous deux `schema_version: 1` ; seul `kind` les distingue, et
///   l'inverser ferait retomber un CV de base dans la désérialisation en [`ResumeWorkspace`],
///   qui échoue faute de `job_offer`, `analysis` et `score`. Une autre version est refusée :
///   seule celle-ci se relit.
/// - `schema_version == 1` sans `kind` : CV ciblé (l'éditeur de CV autonome), désérialisé en
///   [`ResumeWorkspace`] et son document revalidé.
/// - toute autre forme : contenu historique, accepté tel quel pour la lecture et la
///   duplication.
///
/// Sans les deux premières branches, une forme incohérente pourrait entrer en base par un
/// appel IPC forgé.
fn valider_contenu(content: &serde_json::Value) -> AppResult<()> {
    if !content.is_object() {
        return Err(AppError::Validation(
            "Le contenu du CV est illisible : générez-le à nouveau avant de l'enregistrer".into(),
        ));
    }
    let serialise = serde_json::to_string(content)
        .map_err(|error| AppError::Serialization(error.to_string()))?;
    if serialise.chars().count() > MAX_CONTENT_CHARS {
        return Err(AppError::Validation(
            "Le contenu du CV dépasse la taille maximale autorisée".into(),
        ));
    }
    if content.get("kind") == Some(&serde_json::json!(RESUME_BASE_KIND)) {
        // La version est exigée ici comme dans la branche workspace : sans elle, un
        // `kind: "base"` forgé dans une version inconnue entrerait en base, et la
        // bibliothèque le déclarerait ensuite illisible faute de savoir le relire.
        if content.get("schema_version") != Some(&serde_json::json!(RESUME_WORKSPACE_VERSION)) {
            return Err(AppError::Validation(
                "Le contenu du CV est invalide : composez-le à nouveau avant de l'enregistrer"
                    .into(),
            ));
        }
        let base: BaseResume = serde_json::from_value(content.clone()).map_err(|_| {
            AppError::Validation(
                "Le contenu du CV est invalide : composez-le à nouveau avant de l'enregistrer"
                    .into(),
            )
        })?;
        validate_document(&base.document)?;
        return Ok(());
    }
    if content.get("schema_version") == Some(&serde_json::json!(RESUME_WORKSPACE_VERSION)) {
        let workspace: ResumeWorkspace = serde_json::from_value(content.clone()).map_err(|_| {
            AppError::Validation(
                "Le contenu du CV est invalide : générez-le à nouveau avant de l'enregistrer"
                    .into(),
            )
        })?;
        validate_document(&workspace.document)?;
    }
    Ok(())
}

/// Longueur maximale de la mention d'une version, en caractères.
const MAX_VERSION_NOTE: usize = 120;

/// Mention d'une version, rognée ; vide, elle est absente.
fn version_note(note: Option<&str>) -> AppResult<Option<String>> {
    let Some(note) = note.map(str::trim).filter(|note| !note.is_empty()) else {
        return Ok(None);
    };
    if note.chars().count() > MAX_VERSION_NOTE {
        return Err(AppError::Validation(format!(
            "La mention d'une version tient en {MAX_VERSION_NOTE} caractères."
        )));
    }
    Ok(Some(note.to_owned()))
}

/// Refuse une valeur hors du jeu fermé accepté par le rendu.
fn valider_valeur(value: &str, acceptes: &[&str], label: &str) -> AppResult<()> {
    if acceptes.contains(&value) {
        Ok(())
    } else {
        Err(AppError::Validation(format!(
            "{label} n'est pas pris en charge."
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_mention_d_une_version_est_rognee_bornee_et_facultative() {
        assert_eq!(
            version_note(Some("  Relecture manuelle "))
                .unwrap()
                .as_deref(),
            Some("Relecture manuelle")
        );
        assert_eq!(version_note(Some("   ")).unwrap(), None);
        assert_eq!(version_note(None).unwrap(), None);
        assert!(matches!(
            version_note(Some(&"x".repeat(121))),
            Err(AppError::Validation(_))
        ));
    }
    use crate::core::database::{open_pool, run_local_migrations};
    use crate::features::documents::infrastructure::{
        SqliteCoverLetterRepository, SqliteResumeRepository,
    };
    use crate::features::profile::application::ProfileService;
    use crate::features::profile::infrastructure::SqliteProfileRepository;
    use std::sync::Arc;

    fn service() -> DocumentsService<
        SqliteResumeRepository,
        SqliteCoverLetterRepository,
        SqliteProfileRepository,
    > {
        let pool = open_pool(None).unwrap();
        run_local_migrations(&pool).unwrap();
        let profile = Arc::new(ProfileService::new(
            SqliteProfileRepository::new(pool.clone()),
            std::env::temp_dir().join("candilog-photos-documents-tests"),
        ));
        DocumentsService::new(
            SqliteResumeRepository::new(pool.clone()),
            SqliteCoverLetterRepository::new(pool),
            profile,
        )
    }

    #[test]
    fn refuse_un_cv_sans_nom() {
        let err = service()
            .resume_save(&NewResume {
                name: "   ".into(),
                content: serde_json::json!({}),
                ..Default::default()
            })
            .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    /// `content` traverse l'IPC en `serde_json::Value` (`unknown` côté TypeScript) et
    /// atterrissait tel quel en base : ni forme, ni borne. Un appel forgé — ou une
    /// génération inhabituelle — y écrivait un blob arbitraire que la bibliothèque devait
    /// ensuite relire à l'aveugle.
    #[test]
    fn refuse_un_cv_dont_le_contenu_n_est_pas_un_objet() {
        for contenu in [
            serde_json::Value::Null,
            serde_json::json!("texte"),
            serde_json::json!([1, 2, 3]),
        ] {
            let err = service()
                .resume_save(&NewResume {
                    name: "CV Produit".into(),
                    content: contenu.clone(),
                    ..Default::default()
                })
                .unwrap_err();
            assert!(
                matches!(err, AppError::Validation(_)),
                "contenu {contenu} accepté"
            );
        }
    }

    /// Un contenu forgé en v1 (`schema_version: 1`) est désérialisé en `ResumeWorkspace` et
    /// son document validé : un `document` vide n'a ni identité ni profil, la validation doit
    /// donc refuser l'écriture plutôt que de laisser entrer une forme incohérente en base.
    #[test]
    fn refuse_un_cv_v1_dont_le_document_est_invalide() {
        let service = service();
        let err = service
            .resume_save(&NewResume {
                name: "CV Produit".into(),
                content: serde_json::json!({ "schema_version": 1, "document": {} }),
                ..Default::default()
            })
            .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
        assert_eq!(service.resume_list_page(1, 8, "", false).unwrap().total, 0);
    }

    #[test]
    fn refuse_un_cv_dont_le_contenu_depasse_la_borne() {
        let err = service()
            .resume_save(&NewResume {
                name: "CV Produit".into(),
                content: serde_json::json!({ "resume": "x".repeat(MAX_CONTENT_CHARS) }),
                ..Default::default()
            })
            .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    /// Le rendu de lettre refuse déjà tout ton hors `formal|casual|creative` et toute
    /// longueur hors `short|medium|long`. La persistance, elle, acceptait n'importe quelle
    /// chaîne : la même règle avait deux comportements selon la couche traversée.
    #[test]
    fn refuse_une_lettre_au_ton_ou_a_la_longueur_inconnus() {
        let err = service()
            .cover_letter_save(&NewCoverLetter {
                name: "Lettre Nova".into(),
                company: None,
                job_title: None,
                recipient: None,
                recipient_address: None,
                job_reference: None,
                tone: "sarcastique".into(),
                length: "medium".into(),
                content: "Madame, Monsieur…".into(),
                ..Default::default()
            })
            .unwrap_err();
        assert!(
            matches!(err, AppError::Validation(_)),
            "ton inconnu accepté"
        );

        let err = service()
            .cover_letter_save(&NewCoverLetter {
                name: "Lettre Nova".into(),
                company: None,
                job_title: None,
                recipient: None,
                recipient_address: None,
                job_reference: None,
                tone: "formal".into(),
                length: "interminable".into(),
                content: "Madame, Monsieur…".into(),
                ..Default::default()
            })
            .unwrap_err();
        assert!(
            matches!(err, AppError::Validation(_)),
            "longueur inconnue acceptée"
        );
    }

    #[test]
    fn refuse_une_lettre_sans_contenu() {
        let err = service()
            .cover_letter_save(&NewCoverLetter {
                name: "Lettre Nova".into(),
                company: Some("Nova".into()),
                job_title: Some("Designer".into()),
                recipient: None,
                recipient_address: None,
                job_reference: None,
                tone: "formal".into(),
                length: "medium".into(),
                content: "  ".into(),
                ..Default::default()
            })
            .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    use crate::features::documents::domain::ResumeIdentity;

    fn document_minimal() -> ResumeDocument {
        ResumeDocument {
            identity: ResumeIdentity {
                full_name: "Alex Martin".into(),
                email: "alex@example.com".into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    #[test]
    fn un_contenu_de_cv_de_base_valide_est_accepte() {
        let content = serde_json::json!({
            "schema_version": 1,
            "kind": "base",
            "document": serde_json::to_value(document_minimal()).unwrap(),
        });
        assert!(valider_contenu(&content).is_ok());
    }

    /// Un contenu `kind: "base"` incomplet était accepté comme contenu historique avant la
    /// branche de validation : il doit désormais être refusé.
    #[test]
    fn un_cv_de_base_incomplet_n_est_plus_accepte_comme_contenu_historique() {
        let content = serde_json::json!({ "kind": "base" });
        assert!(matches!(
            valider_contenu(&content),
            Err(AppError::Validation(_))
        ));
    }

    /// Les sections écartées voyagent avec le contenu : sans elles, rouvrir un CV de base
    /// rallumerait des interrupteurs éteints et ressusciterait les sections retirées.
    #[test]
    fn un_cv_de_base_conserve_les_sections_ecartees() {
        let content = serde_json::json!({
            "schema_version": 1,
            "kind": "base",
            "document": serde_json::to_value(document_minimal()).unwrap(),
            "excluded_sections": ["skills", "summary"],
        });
        assert!(valider_contenu(&content).is_ok());
        let base: BaseResume = serde_json::from_value(content).unwrap();
        assert_eq!(
            base.excluded_sections,
            vec![ProfileSection::Skills, ProfileSection::Summary]
        );
    }

    /// Les contenus enregistrés avant ce champ restent lisibles sans migration.
    #[test]
    fn un_cv_de_base_sans_sections_ecartees_reste_lisible() {
        let content = serde_json::json!({
            "schema_version": 1,
            "kind": "base",
            "document": serde_json::to_value(document_minimal()).unwrap(),
        });
        let base: BaseResume = serde_json::from_value(content).unwrap();
        assert!(base.excluded_sections.is_empty());
    }

    /// Un `kind: "base"` forgé avec une autre version entrait en base, puis la bibliothèque
    /// le déclarait illisible : la branche `kind` doit exiger la version qu'elle sait relire.
    #[test]
    fn un_cv_de_base_dans_une_version_inconnue_est_refuse() {
        let content = serde_json::json!({
            "schema_version": 2,
            "kind": "base",
            "document": serde_json::to_value(document_minimal()).unwrap(),
        });
        assert!(matches!(
            valider_contenu(&content),
            Err(AppError::Validation(_))
        ));
    }

    #[test]
    fn un_contenu_historique_sans_kind_reste_accepte() {
        let content = serde_json::json!({ "titre": "ancien CV" });
        assert!(valider_contenu(&content).is_ok());
    }
}
