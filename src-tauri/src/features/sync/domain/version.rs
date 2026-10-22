//! Vecteurs de versions : l'ordre causal entre les écritures de plusieurs ordinateurs.
//!
//! Chaque installation numérote ses lots publiés (1, 2, 3…). Une version d'enregistrement —
//! ou d'une colonne — est l'ensemble des derniers lots de chaque installation qu'elle a
//! « vus ». Comparer deux vecteurs dit si l'une des écritures connaissait l'autre ou si
//! elles ont eu lieu en parallèle : c'est la seule information fiable pour arbitrer, les
//! horloges des ordinateurs pouvant diverger de plusieurs minutes, voire de plusieurs jours.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Relation causale entre deux versions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Causality {
    /// Mêmes lots vus des deux côtés.
    Equal,
    /// La version de gauche a vu tout ce qu'a vu celle de droite, et davantage.
    After,
    /// La version de gauche est antérieure à celle de droite.
    Before,
    /// Écritures parallèles : aucune ne connaissait l'autre.
    Concurrent,
}

/// Vecteur de versions : installation → numéro du dernier lot vu.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct VersionVector(BTreeMap<String, u64>);

impl VersionVector {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Vecteur d'une seule écriture.
    #[must_use]
    pub fn single(installation: &str, seq: u64) -> Self {
        let mut vector = Self::new();
        vector.0.insert(installation.to_owned(), seq);
        vector
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[must_use]
    pub fn get(&self, installation: &str) -> u64 {
        self.0.get(installation).copied().unwrap_or(0)
    }

    pub fn entries(&self) -> impl Iterator<Item = (&String, &u64)> {
        self.0.iter()
    }

    /// Retient un lot vu, sans jamais faire reculer un compteur.
    pub fn observe(&mut self, installation: &str, seq: u64) {
        let entry = self.0.entry(installation.to_owned()).or_insert(0);
        if seq > *entry {
            *entry = seq;
        }
    }

    /// Union des connaissances des deux vecteurs (maximum point à point).
    pub fn merge(&mut self, other: &Self) {
        for (installation, seq) in &other.0 {
            self.observe(installation, *seq);
        }
    }

    #[must_use]
    pub fn merged(&self, other: &Self) -> Self {
        let mut result = self.clone();
        result.merge(other);
        result
    }

    /// `self ≤ other` : tout ce que `self` a vu, `other` l'a vu aussi.
    #[must_use]
    pub fn is_covered_by(&self, other: &Self) -> bool {
        self.0
            .iter()
            .all(|(installation, seq)| *seq <= other.get(installation))
    }

    #[must_use]
    pub fn compare(&self, other: &Self) -> Causality {
        match (self.is_covered_by(other), other.is_covered_by(self)) {
            (true, true) => Causality::Equal,
            (true, false) => Causality::Before,
            (false, true) => Causality::After,
            (false, false) => Causality::Concurrent,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vv(entries: &[(&str, u64)]) -> VersionVector {
        let mut vector = VersionVector::new();
        for (installation, seq) in entries {
            vector.observe(installation, *seq);
        }
        vector
    }

    #[test]
    fn deux_vecteurs_identiques_sont_egaux() {
        assert_eq!(vv(&[("a", 2)]).compare(&vv(&[("a", 2)])), Causality::Equal);
        assert_eq!(VersionVector::new().compare(&vv(&[])), Causality::Equal);
    }

    #[test]
    fn une_version_qui_a_tout_vu_est_posterieure() {
        assert_eq!(
            vv(&[("a", 2), ("b", 1)]).compare(&vv(&[("a", 2)])),
            Causality::After
        );
        assert_eq!(vv(&[("a", 1)]).compare(&vv(&[("a", 3)])), Causality::Before);
    }

    #[test]
    fn deux_ecritures_paralleles_sont_concurrentes() {
        assert_eq!(
            vv(&[("a", 2), ("b", 1)]).compare(&vv(&[("a", 1), ("b", 2)])),
            Causality::Concurrent
        );
    }

    #[test]
    fn la_fusion_prend_le_maximum_sans_reculer() {
        let mut left = vv(&[("a", 3), ("b", 1)]);
        left.merge(&vv(&[("a", 1), ("b", 4), ("c", 2)]));
        assert_eq!(left, vv(&[("a", 3), ("b", 4), ("c", 2)]));
        left.observe("a", 2);
        assert_eq!(left.get("a"), 3);
    }

    #[test]
    fn serialisation_compacte_en_objet() {
        let text = serde_json::to_string(&vv(&[("a", 3)])).unwrap();
        assert_eq!(text, r#"{"a":3}"#);
        let back: VersionVector = serde_json::from_str(&text).unwrap();
        assert_eq!(back, vv(&[("a", 3)]));
    }
}
