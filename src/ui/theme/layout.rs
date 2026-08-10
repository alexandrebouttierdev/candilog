//! Règles d'adaptation à la taille de fenêtre.
//!
//! Il ne s'agit pas d'un passage en interface mobile : rien ne devient une
//! colonne unique pleine largeur, aucune cible n'est agrandie. On décide
//! seulement du nombre de colonnes et des colonnes de table qu'on peut se
//! permettre d'afficher.

use iced::Size;

/// Largeur minimale imposée à la fenêtre par `src/main.rs`.
pub const MIN_WIDTH: f32 = 1040.0;
/// Hauteur minimale imposée à la fenêtre par `src/main.rs`.
pub const MIN_HEIGHT: f32 = 660.0;
/// Largeur au-delà de laquelle le contenu cesse de s'étirer et gagne des marges.
pub const CONTENT_MAX: f32 = 1560.0;

/// Largeur à partir de laquelle les tables montrent leurs colonnes secondaires.
const TABLE_SECONDARY: f32 = 1280.0;
/// Largeur à partir de laquelle le tableau de bord se met sur deux colonnes.
const DASHBOARD_TWO_COLUMNS: f32 = 1320.0;
/// Largeur à partir de laquelle les actions de toolbar affichent leur
/// libellé à côté de l'icône. En dessous, il n'y a plus la place pour le
/// texte sans le faire retomber à la ligne : l'action se replie sur son
/// icône seule.
const TOOLBAR_ACTION_LABELS: f32 = 1200.0;

/// Décisions de mise en page dérivées de la taille de la fenêtre.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Layout {
    /// Largeur utile, jamais inférieure à [`MIN_WIDTH`].
    pub width: f32,
    /// Hauteur utile, jamais inférieure à [`MIN_HEIGHT`].
    pub height: f32,
}

impl Layout {
    /// Construit les décisions de mise en page depuis la taille de la fenêtre.
    #[must_use]
    pub fn from_size(size: Size) -> Self {
        Self {
            width: size.width.max(MIN_WIDTH),
            height: size.height.max(MIN_HEIGHT),
        }
    }

    /// Colonnes secondaires d'une table affichables. En dessous, leur contenu
    /// revient dans la colonne principale plutôt que de disparaître.
    #[must_use]
    pub fn table_secondary_columns(&self) -> bool {
        self.width >= TABLE_SECONDARY
    }

    /// Tableau de bord sur deux colonnes.
    #[must_use]
    pub fn dashboard_two_columns(&self) -> bool {
        self.width >= DASHBOARD_TWO_COLUMNS
    }

    /// Actions de toolbar avec leur libellé affiché à côté de l'icône. En
    /// dessous, une action iconique se replie sur son icône seule, son
    /// intitulé restitué en infobulle au survol.
    #[must_use]
    pub fn toolbar_action_labels(&self) -> bool {
        self.width >= TOOLBAR_ACTION_LABELS
    }

    /// Largeur maximale du contenu ; au-delà, il gagne des marges latérales.
    #[must_use]
    pub fn content_max_width(&self) -> f32 {
        CONTENT_MAX
    }
}

impl Default for Layout {
    fn default() -> Self {
        Self::from_size(Size::new(MIN_WIDTH, MIN_HEIGHT))
    }
}

#[cfg(test)]
mod tests {
    use super::{Layout, MIN_HEIGHT, MIN_WIDTH};
    use iced::Size;

    #[test]
    fn la_taille_est_bornee_par_le_minimum_de_la_fenetre() {
        let layout = Layout::from_size(Size::new(600.0, 400.0));
        assert!((layout.width - MIN_WIDTH).abs() < f32::EPSILON);
        assert!((layout.height - MIN_HEIGHT).abs() < f32::EPSILON);
    }

    #[test]
    fn les_seuils_sont_ordonnes_du_plus_bas_au_plus_haut() {
        let large = Layout::from_size(Size::new(1600.0, 900.0));
        assert!(large.table_secondary_columns());
        assert!(large.dashboard_two_columns());
        assert!(large.toolbar_action_labels());

        let minimal = Layout::from_size(Size::new(MIN_WIDTH, MIN_HEIGHT));
        assert!(!minimal.table_secondary_columns());
        assert!(!minimal.dashboard_two_columns());
        assert!(!minimal.toolbar_action_labels());
    }

    #[test]
    fn les_libelles_d_action_de_toolbar_disparaissent_sous_leur_seuil() {
        assert!(!Layout::from_size(Size::new(1100.0, 800.0)).toolbar_action_labels());
        assert!(Layout::from_size(Size::new(1300.0, 800.0)).toolbar_action_labels());
    }

    #[test]
    fn la_largeur_de_contenu_est_plafonnee() {
        let tres_large = Layout::from_size(Size::new(2400.0, 1200.0));
        assert!((tres_large.content_max_width() - super::CONTENT_MAX).abs() < f32::EPSILON);
    }
}
