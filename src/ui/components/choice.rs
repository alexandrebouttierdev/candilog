//! Adaptation d'une entité métier en option de sélecteur.
//!
//! Un `PickList` a besoin d'options comparables et affichables ; les modèles
//! métier n'ont pas à porter cette contrainte.

/// Option d'un sélecteur, adossée à l'identifiant de l'entité choisie.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Choice {
    /// Identifiant de l'entité, ou `nil` pour l'option « toutes ».
    pub id: uuid::Uuid,
    /// Libellé affiché dans le sélecteur.
    pub label: String,
}

impl Choice {
    /// Option correspondant à une entité identifiée.
    #[must_use]
    pub fn new(id: uuid::Uuid, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
        }
    }

    /// Option d'échappement, sans entité associée.
    #[must_use]
    pub fn any(label: impl Into<String>) -> Self {
        Self::new(uuid::Uuid::nil(), label)
    }

    /// Identifiant réel, `None` pour l'option d'échappement.
    #[must_use]
    pub fn value(&self) -> Option<uuid::Uuid> {
        (!self.id.is_nil()).then_some(self.id)
    }

    /// Retrouve l'option correspondant à un identifiant sélectionné.
    #[must_use]
    pub fn find(choices: &[Self], selected: Option<uuid::Uuid>) -> Option<Self> {
        match selected {
            Some(id) => choices.iter().find(|choice| choice.id == id).cloned(),
            None => choices.first().filter(|choice| choice.id.is_nil()).cloned(),
        }
    }
}

impl std::fmt::Display for Choice {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.label)
    }
}

#[cfg(test)]
mod tests {
    use super::Choice;

    #[test]
    fn une_option_d_echappement_ne_porte_pas_d_identifiant() {
        let choice = Choice::any("Toutes les entreprises");
        assert!(choice.id.is_nil());
        assert_eq!(choice.value(), None);
    }

    #[test]
    fn une_option_d_entite_expose_son_identifiant() {
        let id = uuid::Uuid::new_v4();
        let choice = Choice::new(id, "Kaphisto");
        assert_eq!(choice.value(), Some(id));
        assert_eq!(choice.to_string(), "Kaphisto");
    }

    #[test]
    fn la_selection_retrouve_l_option_correspondante() {
        let first = uuid::Uuid::new_v4();
        let second = uuid::Uuid::new_v4();
        let choices = vec![
            Choice::any("Toutes"),
            Choice::new(first, "Agrial"),
            Choice::new(second, "Conty"),
        ];
        assert_eq!(
            Choice::find(&choices, Some(second)),
            Some(Choice::new(second, "Conty"))
        );
    }

    #[test]
    fn sans_selection_l_option_d_echappement_est_retenue() {
        let choices = vec![
            Choice::any("Toutes"),
            Choice::new(uuid::Uuid::new_v4(), "Agrial"),
        ];
        assert_eq!(Choice::find(&choices, None), Some(Choice::any("Toutes")));
    }

    #[test]
    fn sans_option_d_echappement_l_absence_de_selection_reste_vide() {
        let choices = vec![Choice::new(uuid::Uuid::new_v4(), "Agrial")];
        assert_eq!(Choice::find(&choices, None), None);
    }

    #[test]
    fn un_identifiant_inconnu_ne_produit_aucune_option() {
        let choices = vec![Choice::new(uuid::Uuid::new_v4(), "Agrial")];
        assert_eq!(Choice::find(&choices, Some(uuid::Uuid::new_v4())), None);
    }
}
