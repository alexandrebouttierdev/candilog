//! Validation des CV et lettres avant persistance.

use crate::core::errors::{AppError, AppResult};
use crate::features::documents::domain::{
    CvRepository, CvResume, CvVersion, Lettre, LettreRepository, NouveauCv, NouvelleLettre,
};
use uuid::Uuid;

pub struct DocumentsService<C: CvRepository, L: LettreRepository> {
    cv: C,
    lettres: L,
}

impl<C: CvRepository, L: LettreRepository> DocumentsService<C, L> {
    #[must_use]
    pub const fn new(cv: C, lettres: L) -> Self {
        Self { cv, lettres }
    }

    pub fn cv_enregistrer(&self, input: &NouveauCv) -> AppResult<CvVersion> {
        let nom = input.nom.trim();
        if nom.is_empty() {
            return Err(AppError::Validation(
                "Le nom de la version est requis".into(),
            ));
        }
        if nom.chars().count() > 120 {
            return Err(AppError::Validation(
                "Le nom de la version est trop long (120 max)".into(),
            ));
        }
        self.cv.enregistrer(&NouveauCv {
            nom: nom.into(),
            contenu: input.contenu.clone(),
        })
    }

    pub fn cv_lister(&self) -> AppResult<Vec<CvResume>> {
        self.cv.lister()
    }
    pub fn cv_obtenir(&self, id: Uuid) -> AppResult<CvVersion> {
        self.cv.obtenir(id)
    }
    pub fn cv_supprimer(&self, id: Uuid) -> AppResult<()> {
        self.cv.supprimer(id)
    }

    pub fn lettre_enregistrer(&self, input: &NouvelleLettre) -> AppResult<Lettre> {
        let nom = input.nom.trim();
        if nom.is_empty() {
            return Err(AppError::Validation(
                "Le nom de la lettre est requis".into(),
            ));
        }
        if nom.chars().count() > 140 {
            return Err(AppError::Validation(
                "Le nom de la lettre est trop long".into(),
            ));
        }
        if input.contenu.trim().is_empty() {
            return Err(AppError::Validation(
                "Générez une lettre avant de l'enregistrer".into(),
            ));
        }
        let mut nettoyee = input.clone();
        nettoyee.nom = nom.into();
        self.lettres.enregistrer(&nettoyee)
    }

    pub fn lettres_lister(&self) -> AppResult<Vec<Lettre>> {
        self.lettres.lister()
    }
    pub fn lettre_obtenir(&self, id: Uuid) -> AppResult<Lettre> {
        self.lettres.obtenir(id)
    }
    pub fn lettre_supprimer(&self, id: Uuid) -> AppResult<()> {
        self.lettres.supprimer(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::database::{open_pool, run_local_migrations};
    use crate::features::documents::infrastructure::{SqliteCvRepository, SqliteLettreRepository};

    fn service() -> DocumentsService<SqliteCvRepository, SqliteLettreRepository> {
        let pool = open_pool(None).unwrap();
        run_local_migrations(&pool).unwrap();
        DocumentsService::new(
            SqliteCvRepository::new(pool.clone()),
            SqliteLettreRepository::new(pool),
        )
    }

    #[test]
    fn refuse_un_cv_sans_nom() {
        let err = service()
            .cv_enregistrer(&NouveauCv {
                nom: "   ".into(),
                contenu: serde_json::json!({}),
            })
            .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn refuse_une_lettre_sans_contenu() {
        let err = service()
            .lettre_enregistrer(&NouvelleLettre {
                nom: "Lettre Nova".into(),
                entreprise: Some("Nova".into()),
                poste: Some("Designer".into()),
                ton: "formal".into(),
                longueur: "medium".into(),
                contenu: "  ".into(),
            })
            .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }
}
