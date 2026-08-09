//! Espacements, rayons et dimensions de l'interface desktop.

/// Échelle d'espacement, en pixels.
pub mod space {
    /// Séparation minimale entre deux glyphes liés.
    pub const XXS: f32 = 2.0;
    /// Écart interne d'un groupe très serré.
    pub const XS: f32 = 4.0;
    /// Gouttière entre une icône et son libellé, entre contrôles d'un groupe.
    pub const SM: f32 = 6.0;
    /// Gouttière courante d'une ligne composée.
    pub const MD: f32 = 8.0;
    /// Écart entre deux blocs proches.
    pub const LG: f32 = 10.0;
    /// Padding intérieur d'un panneau, gouttière entre groupes de toolbar.
    pub const XL: f32 = 12.0;
    /// Padding d'un dialogue, respiration d'une section importante.
    pub const XXL: f32 = 16.0;
    /// Marge maximale autorisée, réservée aux plans de travail de document.
    pub const MAX: f32 = 24.0;
}

/// Rayons par rôle de surface.
pub mod radius {
    /// Zones de données : lignes, cellules, en-têtes.
    pub const NONE: f32 = 0.0;
    /// Page d'un aperçu de document.
    pub const DOCUMENT: f32 = 2.0;
    /// Boutons, champs, selects, segments.
    pub const CONTROL: f32 = 5.0;
    /// Panneaux et cartes réellement autonomes.
    pub const PANEL: f32 = 8.0;
    /// Modales, drawers, menus.
    pub const DIALOG: f32 = 10.0;
    /// Jetons de statut, compteurs, points.
    pub const PILL: f32 = 999.0;
}

/// Hauteurs et largeurs de référence.
pub mod size {
    /// Toolbar d'écran.
    pub const TOOLBAR: f32 = 44.0;
    /// Barre d'état de bas de fenêtre.
    pub const STATUS_BAR: f32 = 22.0;
    /// En-tête de section à l'intérieur d'un panneau.
    pub const SECTION_HEADER: f32 = 28.0;
    /// En-tête de colonnes d'une table.
    pub const TABLE_HEADER: f32 = 26.0;
    /// Ligne de données à une seule ligne de texte.
    pub const ROW: f32 = 32.0;
    /// Ligne de données portant un titre et une métadonnée.
    pub const ROW_COMFORTABLE: f32 = 40.0;
    /// Hauteur commune des contrôles.
    pub const CONTROL: f32 = 26.0;
    /// Côté d'un bouton purement iconique.
    pub const ICON_BUTTON: f32 = 26.0;
    /// Entrée de navigation de la barre latérale.
    pub const NAV_ROW: f32 = 27.0;

    /// Largeur de la barre latérale.
    pub const SIDEBAR: f32 = 208.0;
    /// Largeur du volet liste d'un master-detail.
    pub const MASTER: f32 = 300.0;
    /// Largeur minimale du volet liste.
    pub const MASTER_MIN: f32 = 240.0;
    /// Largeur du sommaire des paramètres.
    pub const SUMMARY: f32 = 196.0;
    /// Largeur du drawer d'inspecteur.
    pub const DRAWER: f32 = 420.0;
    /// Largeur du champ de recherche d'une toolbar.
    pub const SEARCH: f32 = 220.0;
    /// Largeur d'une colonne Kanban.
    pub const KANBAN_COLUMN: f32 = 268.0;

    /// Largeur d'une modale de confirmation.
    pub const DIALOG_CONFIRM: f32 = 380.0;
    /// Largeur d'une modale de formulaire.
    pub const DIALOG_FORM: f32 = 560.0;
    /// Largeur d'une modale de formulaire dense.
    pub const DIALOG_WIDE: f32 = 720.0;
}

/// Épaisseurs de filets.
pub mod stroke {
    /// Filet standard.
    pub const HAIRLINE: f32 = 1.0;
    /// Marqueur de sélection posé sur le bord gauche d'une ligne.
    pub const MARKER: f32 = 2.0;
    /// Filet de focus d'un contrôle.
    pub const FOCUS: f32 = 2.0;
}

