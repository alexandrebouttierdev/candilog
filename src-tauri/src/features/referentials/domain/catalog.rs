//! Entités des quatre référentiels métier.

use serde::{Deserialize, Serialize};

/// Secteur d'activité **de l'entreprise**.
///
/// Seul référentiel dont la clé est un UUID : la table existe depuis l'origine et ses
/// libellés n'ont pas de code métier public auquel se rattacher.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "referentials.ts")]
pub struct ActivitySector {
    /// Id du secteur.
    pub id: uuid::Uuid,
    /// Libellé français affiché dans les sélecteurs.
    pub name: String,
}

/// Entrée d'un référentiel identifié par son code métier.
///
/// Le code est la valeur persistée (`M18`, `MIS`, `IT_SERVICES_COMPANY`), le libellé est ce
/// que l'interface affiche. Générer un UUID pour une valeur déjà identifiante n'ajouterait
/// qu'une indirection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "referentials.ts")]
pub struct ReferenceItem {
    /// Code persisté en base.
    pub code: String,
    /// Libellé français affiché dans les sélecteurs.
    pub name: String,
}

/// Les quatre catalogues, renvoyés en un seul aller-retour IPC.
///
/// Groupés et non exposés par une commande chacun : les formulaires et les filtres en ont
/// besoin ensemble, et quatre requêtes séparées multiplieraient les états de chargement
/// pour des listes figées de quelques dizaines de lignes.
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "referentials.ts")]
pub struct Referentials {
    /// Secteurs d'activité des entreprises.
    pub sectors: Vec<ActivitySector>,
    /// Domaines professionnels des postes.
    pub professional_domains: Vec<ReferenceItem>,
    /// Natures d'organisation employeuse.
    pub company_types: Vec<ReferenceItem>,
    /// Types de contrat.
    pub contract_types: Vec<ReferenceItem>,
}
