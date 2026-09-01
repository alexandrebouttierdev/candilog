//! Cas de test isolé.

use super::*;

/// Un nom long et un titre long doivent tenir dans la colonne d'identité.
///
/// Sans repli, le titre traversait la page et surimprimait la lettre ; le nom, lui,
/// était coupé en plein mot alors que le template ne coupe jamais un mot.
#[test]
fn export_pdf_garde_l_identite_dans_sa_colonne() {
    let titre =
        "Technicien supérieur systèmes et réseaux (TSSR) · Recherche contrat de professionnalisation";
    let cover_letter = CoverLetterPdf {
        first_name: "Alexandre".into(),
        last_name: "Bouttier".into(),
        title: Some(titre.into()),
        address: Some("Saint-Jacques-de-la-Lande (35)".into()),
        phone: Some("07 86 66 46 99".into()),
        email: "alexandrebouttier@gmail.com".into(),
        corps: "Madame, Monsieur,\n\nJe vous adresse ma candidature.".into(),
        ..CoverLetterPdf::default()
    };

    let octets = cover_letter.render_bytes().unwrap();
    let document = lopdf::Document::load_mem(&octets).unwrap();
    let texte = document.extract_text(&[1]).unwrap();

    eprintln!(
        "--T--
{texte}
--F--"
    );
    // Le prénom paraît deux fois entier : colonne d'identité et signature.
    // Compter évite qu'un prénom coupé dans la colonne passe grâce à la signature.
    assert_eq!(
        texte.matches("Alexandre").count(),
        2,
        "prénom coupé dans la colonne d'identité : {texte}"
    );
    assert_eq!(texte.matches("Bouttier").count(), 2);

    // Le titre est replié : il ne subsiste pas sur une seule ligne.
    assert!(
        !texte.contains(&titre.to_uppercase()),
        "le titre tient encore sur une ligne, donc il déborde : {texte}"
    );
    for mot in ["TECHNICIEN", "SUPÉRIEUR", "RECHERCHE", "CONTRAT"] {
        assert!(texte.contains(mot), "mot absent du titre : {mot}");
    }
}
