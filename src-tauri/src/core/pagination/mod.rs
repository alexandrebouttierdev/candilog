//! Pagination partagée par toutes les features.
//!
//! Placée dans `core` et non dans une feature : le contrat de page est le même pour les
//! candidatures, les entreprises, les contacts et l'historique des scores, et le rattacher
//! à l'une d'elles ferait dépendre les autres d'un domaine qui ne les concerne pas.

use serde::Serialize;

/// Page de résultats et ses métadonnées.
#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "page.ts")]
pub struct Page<T: ts_rs::TS> {
    /// Éléments de la page courante.
    pub items: Vec<T>,
    /// Count total d'éléments, toutes pages confondues.
    ///
    /// Les quatre compteurs sont annoncés `number` et non `number | bigint` côté
    /// TypeScript, contrairement à ce que `ts-rs` déduit d'un `u64` : ils comptent des
    /// lignes d'une base `SQLite` locale de suivi de candidatures, qui ne peut pas
    /// approcher les 2^53 au-delà desquels un entier JavaScript perdrait en précision.
    /// Laisser l'union obligerait chaque affichage de compteur à traiter un cas de `bigint`
    /// qui ne se produira jamais.
    #[ts(type = "number")]
    pub total: u64,
    /// Numéro de page courant, à partir de 1.
    #[ts(type = "number")]
    pub page: u64,
    /// Count maximal d'éléments par page.
    #[ts(type = "number")]
    pub page_size: u64,
    /// Count total de pages, au moins 1 même sur une collection vide.
    #[ts(type = "number")]
    pub total_pages: u64,
}

impl<T: ts_rs::TS> Page<T> {
    /// Construit les métadonnées d'une page à partir de son total.
    ///
    /// `page_size` est ramené à 1 au minimum : une taille nulle rendrait `total_pages`
    /// indéfini et ferait diviser par zéro le calcul de la tranche.
    #[must_use]
    pub fn new(items: Vec<T>, total: u64, page: u64, page_size: u64) -> Self {
        let page_size = page_size.max(1);
        Self {
            items,
            total,
            page,
            page_size,
            total_pages: total.div_ceil(page_size).max(1),
        }
    }

    /// Décalage `SQL OFFSET` correspondant à une page, sans débordement.
    #[must_use]
    pub fn offset(page: u64, page_size: u64) -> u64 {
        page.saturating_sub(1).saturating_mul(page_size.max(1))
    }
}

#[cfg(test)]
#[path = "tests/page/mod.rs"]
mod tests;
