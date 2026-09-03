//! Cas de test isolé.

use crate::features::documents::application::build;
use crate::features::documents::domain::{ResumeDocument, ResumeExperienceBlock, ResumeIdentity};

fn document() -> ResumeDocument {
    ResumeDocument {
        identity: ResumeIdentity {
            full_name: "Alex Exemple".into(),
            title: "Administrateur systèmes".into(),
            email: "alex@exemple.fr".into(),
            ..ResumeIdentity::default()
        },
        profile: "Profil de test.".into(),
        experiences: vec![ResumeExperienceBlock {
            id: "exp-1".into(),
            title: "Ingénieur".into(),
            // Retour à la ligne saisi dans un champ mono-ligne : le cas réel qui
            // sortait en rectangle, l'aperçu HTML le rendant lui comme une espace.
            company: "OVH\nCloud".into(),
            location: None,
            period: "2024".into(),
            bullets: vec!["Mission.".into()],
        }],
        ..ResumeDocument::default()
    }
}

/// Le libellé de section vit dans une colonne étroite : sans repli il surimprimait
/// le contenu placé à sa droite.
#[test]
fn le_libelle_de_section_se_replie_dans_sa_colonne() {
    let octets = build(&document(), None).render_bytes().unwrap();
    let pdf = lopdf::Document::load_mem(&octets).unwrap();
    let texte = pdf.extract_text(&[1]).unwrap().to_uppercase();

    assert!(texte.contains("EXPÉRIENCES"));
    assert!(
        !texte.contains("EXPÉRIENCES PROFESSIONNELLES"),
        "le libellé tient encore sur une ligne, donc il déborde : {texte}"
    );
}

/// Un caractère sans glyphe sortait en rectangle vide au lieu d'une espace.
#[test]
fn un_caractere_sans_glyphe_ne_sort_pas_en_rectangle() {
    let octets = build(&document(), None).render_bytes().unwrap();
    let pdf = lopdf::Document::load_mem(&octets).unwrap();
    let texte = pdf.extract_text(&[1]).unwrap();

    assert!(
        texte.contains("OVH Cloud"),
        "le caractère absent n'a pas été remplacé par une espace : {texte}"
    );
}
