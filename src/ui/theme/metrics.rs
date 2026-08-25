//! Espacements, rayons et dimensions de l'interface desktop.

/// Échelle d'espacement, en pixels.
pub mod space {
    /// Séparation minimale entre deux glyphes liés.
    pub const XXS: f32 = 2.0;
    /// Écart interne d'un groupe très serré.
    pub const XS: f32 = 4.0;
    /// Gouttière entre une icône et son libellé.
    pub const SM: f32 = 8.0;
    /// Gouttière courante d'une ligne composée.
    pub const MD: f32 = 12.0;
    /// Écart entre deux blocs proches.
    pub const LG: f32 = 14.0;
    /// Padding intérieur d'un panneau, gouttière entre groupes de toolbar.
    pub const XL: f32 = 16.0;
    /// Padding d'un dialogue, respiration d'une section importante.
    pub const XXL: f32 = 20.0;
    /// Padding horizontal d'une page de contenu (`px-7`).
    pub const PAGE: f32 = 28.0;
    /// Padding vertical d'une page de contenu (`py-6`).
    pub const PAGE_Y: f32 = 24.0;
    /// Marge maximale autorisée, réservée aux plans de travail de document.
    pub const MAX: f32 = 32.0;
    /// Padding horizontal d'un plan de travail (workspace) : `XXL` resserré
    /// de 2 px pour équilibrer visuellement avec le padding vertical `XL`.
    pub const WORKSPACE_X: f32 = 14.0;
}

/// Rayons par rôle de surface.
pub mod radius {
    /// Zones de données : lignes, cellules, en-têtes.
    pub const NONE: f32 = 0.0;
    /// Arrondi d'un marqueur de sélection ou d'une barre fine.
    pub const MARKER: f32 = 1.0;
    /// Arrondi du jeton de statut « barré », presque carré : ni un marqueur
    /// de sélection ni une barre, mais un jeton du même ordre de grandeur.
    pub const BARRED: f32 = 1.0;
    /// Page d'un aperçu de document.
    pub const DOCUMENT: f32 = 3.0;
    /// Boutons, segments, pastilles d'icône.
    pub const CONTROL: f32 = 6.0;
    /// Champs de saisie.
    pub const FIELD: f32 = 8.0;
    /// Panneaux et cartes réellement autonomes.
    pub const PANEL: f32 = 10.0;
    /// Cartes de contenu compactes.
    pub const CARD: f32 = 10.0;
    /// Modales, drawers, menus.
    pub const DIALOG: f32 = 14.0;
    /// Jetons de statut, compteurs, points.
    pub const PILL: f32 = 999.0;
}

/// Hauteurs et largeurs de référence.
pub mod size {
    /// Toolbar d'écran.
    pub const TOOLBAR: f32 = 52.0;
    /// Hauteur du séparateur vertical entre deux groupes de toolbar.
    pub const TOOLBAR_SEPARATOR: f32 = 16.0;
    /// Hauteur de la bande secondaire sous la toolbar (jetons de filtres,
    /// sélection, contexte) : un peu plus haute qu'un en-tête de table pour
    /// respirer sous la toolbar.
    pub const TOOLBAR_STRIP: f32 = TABLE_HEADER + 6.0;
    /// Hauteur de la barre supérieure : onglets contextuels, recherche et runtime.
    pub const TOPBAR: f32 = 46.0;
    /// En-tête de section à l'intérieur d'un panneau.
    pub const SECTION_HEADER: f32 = 34.0;
    /// En-tête de colonnes d'une table.
    pub const TABLE_HEADER: f32 = 36.0;
    /// Ligne de données à une seule ligne de texte.
    pub const ROW: f32 = 42.0;
    /// Ligne de données portant un titre et une métadonnée.
    pub const ROW_COMFORTABLE: f32 = 56.0;
    /// Hauteur commune des contrôles.
    pub const CONTROL: f32 = 30.0;
    /// Hauteur d'un bouton d'action avec libellé.
    pub const ACTION: f32 = 30.0;
    /// Hauteur d'un champ de saisie.
    pub const FIELD_CONTROL: f32 = 36.0;
    /// Côté d'un bouton purement iconique.
    pub const ICON_BUTTON: f32 = 32.0;
    /// Hauteur du rail d'un interrupteur (switch) de ligne de réglage.
    pub const SWITCH: f32 = 16.0;