/// Proportions d'une page A4, utilisées par l'aperçu de document.
pub const A4_RATIO: f32 = std::f32::consts::SQRT_2;

/// Invariants du design system, vérifiés à la compilation.
mod invariants {
    use super::{radius, size, space, stroke, A4_RATIO};

    const _: () = assert!(space::XXS < space::XS, "échelle d'espacement non ordonnée");
    const _: () = assert!(space::XS < space::SM, "échelle d'espacement non ordonnée");
    const _: () = assert!(space::SM < space::MD, "échelle d'espacement non ordonnée");
    const _: () = assert!(space::MD < space::LG, "échelle d'espacement non ordonnée");
    const _: () = assert!(space::LG < space::XL, "échelle d'espacement non ordonnée");
    const _: () = assert!(space::XL < space::XXL, "échelle d'espacement non ordonnée");
    const _: () = assert!(space::XXL < space::MAX, "échelle d'espacement non ordonnée");

    const _: () = assert!(size::ROW <= 32.0, "ligne de données trop haute");
    const _: () = assert!(size::TOOLBAR <= 44.0, "toolbar trop haute");
    const _: () = assert!(size::CONTROL <= 28.0, "contrôle trop haut");
    const _: () = assert!(space::XL <= 12.0, "padding de panneau trop généreux");
    const _: () = assert!(
        size::TABLE_HEADER < size::ROW,
        "en-tête plus haut qu'une ligne"
    );
    const _: () = assert!(size::ROW < size::ROW_COMFORTABLE, "densités inversées");
    const _: () = assert!(size::STATUS_BAR < size::TOOLBAR, "barre d'état trop haute");

    const _: () = assert!(size::ICON_BUTTON >= 26.0, "zone cliquable trop petite");
    const _: () = assert!(size::CONTROL >= 26.0, "zone cliquable trop petite");
    const _: () = assert!(size::NAV_ROW >= 26.0, "zone cliquable trop petite");

    const _: () = assert!(radius::NONE < radius::DOCUMENT, "rayons non différenciés");
    const _: () = assert!(
        radius::DOCUMENT < radius::CONTROL,
        "rayons non différenciés"
    );
    const _: () = assert!(radius::CONTROL < radius::PANEL, "rayons non différenciés");
    const _: () = assert!(radius::PANEL < radius::DIALOG, "rayons non différenciés");
    const _: () = assert!(radius::DIALOG < radius::PILL, "rayons non différenciés");

    const _: () = assert!(
        size::DIALOG_CONFIRM < size::DIALOG_FORM,
        "gabarits de dialogue non ordonnés"
    );
    const _: () = assert!(
        size::DIALOG_FORM < size::DIALOG_WIDE,
        "gabarits de dialogue non ordonnés"
    );
    const _: () = assert!(size::MASTER_MIN < size::MASTER, "volet maître incohérent");
    const _: () = assert!(size::MASTER <= 420.0, "volet maître trop large");
    const _: () = assert!(size::SIDEBAR <= 220.0, "barre latérale trop large");
    const _: () = assert!(size::SEARCH <= 240.0, "recherche trop large");
    const _: () = assert!(size::DRAWER <= 440.0, "drawer trop large");
    const _: () = assert!(
        size::KANBAN_COLUMN >= 240.0,
        "colonne de pipeline illisible"
    );
    const _: () = assert!(size::SECTION_HEADER <= 28.0, "en-tête de section trop haut");

    const _: () = assert!(stroke::HAIRLINE <= 1.0, "filet trop épais");
    const _: () = assert!(stroke::MARKER <= 2.0, "marqueur trop épais");

    const _: () = assert!(
        A4_RATIO > 1.41 && A4_RATIO < 1.42,
        "proportions A4 incorrectes"
    );
}
