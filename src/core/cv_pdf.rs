//! Export PDF natif à partir du modèle de CV commun.

use crate::shared::error::{AppError, AppResult};
use lopdf::content::{Content, Operation};
use lopdf::{dictionary, Document, Object, Stream};
use std::path::Path;

/// Modèle de mise en page partagé entre l'aperçu Iced et l'export PDF.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CvLayout {
    /// Nom complet affiché en en-tête.
    pub name: String,
    /// Titre professionnel.
    pub headline: String,
    /// Lignes textuelles déjà ordonnées pour le rendu.
    pub lines: Vec<String>,
}

impl CvLayout {
    /// Exporte le CV dans un PDF A4 utilisant la police PDF standard Helvetica.
    ///
    /// # Errors
    /// Retourne une erreur si le document ne peut pas être encodé ou enregistré.
    pub fn render_pdf(&self, path: &Path) -> AppResult<()> {
        let mut document = Document::with_version("1.5");
        let pages_id = document.new_object_id();
        let page_id = document.new_object_id();
        let font_id = document.add_object(dictionary! {
            "Type" => "Font",
            "Subtype" => "Type1",
            "BaseFont" => "Helvetica",
        });

        let mut operations = vec![
            Operation::new("BT", vec![]),
            Operation::new("Tf", vec![Object::Name(b"F1".to_vec()), 20.into()]),
            Operation::new("Td", vec![50.into(), 790.into()]),
            Operation::new(
                "Tj",
                vec![Object::string_literal(sanitize_pdf_text(&self.name))],
            ),
            Operation::new("Tf", vec![Object::Name(b"F1".to_vec()), 12.into()]),
            Operation::new("Td", vec![0.into(), (-24).into()]),
            Operation::new(
                "Tj",
                vec![Object::string_literal(sanitize_pdf_text(&self.headline))],
            ),
        ];
        for line in &self.lines {
            operations.push(Operation::new("Td", vec![0.into(), (-18).into()]));
            operations.push(Operation::new(
                "Tj",
                vec![Object::string_literal(sanitize_pdf_text(line))],
            ));
        }
        operations.push(Operation::new("ET", vec![]));

        let encoded = Content { operations }
            .encode()
            .map_err(|error| AppError::Serialization(error.to_string()))?;
        let content_id = document.add_object(Stream::new(dictionary! {}, encoded));
        document.objects.insert(
            page_id,
            Object::Dictionary(dictionary! {
                "Type" => "Page",
                "Parent" => pages_id,
                "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
                "Contents" => content_id,
                "Resources" => dictionary! { "Font" => dictionary! { "F1" => font_id } },
            }),
        );
        document.objects.insert(
            pages_id,
            Object::Dictionary(dictionary! {
                "Type" => "Pages",
                "Kids" => vec![page_id.into()],
                "Count" => 1,
            }),
        );
        let catalog_id =
            document.add_object(dictionary! { "Type" => "Catalog", "Pages" => pages_id });
        document.trailer.set("Root", catalog_id);
        document.compress();
        document.save(path).map_err(|error| {
            AppError::Database(format!("Impossible d'exporter le PDF : {error}"))
        })?;
        Ok(())
    }
}

fn sanitize_pdf_text(text: &str) -> String {
    text.chars()
        .map(|character| if character.is_ascii() { character } else { '?' })
        .collect()
}

#[cfg(test)]
#[path = "tests/cv_pdf/mod.rs"]
mod tests;
