//! Un nom composé plus large que la colonne d'identité doit y rester, coupé comme le ferait
//! le navigateur : au trait d'union d'abord, en plein mot seulement en dernier recours.

use super::*;

fn lettre(prenom: &str, nom: &str) -> CoverLetterPdf {
    CoverLetterPdf {
        first_name: prenom.into(),
        last_name: nom.into(),
        title: Some("Administrateur systèmes".into()),
        address: Some("132 avenue du Maréchal-de-Lattre-de-Tassigny".into()),
        city: Some("Villeneuve-d'Ascq".into()),
        email: "jean-baptiste.delacroix@exemple.fr".into(),
        company: Some("Astek".into()),
        job_title: Some("Administrateur Système et Réseau".into()),
        corps: "Madame, Monsieur,\n\nJe vous adresse ma candidature.".into(),
        ..CoverLetterPdf::default()
    }
}

#[test]
fn un_patronyme_compose_est_coupe_au_trait_d_union() {
    let octets = lettre("Jean-Baptiste", "Delacroix-Vandenberghe")
        .render_bytes()
        .expect("un nom composé ne doit pas faire échouer l'export");
    let pdf = lopdf::Document::load_mem(&octets).unwrap();
    let texte = pdf.extract_text(&[1]).unwrap();

    assert!(
        texte.contains("Jean-\nBaptiste"),
        "le prénom doit passer à la ligne après le tiret, pas au milieu de « Baptiste » : {texte}"
    );
    assert!(
        texte.contains("Delacroix-\nVandenbe"),
        "la coupe en plein mot ne sert qu'après avoir épuisé les tirets : {texte}"
    );
}

/// Une adresse repliée à ses tirets ne doit pas revenir avec des espaces en trop : les
/// fragments d'un même mot se recollent sans séparateur.
#[test]
fn une_adresse_repliee_ne_gagne_pas_d_espaces() {
    let octets = lettre("Alex", "Exemple").render_bytes().unwrap();
    let pdf = lopdf::Document::load_mem(&octets).unwrap();
    let texte = pdf.extract_text(&[1]).unwrap();

    assert!(
        texte.contains("Maréchal-de-Lattre-\nde-Tassigny"),
        "les fragments d'un même mot doivent se recoller sans espace : {texte}"
    );
}