    /// Hauteur d'un jeton de statut (badge) et diamètre d'une pastille
    /// compacte, comme celle du jour courant sur un calendrier.
    pub const TAG: f32 = 18.0;
    /// Hauteur d'un compteur discret, plus fin qu'un jeton de statut.
    pub const COUNTER: f32 = 17.0;
    /// Diamètre du marqueur de forme dessiné devant un jeton de statut.
    pub const STATUS_DOT: f32 = 7.0;

    /// Hauteur du marqueur de sélection posé sur le bord d'une ligne.
    pub const MARKER: f32 = 16.0;
    /// Hauteur du marqueur de sélection d'une ligne de table, en retrait de
    /// la hauteur de ligne pour ne pas la toucher.
    pub const ROW_MARKER: f32 = 36.0;
    /// Taille du chevron affiché à côté du libellé de la colonne triée dans
    /// un en-tête de table.
    pub const TABLE_SORT_ICON: f32 = 11.0;
    /// Hauteur du marqueur de sélection d'une ligne de liste (`list.rs`),
    /// fixe qu'elle porte un titre et une métadonnée ou une seule ligne de
    /// texte, en retrait pour ne jamais toucher les bords.
    pub const LIST_MARKER: f32 = 34.0;
    /// Épaisseur d'une barre de progression inline (opération longue).
    pub const PROGRESS_BAR: f32 = 3.0;
    /// Largeur maximale du toast de notification flottant.
    pub const TOAST: f32 = 460.0;

    /// Largeur du volet liste d'un master-detail.
    pub const MASTER: f32 = 320.0;
    /// Largeur minimale du volet liste.
    pub const MASTER_MIN: f32 = 260.0;
    /// Largeur du drawer d'inspecteur.
    pub const DRAWER: f32 = 460.0;
    /// Largeur du champ de recherche d'une toolbar.
    pub const SEARCH: f32 = 260.0;
    /// Largeur d'une colonne Kanban.
    pub const KANBAN_COLUMN: f32 = 280.0;
    /// Largeur du rail de navigation desktop (maquette « refonte-design »).
    pub const SIDEBAR: f32 = 86.0;

    /// Largeur de la modale standard candilog-desktop (max-w-[34rem]).
    pub const MODAL_WIDTH: f32 = 544.0;

    /// Largeur d'une modale de confirmation.
    pub const DIALOG_CONFIRM: f32 = 420.0;
    /// Largeur d'une modale de formulaire.
    pub const DIALOG_FORM: f32 = 600.0;
    /// Largeur d'une modale de formulaire dense.
    pub const DIALOG_WIDE: f32 = 780.0;
}

/// Élévation des surfaces qui flottent réellement au-dessus du plan de travail.
pub mod elevation {
    /// Décalage vertical de l'ombre d'une surface surélevée (menu, feuille).
    pub const OFFSET: f32 = 4.0;
    /// Étalement de l'ombre d'une surface surélevée (menu, feuille).
    pub const BLUR: f32 = 16.0;
    /// Décalage vertical de l'ombre d'une modale : la plus haute des
    /// surfaces superposées, elle se détache plus franchement qu'un menu.
    pub const OVERLAY_OFFSET: f32 = 18.0;
    /// Étalement de l'ombre d'une modale.
    pub const OVERLAY_BLUR: f32 = 44.0;
    /// Décalage vertical de l'ombre d'un toast : flotte au-dessus du plan de
    /// travail, plus franchement qu'un menu mais sans rivaliser avec une
    /// modale bloquante.
    pub const TOAST_OFFSET: f32 = 10.0;
    /// Étalement de l'ombre d'un toast.
    pub const TOAST_BLUR: f32 = 26.0;
    /// Décalage vertical de l'ombre du panneau de verre.
    pub const GLASS_OFFSET: f32 = 12.0;
    /// Étalement de l'ombre du panneau de verre.
    pub const GLASS_BLUR: f32 = 32.0;
}

