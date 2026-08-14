//! Brouillons de formulaires, filtres et choix d'affichage.

use crate::modules::candidatures::model::{StatutCandidature, TypeContrat};
use crate::modules::entretiens::model::TypeEntretien;
use crate::modules::settings::model::AppSettings;

/// État du formulaire entreprise.
#[derive(Debug, Default)]
pub struct EntrepriseForm {
    pub nom: String,
    pub secteur: String,
    pub type_: String,
    pub site_web: String,
    pub ville: String,
    pub adresse: String,
    pub notes: iced::widget::text_editor::Content,
}

/// État du formulaire contact.
#[derive(Debug, Default)]
pub struct ContactForm {
    pub entreprise_id: Option<uuid::Uuid>,
    pub prenom: String,
    pub nom: String,
    pub poste: String,
    pub email: String,
    pub telephone: String,
    pub linkedin: String,
    pub notes: iced::widget::text_editor::Content,
}

/// État du formulaire candidature.
#[derive(Debug, Clone)]
pub struct CandidatureForm {
    pub entreprise_id: Option<uuid::Uuid>,
    pub poste: String,
    pub type_contrat: TypeContrat,
    pub statut: StatutCandidature,
    pub date_envoi: String,
    pub lien_offre: String,
    pub notes: String,
}

impl Default for CandidatureForm {
    fn default() -> Self {
        Self {
            entreprise_id: None,
            poste: String::new(),
            type_contrat: TypeContrat::Cdi,
            statut: StatutCandidature::EnAttente,
            date_envoi: chrono::Local::now().format("%d-%m-%Y").to_string(),
            lien_offre: String::new(),
            notes: String::new(),
        }
    }
}

/// État du formulaire entretien.
#[derive(Debug)]
pub struct EntretienForm {
    pub candidature_id: Option<uuid::Uuid>,
    pub contact_id: Option<uuid::Uuid>,
    pub date_entretien: String,
    pub type_entretien: TypeEntretien,
    pub lieu: String,
    pub notes: iced::widget::text_editor::Content,
    pub compte_rendu: iced::widget::text_editor::Content,
}

impl Default for EntretienForm {
    fn default() -> Self {
        Self {
            candidature_id: None,
            contact_id: None,
            date_entretien: chrono::Local::now().format("%d-%m-%Y %H:%M").to_string(),
            type_entretien: TypeEntretien::Presentiel,
            lieu: String::new(),
            notes: iced::widget::text_editor::Content::new(),
            compte_rendu: iced::widget::text_editor::Content::new(),
        }
    }
}

/// État du formulaire relance.
#[derive(Debug, Clone)]
pub struct RelanceForm {
    pub candidature_id: Option<uuid::Uuid>,
    pub date_relance: String,
    pub type_relance: String,
    pub notes: String,
}

/// Formulaire de l'écran Paramètres, **distinct** de l'instantané de données.
///
/// Les huit messages d'édition écrivaient directement dans `app.data.settings`, c'est-à-dire
/// dans la copie censée refléter le contenu de la base, alors que les cinq autres formulaires
/// du projet disposent tous d'une structure d'édition séparée. Trois conséquences : un échec
/// d'enregistrement laissait `app.data.settings` divergé de la base sans qu'aucun rechargement
/// ne vienne le corriger ; quitter l'écran sans enregistrer conservait les modifications en
/// mémoire ; et le bandeau de titre annonçait aussitôt un fournisseur qui n'était pas celui
/// réellement persisté.
///
/// La clé API n'y transite que le temps de la saisie — le champ de l'instantané porte la
/// mention « sans secret en clair », qu'y déposer la clé contredisait.
#[derive(Debug, Clone, Default)]
pub struct SettingsForm {
    /// Valeurs en cours d'édition.
    pub draft: AppSettings,
}

impl SettingsForm {
    /// Initialise le formulaire à l'ouverture de l'écran, depuis l'état persisté.
    #[must_use]
    pub fn from_settings(settings: &AppSettings) -> Self {
        Self {
            draft: settings.clone(),
        }
    }
}

/// Filtres cumulables des candidatures.
#[derive(Debug, Clone, Default)]
pub struct CandidateFilters {
    pub status: Option<StatutCandidature>,
    pub contract: Option<TypeContrat>,
    pub company_id: Option<uuid::Uuid>,
    pub city: String,
    pub position: String,
    pub date_from: String,
    pub date_to: String,
}

impl CandidateFilters {
    /// Nombre de critères réellement actifs, hors recherche globale.
    #[must_use]
    pub fn active_count(&self) -> usize {
        usize::from(self.status.is_some())
            + usize::from(self.contract.is_some())
            + usize::from(self.company_id.is_some())
            + usize::from(!self.city.trim().is_empty())
            + usize::from(!self.position.trim().is_empty())
            + usize::from(!self.date_from.trim().is_empty())
            + usize::from(!self.date_to.trim().is_empty())
    }
}

/// Colonne de tri de la vue Liste des candidatures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CandidateSort {
    /// Intitulé du poste.
    Poste,
    /// Nom de l'entreprise.
    Entreprise,
    /// Statut de la candidature.
    Statut,
    /// Date d'envoi, ordre par défaut.
    #[default]
    Date,
}

impl CandidateSort {
    /// Colonnes triables, dans l'ordre d'affichage de la table.
    pub const ALL: [Self; 4] = [Self::Poste, Self::Entreprise, Self::Statut, Self::Date];

    /// Colonne correspondant à un index d'en-tête.
    #[must_use]
    pub fn from_column(index: usize) -> Option<Self> {
        Self::ALL.get(index).copied()
    }

    /// Index d'en-tête de la colonne.
    #[must_use]
    pub fn column(self) -> usize {
        Self::ALL
            .iter()
            .position(|value| *value == self)
            .unwrap_or_default()
    }
}

/// Onglet actif de l'écran Statistiques.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StatisticsTab {
    /// Suivi du pipeline et des relances.
    #[default]
    Candidatures,
    /// Scores ATS et appels IA.
    PerformanceCv,
}

/// Décision utilisateur sur une recommandation ATS.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecommendationStatus {
    /// Aucune décision.
    Pending,
    /// Proposition appliquée.
    Accepted,
    /// Proposition refusée.
    Rejected,
}

impl Default for RelanceForm {
    fn default() -> Self {
        Self {
            candidature_id: None,
            date_relance: chrono::Local::now().format("%d-%m-%Y").to_string(),
            type_relance: "Email".into(),
            notes: String::new(),
        }
    }
}
