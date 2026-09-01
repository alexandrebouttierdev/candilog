//! Cas de test isolés de l'export PDF des lettres.
use super::*;
use crate::infrastructure::pdf::A4;

mod export_pdf_garde_l_identite_dans_sa_colonne;
mod export_pdf_produit_un_document_lisible;
mod export_pdf_refuse_un_token_unicode_trop_long;
mod export_pdf_refuse_une_lettre_trop_longue;
mod export_pdf_rend_la_mise_en_forme;
mod export_pdf_reproduit_la_mise_en_page_du_template;

fn assert_a4_media_box(document: &lopdf::Document) {
    let page_id = *document.get_pages().values().next().unwrap();
    let page = document.get_dictionary(page_id).unwrap();
    let media_box = page.get(b"MediaBox").unwrap().as_array().unwrap();
    assert!((media_box[2].as_f32().unwrap() - A4.width_pt).abs() < 0.1);
    assert!((media_box[3].as_f32().unwrap() - A4.height_pt).abs() < 0.1);
}
