//! Navigation typée sans routeur web.

use crate::ui::components::icon::Icon;

/// Regroupement d'écrans dans la barre latérale.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    /// Vue d'ensemble, sans intitulé de groupe.
    Pilotage,
    /// Suivi du pipeline et de l'agenda.
    Recherche,
    /// Entreprises et contacts.
    Relations,
    /// CV, lettres et outils IA.
    Documents,
    /// Profil et configuration.
    Systeme,
}

impl Section {
    /// Intitulé discret affiché au-dessus du groupe, absent pour le pilotage.
    #[must_use]
    pub const fn label(self) -> Option<&'static str> {
        match self {
            Self::Pilotage => None,
            Self::Recherche => Some("Recherche"),
            Self::Relations => Some("Relations"),
            Self::Documents => Some("Documents"),
            Self::Systeme => Some("Système"),
        }
    }

    /// Sections dans l'ordre de la barre latérale.
    pub const ALL: [Self; 5] = [
        Self::Pilotage,
        Self::Recherche,
        Self::Relations,
        Self::Documents,
        Self::Systeme,
    ];
}

/// Écrans natifs de Candilog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Route {
    /// Tableau de bord.
    #[default]
    Dashboard,
    /// Pipeline Kanban ou liste.
    Candidatures,
    /// Calendrier des entretiens et relances.
    Calendrier,
    /// Statistiques métier, ATS et IA.
    Statistiques,
    /// Entreprises.
    Entreprises,
    /// Réseau de contacts.
    Reseau,
    /// Versions de CV.
    Cv,
    /// Générateur de CV assisté par IA.
    CvGenerator,
    /// Générateur de lettre de motivation.
    LettreMotivation,
    /// Analyse d'un CV PDF externe.
    CvImport,
    /// Profil professionnel.
    Profil,
    /// Paramètres.
    Parametres,
}

impl Route {
    /// Icône vectorielle affichée dans la navigation native.
    #[must_use]
    pub const fn icon(self) -> Icon {
        match self {
            Self::Dashboard => Icon::Dashboard,
            Self::Candidatures => Icon::Applications,
            Self::Calendrier => Icon::Calendar,
            Self::Statistiques => Icon::Chart,
            Self::Entreprises => Icon::Building,
            Self::Reseau => Icon::Network,
            Self::Cv => Icon::Document,
            Self::CvGenerator => Icon::Sparkles,
            Self::LettreMotivation => Icon::Letter,
            Self::CvImport => Icon::Import,
            Self::Profil => Icon::Profile,
            Self::Parametres => Icon::Settings,
        }
    }

    /// Libellé français affiché dans la navigation.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Dashboard => "Tableau de bord",
            Self::Candidatures => "Candidatures",
            Self::Calendrier => "Calendrier",
            Self::Statistiques => "Statistiques",
            Self::Entreprises => "Entreprises",
            Self::Reseau => "Réseau",
            Self::Cv => "Mes CV",
            Self::CvGenerator => "CV Generator",
            Self::LettreMotivation => "Lettre de motivation",
            Self::CvImport => "Analysez un CV",
            Self::Profil => "Mon Profil",
            Self::Parametres => "Paramètres",
        }
    }

    /// Groupe auquel appartient l'écran dans la barre latérale.
    #[must_use]
    pub const fn section(self) -> Section {
        match self {
            Self::Dashboard => Section::Pilotage,
            Self::Candidatures | Self::Calendrier | Self::Statistiques => Section::Recherche,
            Self::Entreprises | Self::Reseau => Section::Relations,
            Self::Cv | Self::CvGenerator | Self::LettreMotivation | Self::CvImport => {
                Section::Documents
            }
            Self::Profil | Self::Parametres => Section::Systeme,
        }
    }

    /// Chiffre du raccourci `Ctrl+n`, quand l'écran en possède un.
    #[must_use]
    pub const fn shortcut(self) -> Option<char> {
        match self {
            Self::Dashboard => Some('1'),
            Self::Candidatures => Some('2'),
            Self::Calendrier => Some('3'),
            Self::Statistiques => Some('4'),
            Self::Entreprises => Some('5'),
            Self::Reseau => Some('6'),
            Self::Cv => Some('7'),
            Self::CvGenerator => Some('8'),
            Self::LettreMotivation => Some('9'),
            _ => None,
        }
    }

    /// Résout un raccourci clavier vers son écran.
    #[must_use]
    pub fn from_shortcut(digit: char) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|route| route.shortcut() == Some(digit))
    }

    /// Écrans du groupe donné, dans l'ordre de la barre latérale.
    #[must_use]
    pub fn of_section(section: Section) -> Vec<Self> {
        Self::ALL
            .into_iter()
            .filter(|route| route.section() == section)
            .collect()
    }

    /// Toutes les routes, dans l'ordre de la barre latérale.
    pub const ALL: [Self; 12] = [
        Self::Dashboard,
        Self::Candidatures,
        Self::Calendrier,
        Self::Statistiques,
        Self::Entreprises,
        Self::Reseau,
        Self::Cv,
        Self::CvGenerator,
        Self::LettreMotivation,
        Self::CvImport,
        Self::Profil,
        Self::Parametres,
    ];
}

#[cfg(test)]
mod tests {
    use super::{Route, Section};

    #[test]
    fn chaque_ecran_appartient_a_un_seul_groupe() {
        let mut total = 0;
        for section in Section::ALL {
            total += Route::of_section(section).len();
        }
        assert_eq!(total, Route::ALL.len());
    }

    #[test]
    fn les_groupes_conservent_l_ordre_de_la_barre_laterale() {
        let ordered: Vec<Route> = Section::ALL
            .into_iter()
            .flat_map(Route::of_section)
            .collect();
        assert_eq!(ordered, Route::ALL.to_vec());
    }

    #[test]
    fn aucun_groupe_n_est_vide() {
        for section in Section::ALL {
            assert!(
                !Route::of_section(section).is_empty(),
                "groupe vide : {section:?}"
            );
        }
    }

    #[test]
    fn le_pilotage_n_affiche_pas_d_intitule() {
        assert_eq!(Section::Pilotage.label(), None);
        for section in Section::ALL.into_iter().skip(1) {
            assert!(section.label().is_some());
        }
    }

    #[test]
    fn les_raccourcis_sont_uniques_et_reversibles() {
        let mut seen = std::collections::BTreeSet::new();
        for route in Route::ALL {
            if let Some(digit) = route.shortcut() {
                assert!(seen.insert(digit), "raccourci dupliqué : {digit}");
                assert_eq!(Route::from_shortcut(digit), Some(route));
            }
        }
        assert_eq!(Route::from_shortcut('0'), None);
    }

    #[test]
    fn chaque_ecran_porte_un_libelle_et_une_icone_distincts() {
        let labels: std::collections::BTreeSet<_> =
            Route::ALL.iter().map(|route| route.label()).collect();
        assert_eq!(labels.len(), Route::ALL.len());
        let icons: std::collections::BTreeSet<_> = Route::ALL
            .iter()
            .map(|route| format!("{:?}", route.icon()))
            .collect();
        assert_eq!(icons.len(), Route::ALL.len());
    }

    #[test]
    fn la_route_par_defaut_est_le_tableau_de_bord() {
        assert_eq!(Route::default(), Route::Dashboard);
    }
}
