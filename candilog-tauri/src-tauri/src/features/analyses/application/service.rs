//! Composition des agrégats du tableau de bord et de l'écran Analyses.

use crate::core::errors::{AppError, AppResult};
use crate::features::analyses::domain::{
    Analyses, AnalysesRepository, Etape, Periode, TableauDeBord,
};
use chrono::NaiveDate;

/// Service d'analyses, générique sur le dépôt pour rester testable sans `SQLite`.
pub struct AnalysesService<R: AnalysesRepository> {
    repo: R,
}

impl<R: AnalysesRepository> AnalysesService<R> {
    /// Construit le service autour du dépôt d'agrégats.
    #[must_use]
    pub const fn new(repo: R) -> Self {
        Self { repo }
    }

    /// Charge tous les blocs du tableau de bord en un appel IPC.
    ///
    /// # Errors
    /// Propage l'erreur d'un des agrégats du dépôt.
    pub fn tableau_de_bord(&self, aujourdhui: NaiveDate) -> AppResult<TableauDeBord> {
        let depuis = (aujourdhui - chrono::Duration::days(30))
            .format("%Y-%m-%d")
            .to_string();
        let jour = aujourdhui.format("%Y-%m-%d").to_string();
        Ok(TableauDeBord {
            indicateurs: self.repo.indicateurs(Some(&depuis))?,
            performance: self.repo.performance(Some(&depuis))?,
            echeances: self.repo.echeances(&jour, 6)?,
            pipeline: self.repo.pipeline()?,
            activite: self.repo.activite_hebdomadaire(8)?,
            recentes: self.repo.recentes(6)?,
        })
    }

    /// Charge les analyses pour la période choisie.
    ///
    /// # Errors
    /// Propage l'erreur d'un des agrégats du dépôt.
    pub fn analyses(&self, periode: Periode, aujourdhui: NaiveDate) -> AppResult<Analyses> {
        let depuis = periode.depuis(aujourdhui);
        let indicateurs = self.repo.indicateurs(depuis.as_deref())?;
        let total = indicateurs.candidatures;
        let entonnoir = [
            ("Envoyées", indicateurs.candidatures),
            ("Réponses", indicateurs.reponses),
            ("Entretiens", indicateurs.entretiens),
            ("Refus", indicateurs.refus),
        ]
        .into_iter()
        .map(|(label, nombre)| Etape {
            label: label.into(),
            nombre,
            pourcentage: pourcentage(nombre, total),
        })
        .collect();
        let jour = aujourdhui.format("%Y-%m-%d").to_string();

        Ok(Analyses {
            indicateurs,
            performance: self.repo.performance(depuis.as_deref())?,
            activite: self.repo.activite_hebdomadaire(periode.semaines())?,
            entonnoir,
            a_relancer: self.repo.a_relancer(&jour, 7, 5)?,
        })
    }

    /// Produit un export CSV lisible dans un tableur pour la période choisie.
    ///
    /// # Errors
    /// Propage l'erreur de chargement ou une erreur de sérialisation CSV.
    pub fn exporter_csv(&self, periode: Periode, aujourdhui: NaiveDate) -> AppResult<Vec<u8>> {
        let donnees = self.analyses(periode, aujourdhui)?;
        let mut writer = csv::WriterBuilder::new()
            .delimiter(b';')
            .from_writer(vec![]);
        writer
            .write_record(["Indicateur", "Valeur"])
            .map_err(|error| AppError::Serialization(error.to_string()))?;
        for (label, valeur) in [
            ("Candidatures", donnees.indicateurs.candidatures.to_string()),
            ("Entretiens", donnees.indicateurs.entretiens.to_string()),
            ("Réponses", donnees.indicateurs.reponses.to_string()),
            ("Refus", donnees.indicateurs.refus.to_string()),
            (
                "Taux de réponse",
                format!("{} %", donnees.indicateurs.taux_reponse),
            ),
            (
                "Taux d'entretien",
                format!("{} %", donnees.indicateurs.taux_entretien),
            ),
        ] {
            writer
                .write_record([label, &valeur])
                .map_err(|error| AppError::Serialization(error.to_string()))?;
        }
        writer
            .write_record(["", ""])
            .map_err(|error| AppError::Serialization(error.to_string()))?;
        writer
            .write_record(["Semaine du", "Candidatures envoyées"])
            .map_err(|error| AppError::Serialization(error.to_string()))?;
        for semaine in donnees.activite {
            writer
                .write_record([semaine.debut, semaine.nombre.to_string()])
                .map_err(|error| AppError::Serialization(error.to_string()))?;
        }
        writer
            .into_inner()
            .map_err(|error| AppError::Serialization(error.to_string()))
    }
}

fn pourcentage(part: u64, total: u64) -> u8 {
    if total == 0 {
        return 0;
    }
    ((part.min(total) as f64 / total as f64) * 100.0).round() as u8
}