/// Épaisseurs de filets.
pub mod stroke {
    /// Filet standard.
    pub const HAIRLINE: f32 = 1.0;
    /// Filet plus marqué qu'un hairline, pour le contour d'un marqueur creux
    /// ou barré : il doit rester lisible à l'échelle d'un petit jeton.
    pub const EMPHASIS: f32 = 1.5;
    /// Marqueur de sélection posé sur le bord gauche d'une ligne.
    pub const MARKER: f32 = 2.0;
    /// Filet de focus d'un contrôle.
    pub const FOCUS: f32 = 2.0;
}

/// Proportions d'une page A4, utilisées par l'aperçu de document.
pub const A4_RATIO: f32 = std::f32::consts::SQRT_2;

#[cfg(test)]
mod tests {
    use super::size;

    #[test]
    fn gabarits_de_la_coquille_sont_definis() {
        assert_eq!(size::SIDEBAR, 86.0);
        assert_eq!(size::TOPBAR, 46.0);
        assert_eq!(size::MODAL_WIDTH, 544.0);
        assert_eq!(size::DRAWER, 460.0);
    }
}

/// Invariants du design system, vérifiés à la compilation.
mod invariants {
    use super::{elevation, radius, size, space, stroke, A4_RATIO};

    // Ordre de l'échelle d'espacement : inchangé.
    const _: () = assert!(space::XXS < space::XS, "échelle d'espacement non ordonnée");
    const _: () = assert!(space::XS < space::SM, "échelle d'espacement non ordonnée");
    const _: () = assert!(space::SM < space::MD, "échelle d'espacement non ordonnée");
    const _: () = assert!(space::MD < space::LG, "échelle d'espacement non ordonnée");
    const _: () = assert!(space::LG < space::XL, "échelle d'espacement non ordonnée");
    const _: () = assert!(space::XL <= space::XXL, "échelle d'espacement non ordonnée");
    const _: () = assert!(space::XXL < space::MAX, "échelle d'espacement non ordonnée");
    const _: () = assert!(
        space::WORKSPACE_X < space::XL,
        "padding horizontal du plan de travail pas assez resserré face à XL"
    );
    const _: () = assert!(
        space::WORKSPACE_X < space::XXL,
        "padding horizontal du plan de travail aussi large que XXL"
    );

    // Densité Confort : ces planchers remplacent les plafonds de la densité
    // dense précédente. Ils protègent contre un resserrement accidentel.
    const _: () = assert!(size::ROW >= 40.0, "densité Confort : ligne trop basse");
    const _: () = assert!(
        size::TOOLBAR >= 48.0,
        "densité Confort : toolbar trop basse"
    );
    const _: () = assert!(size::CONTROL >= 28.0, "densité Confort : contrôle trop bas");
    const _: () = assert!(space::XL >= 16.0, "densité Confort : panneau trop serré");

    const _: () = assert!(
        size::COUNTER < size::TAG,
        "compteur pas assez discret face au badge"
    );
    const _: () = assert!(size::TAG < size::CONTROL, "jeton aussi haut qu'un contrôle");
    const _: () = assert!(
        size::STATUS_DOT <= space::SM,
        "marqueur de statut trop imposant"
    );

