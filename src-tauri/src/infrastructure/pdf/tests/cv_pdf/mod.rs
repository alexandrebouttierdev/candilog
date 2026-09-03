//! Cas de test isolés de l'export PDF.
use super::*;
use crate::infrastructure::pdf::A4;

mod export_pdf_garde_les_libelles_dans_leur_colonne;
mod export_pdf_place_la_photo_sans_la_deformer;
mod export_pdf_produit_un_document_lisible;
mod export_pdf_refuse_un_cv_trop_long;
mod export_pdf_refuse_un_token_unicode_trop_long;
mod export_pdf_replie_les_lignes_trop_larges;
mod export_pdf_reproduit_toutes_les_sections;

fn assert_a4_media_box(document: &lopdf::Document) {
    let page_id = *document.get_pages().values().next().unwrap();
    let page = document.get_dictionary(page_id).unwrap();
    let media_box = page.get(b"MediaBox").unwrap().as_array().unwrap();
    assert!((media_box[2].as_f32().unwrap() - A4.width_pt).abs() < 0.1);
    assert!((media_box[3].as_f32().unwrap() - A4.height_pt).abs() < 0.1);
}
