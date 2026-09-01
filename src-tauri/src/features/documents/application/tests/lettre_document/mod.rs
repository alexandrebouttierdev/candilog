//! Tests du modèle de lettre fusionné.

use super::*;
use crate::features::documents::domain::CoverLetterExport;
use crate::features::profile::domain::{Identity, Profile};

fn profile() -> Profile {
    Profile {
        identity: Identity {
            first_name: "Alex".into(),
            name: "Exemple".into(),
            email: "alex@exemple.fr".into(),
            city: Some("Rennes".into()),
            ..Identity::default()
        },
        ..Profile::default()
    }
}

fn cover_letter() -> CoverLetterExport {
    CoverLetterExport {
        name: "Lettre Nova".into(),
        company: Some("Nova".into()),
        job_title: Some("Développeur".into()),
        recipient: Some("Service recrutement".into()),
        recipient_address: Some("12 rue de la Monnaie, 35000 Rennes".into()),
        job_reference: Some("FS-2026-114".into()),
        content: "Madame, Monsieur,".into(),
    }
}

mod pose_l_identite_et_l_objet;