    const _: () = assert!(
        size::TABLE_HEADER < size::ROW,
        "en-tête plus haut qu'une ligne"
    );
    const _: () = assert!(size::ROW < size::ROW_COMFORTABLE, "densités inversées");
    const _: () = assert!(
        size::TOOLBAR_SEPARATOR < size::TOOLBAR,
        "séparateur de toolbar aussi haut que la toolbar"
    );
    const _: () = assert!(
        size::TOOLBAR_STRIP > size::TABLE_HEADER,
        "bande secondaire de toolbar pas assez dégagée par rapport à un en-tête de table"
    );

    const _: () = assert!(size::ICON_BUTTON >= 28.0, "zone cliquable trop petite");
    const _: () = assert!(size::CONTROL >= 28.0, "zone cliquable trop petite");
    const _: () = assert!(
        size::SWITCH < size::CONTROL,
        "interrupteur aussi haut qu'un contrôle"
    );

    const _: () = assert!(radius::NONE < radius::MARKER, "rayons non différenciés");
    const _: () = assert!(radius::MARKER < radius::DOCUMENT, "rayons non différenciés");
    const _: () = assert!(radius::NONE < radius::BARRED, "rayons non différenciés");
    const _: () = assert!(radius::BARRED < radius::DOCUMENT, "rayons non différenciés");
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
    const _: () = assert!(size::DRAWER <= 460.0, "drawer trop large");
    const _: () = assert!(
        size::KANBAN_COLUMN >= 280.0,
        "colonne de pipeline illisible"
    );
    const _: () = assert!(
        size::ROW_MARKER < size::ROW,
        "marqueur de ligne de table plus haut que la ligne qui le porte"
    );
    const _: () = assert!(
        size::TABLE_SORT_ICON < size::TABLE_HEADER,
        "chevron de tri plus haut que l'en-tête qui le porte"
    );
    const _: () = assert!(
        size::LIST_MARKER < size::ROW,
        "marqueur de ligne de liste plus haut que la plus petite ligne qui le porte"
    );
    const _: () = assert!(
        size::TOAST < size::DIALOG_FORM,
        "toast aussi large qu'une modale de formulaire"
    );

    const _: () = assert!(stroke::HAIRLINE <= 1.0, "filet trop épais");
    const _: () = assert!(stroke::MARKER <= 2.0, "marqueur trop épais");
    const _: () = assert!(
        stroke::HAIRLINE < stroke::EMPHASIS,
        "filets non différenciés"
    );
    const _: () = assert!(stroke::EMPHASIS < stroke::MARKER, "filets non différenciés");

    const _: () = assert!(
        A4_RATIO > 1.41 && A4_RATIO < 1.42,
        "proportions A4 incorrectes"
    );

    const _: () = assert!(
        elevation::OFFSET < elevation::BLUR,
        "une ombre plus décalée qu'étalée paraît décollée"
    );
    const _: () = assert!(
        elevation::OFFSET < elevation::OVERLAY_OFFSET,
        "une modale doit se détacher plus franchement qu'une surface surélevée courante"
    );
    const _: () = assert!(
        elevation::BLUR < elevation::OVERLAY_BLUR,
        "une modale doit se détacher plus franchement qu'une surface surélevée courante"
    );
    const _: () = assert!(
        elevation::OFFSET < elevation::TOAST_OFFSET,
        "un toast flottant doit se détacher plus franchement qu'une surface surélevée courante"
    );
    const _: () = assert!(
        elevation::TOAST_OFFSET < elevation::OVERLAY_OFFSET,
        "un toast ne doit pas rivaliser avec l'élévation d'une modale bloquante"
    );
    const _: () = assert!(
        elevation::BLUR < elevation::TOAST_BLUR,
        "un toast flottant doit se détacher plus franchement qu'une surface surélevée courante"
    );
    const _: () = assert!(
        elevation::TOAST_BLUR < elevation::OVERLAY_BLUR,
        "un toast ne doit pas rivaliser avec l'élévation d'une modale bloquante"
    );
}
