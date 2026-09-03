use crate::features::documents::application::build;
use crate::features::documents::domain::{ResumeDocument, ResumeIdentity};
use crate::infrastructure::pdf::resume_pdf::photo_dessinee;

/// CV minimal, identique dans les trois cas : seule la photo change.
fn document() -> ResumeDocument {
    ResumeDocument {
        identity: ResumeIdentity {
            full_name: "Alex Exemple".into(),
            title: "Administrateur systèmes".into(),
            email: "alex@exemple.fr".into(),
            ..ResumeIdentity::default()
        },
        profile: "Un profil de test.".into(),
        ..ResumeDocument::default()
    }
}

/// PNG de test au rapport donné, encodé en mémoire.
fn png(largeur: u32, hauteur: u32) -> Vec<u8> {
    let mut buffer = std::io::Cursor::new(Vec::new());
    image::RgbaImage::from_pixel(largeur, hauteur, image::Rgba([120, 130, 180, 255]))
        .write_to(&mut buffer, image::ImageFormat::Png)
        .unwrap();
    buffer.into_inner()
}

#[test]
fn un_cv_sans_photo_s_exporte_et_ne_contient_aucune_image() {
    let bytes = build(&document(), None).render_bytes().unwrap();
    let document = lopdf::Document::load_mem(&bytes).unwrap();

    assert_eq!(images(&document), 0);
}

#[test]
fn un_cv_avec_photo_embarque_exactement_une_image() {
    let bytes = build(&document(), Some(png(400, 400)))
        .render_bytes()
        .unwrap();
    let document = lopdf::Document::load_mem(&bytes).unwrap();

    assert_eq!(images(&document), 1);
}

#[test]
fn la_photo_est_inscrite_dans_son_cadre_sans_deformation() {
    // Cadre 26 × 30 mm : une image large est bornée par la largeur, une image haute par la
    // hauteur, et le rapport d'origine est conservé dans les deux cas.
    let (largeur, hauteur) = photo_dessinee(400, 200);
    assert!((largeur / hauteur - 2.0).abs() < 0.001);

    let (largeur, hauteur) = photo_dessinee(200, 400);
    assert!((largeur / hauteur - 0.5).abs() < 0.001);
}

/// Count d'images XObject embarquées dans le PDF.
fn images(document: &lopdf::Document) -> usize {
    document
        .objects
        .values()
        .filter(|objet| match objet {
            lopdf::Object::Stream(flux) => {
                flux.dict
                    .get(b"Subtype")
                    .ok()
                    .and_then(|valeur| valeur.as_name().ok())
                    == Some(b"Image")
            }
            _ => false,
        })
        .count()
}

#[test]
fn la_photo_ne_fait_pas_deborder_un_cv_qui_tenait_sur_la_page() {
    // Le cadre de la photo rétrécit l'en-tête : un nom et un titre longs s'y replient sur
    // davantage de lignes. Le CV doit malgré tout tenir sur sa page A4.
    let mut document = document();
    document.identity.full_name = "Alexandra-Charlotte Beaumont-Delaunay".into();
    document.identity.title = "Administratrice systèmes, réseaux et sécurité".into();
    document.identity.headline = Some(
        "Quinze ans d'exploitation, de supervision et de durcissement d'infrastructures \
         critiques en environnement réglementé."
            .into(),
    );

    assert!(build(&document, None).render_bytes().is_ok());
    assert!(build(&document, Some(png(400, 500))).render_bytes().is_ok());
}
