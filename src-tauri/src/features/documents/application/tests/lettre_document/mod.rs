//! Tests du modèle de lettre fusionné.

use super::*;
use crate::features::documents::domain::ExportLettre;
use crate::features::profil::domain::{Identite, Profil};

fn profil() -> Profil {
    Profil {
        identite: Identite {
            prenom: "Alex".into(),
            nom: "Exemple".into(),
            email: "alex@exemple.fr".into(),
            ville: Some("Rennes".into()),
            ..Identite::default()
        },
        ..Profil::default()
    }
}

fn lettre() -> ExportLettre {
    ExportLettre {
        nom: "Lettre Nova".into(),
        entreprise: Some("Nova".into()),
        poste: Some("Développeur".into()),
        contenu: "Madame, Monsieur,".into(),
    }
}

mod pose_l_identite_et_l_objet;
