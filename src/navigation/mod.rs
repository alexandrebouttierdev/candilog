//! Navigation typée sans routeur web.

use crate::ui::components::icon::Icon;

/// Espaces de travail de premier niveau affichés dans le rail desktop.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Section {
    /// Vue d'ensemble.
    Dashboard,
    /// Pipeline et calendrier.
    Suivi,
    /// Entreprises et réseau.
    Relations,
    /// Bibliothèque et outils documentaires.
    Documents,
    /// Statistiques métier.
    Analyse,
    /// Profil professionnel.
    Profil,
    /// Préférences locales et maintenance.
    Reglages,
}

impl Section {
    /// Intitulé de l'espace, réutilisé par les surfaces de navigation contextuelle.
    #[must_use]
    pub const fn label(self) -> Option<&'static str> {
        match self {
            Self::Dashboard => None,
            Self::Suivi => Some("Suivi"),
            Self::Relations => Some("Relations"),
            Self::Documents => Some("Documents"),
            Self::Analyse => Some("Analyses"),
            Self::Profil => Some("Profil"),
            Self::Reglages => Some("Réglages"),
        }
    }

    /// Icône de la tuile du rail représentant ce groupe.
    #[must_use]
    pub const fn icon(self) -> Icon {
        match self {
            Self::Dashboard => Icon::Dashboard,
            Self::Suivi => Icon::Applications,
            Self::Relations => Icon::Network,
            Self::Documents => Icon::Document,
            Self::Analyse => Icon::Chart,
            Self::Profil => Icon::Profile,
            Self::Reglages => Icon::Settings,
        }
    }

    /// Écran ouvert lorsqu'on active ce groupe depuis le rail.
    #[must_use]
    pub const fn default_route(self) -> Route {
        match self {
            Self::Dashboard => Route::Dashboard,
            Self::Suivi => Route::Candidatures,
            Self::Relations => Route::Entreprises,
            Self::Documents => Route::Cv,
            Self::Analyse => Route::Statistiques,
            Self::Profil => Route::Profil,
            Self::Reglages => Route::Parametres,
        }
    }

    /// Intitulé court affiché sous la pastille d'une tuile de rail.
    #[must_use]
    pub const fn short_label(self) -> &'static str {
        match self {
            Self::Dashboard => "Accueil",
            Self::Suivi => "Suivi",
            Self::Relations => "Relations",
            Self::Documents => "Documents",
            Self::Analyse => "Analyses",
            Self::Profil => "Profil",
            Self::Reglages => "Réglages",
        }
    }

    /// Intitulé complet, donné en infobulle lorsque la tuile est repliée.
    #[must_use]
    pub const fn long_label(self) -> &'static str {
        match self {
            Self::Dashboard => "Tableau de bord",
            Self::Suivi => "Candidatures et calendrier",
            Self::Relations => "Entreprises et réseau",
            Self::Documents => "CV et lettres de motivation",
            Self::Analyse => "Statistiques",
            Self::Profil => "Profil professionnel",
            Self::Reglages => "Intelligence artificielle et maintenance",
        }
    }

    /// Sections dans l'ordre de la barre latérale.
    pub const ALL: [Self; 7] = [
        Self::Dashboard,
        Self::Suivi,
        Self::Relations,
        Self::Documents,
        Self::Analyse,
        Self::Profil,
        Self::Reglages,
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
    /// Sauvegarde et restauration des données.
    Sauvegardes,
    /// Recherche et installation des mises à jour.
    MisesAJour,
    /// Informations sur l'application et son auteur.
    APropos,
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
            Self::Sauvegardes => Icon::Save,
            Self::MisesAJour => Icon::Download,
            Self::APropos => Icon::Info,
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
            Self::CvGenerator => "Générer un CV",
            Self::LettreMotivation => "Lettre de motivation",
            Self::CvImport => "Analyser un CV",
            Self::Profil => "Profil",
            Self::Parametres => "Intelligence artificielle",
            Self::Sauvegardes => "Sauvegardes",
            Self::MisesAJour => "Mises à jour",
            Self::APropos => "À propos",
        }
    }

    /// Groupe auquel appartient l'écran dans la barre latérale.
    #[must_use]
    pub const fn section(self) -> Section {
        match self {
            Self::Dashboard => Section::Dashboard,
            Self::Candidatures | Self::Calendrier => Section::Suivi,
            Self::Entreprises | Self::Reseau => Section::Relations,
            Self::Cv | Self::CvGenerator | Self::LettreMotivation | Self::CvImport => {
                Section::Documents
            }
            Self::Statistiques => Section::Analyse,
            Self::Profil => Section::Profil,
            Self::Parametres | Self::Sauvegardes | Self::MisesAJour | Self::APropos => {
                Section::Reglages
            }
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
    pub const ALL: [Self; 15] = [
        Self::Dashboard,
        Self::Candidatures,
        Self::Calendrier,
        Self::Entreprises,
        Self::Reseau,
        Self::Cv,
        Self::CvGenerator,
        Self::LettreMotivation,
        Self::CvImport,
        Self::Statistiques,
        Self::Profil,
        Self::Parametres,
        Self::Sauvegardes,
        Self::MisesAJour,
        Self::APropos,
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
    fn le_tableau_de_bord_n_affiche_pas_d_intitule() {
        assert_eq!(Section::Dashboard.label(), None);
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

    #[test]
    fn chaque_groupe_porte_une_icone_distincte() {
        let icones: std::collections::BTreeSet<_> = Section::ALL
            .iter()
            .map(|section| format!("{:?}", section.icon()))
            .collect();
        assert_eq!(icones.len(), Section::ALL.len());
    }

    #[test]
    fn la_route_par_defaut_d_un_groupe_appartient_au_groupe() {
        for section in Section::ALL {
            let route = section.default_route();
            assert_eq!(route.section(), section);
            assert_eq!(Some(route), Route::of_section(section).first().copied());
        }
    }

    #[test]
    fn chaque_groupe_porte_deux_intitules_distincts() {
        for section in Section::ALL {
            assert!(!section.short_label().is_empty());
            assert!(!section.long_label().is_empty());
            assert_ne!(section.short_label(), section.long_label());
        }
    }
}
