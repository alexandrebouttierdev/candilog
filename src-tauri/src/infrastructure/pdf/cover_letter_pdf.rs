//! Export PDF d'une lettre de motivation, polices Geist embarquées.

use crate::core::errors::{AppError, AppResult};
use chrono::{Datelike, Local};
use printpdf::{
    Color, FontId, Op, ParsedFont, PdfDocument, PdfFontHandle, PdfPage, PdfSaveOptions, Point, Pt,
    Rgb, TextItem,
};
use std::path::Path;

#[derive(Clone, Copy)]
enum Weight {
    Regular,
    Medium,
    SemiBold,
}

struct Fonts {
    regular: ParsedFont,
    medium: ParsedFont,
    semibold: ParsedFont,
    regular_id: FontId,
    medium_id: FontId,
    semibold_id: FontId,
}

impl Fonts {
    fn source(&self, weight: Weight) -> &ParsedFont {
        match weight {
            Weight::Regular => &self.regular,
            Weight::Medium => &self.medium,
            Weight::SemiBold => &self.semibold,
        }
    }

    fn id(&self, weight: Weight) -> &FontId {
        match weight {
            Weight::Regular => &self.regular_id,
            Weight::Medium => &self.medium_id,
            Weight::SemiBold => &self.semibold_id,
        }
    }
}

fn rgb(r: f32, g: f32, b: f32) -> Color {
    Color::Rgb(Rgb {
        r: r / 255.0,
        g: g / 255.0,
        b: b / 255.0,
        icc_profile: None,
    })
}

const PAGE_W: f32 = 595.28;
const PAGE_H: f32 = 841.89;
const MARGIN: f32 = 56.7;
const CONTENT_W: f32 = PAGE_W - 2.0 * MARGIN;
const ASCENT: f32 = 0.8;
const TEXT: (f32, f32, f32) = (26.0, 26.0, 26.0);
const SECONDAIRE: (f32, f32, f32) = (91.0, 96.0, 112.0);

/// CoverLetter prête à exporter : identité du profil + corps généré.
#[derive(Debug, Clone, Default)]
pub struct CoverLetterPdf {
    pub name: String,
    pub city: Option<String>,
    pub email: String,
    pub subject: String,
    pub corps: String,
}

impl CoverLetterPdf {
    /// # Errors
    /// Font illisible ou écriture du fichier impossible.
    pub fn render_pdf(&self, path: &Path) -> AppResult<()> {
        let mut avertissements = Vec::new();
        let (regular, medium, semibold) = load_fonts()?;
        let mut document = PdfDocument::new("Lettre Candilog");
        let fonts = Fonts {
            regular_id: document.add_font(&regular),
            medium_id: document.add_font(&medium),
            semibold_id: document.add_font(&semibold),
            regular,
            medium,
            semibold,
        };

        let mut plan = Plan {
            ops: Vec::new(),
            pages: Vec::new(),
            fonts: &fonts,
            y: MARGIN,
        };
        plan.composer(self);

        let pages: Vec<PdfPage> = plan
            .pages_finales()
            .into_iter()
            .map(|ops| PdfPage::new(printpdf::Mm(210.0), printpdf::Mm(297.0), ops))
            .collect();
        let octets = document
            .with_pages(pages)
            .save(&PdfSaveOptions::default(), &mut avertissements);
        std::fs::write(path, octets)
            .map_err(|error| AppError::Database(format!("Impossible d'exporter le PDF : {error}")))
    }
}

fn load_fonts() -> AppResult<(ParsedFont, ParsedFont, ParsedFont)> {
    let decodage = |octets: &[u8]| -> AppResult<ParsedFont> {
        ParsedFont::from_bytes(octets, 0, &mut Vec::new())
            .ok_or_else(|| AppError::Serialization("Police lettre illisible".into()))
    };
    Ok((
        decodage(include_bytes!("../../../assets/fonts/Geist-Regular.ttf"))?,
        decodage(include_bytes!("../../../assets/fonts/Geist-Medium.ttf"))?,
        decodage(include_bytes!("../../../assets/fonts/Geist-SemiBold.ttf"))?,
    ))
}

fn date_du_day() -> String {
    let now = Local::now();
    let month = [
        "janvier",
        "février",
        "mars",
        "avril",
        "mai",
        "juin",
        "juillet",
        "août",
        "septembre",
        "octobre",
        "novembre",
        "décembre",
    ];
    let index = now.month0() as usize;
    format!(
        "{} {} {}",
        now.day(),
        month.get(index).copied().unwrap_or(""),
        now.year()
    )
}

struct Plan<'a> {
    ops: Vec<Op>,
    pages: Vec<Vec<Op>>,
    fonts: &'a Fonts,
    y: f32,
}

impl Plan<'_> {
    fn pdf_y(&self, y_haut: f32) -> f32 {
        PAGE_H - y_haut
    }

    fn pages_finales(mut self) -> Vec<Vec<Op>> {
        if !self.ops.is_empty() {
            self.pages.push(self.ops);
        }
        if self.pages.is_empty() {
            self.pages.push(Vec::new());
        }
        self.pages
    }

    fn assurer_place(&mut self, hauteur: f32) {
        if self.y + hauteur > PAGE_H - MARGIN {
            self.pages.push(std::mem::take(&mut self.ops));
            self.y = MARGIN;
        }
    }

    fn composer(&mut self, cover_letter: &CoverLetterPdf) {
        let name = if cover_letter.name.trim().is_empty() {
            "Candilog"
        } else {
            cover_letter.name.trim()
        };
        self.bloc_text(
            Weight::SemiBold,
            13.0,
            rgb(TEXT.0, TEXT.1, TEXT.2),
            18.0,
            name,
        );
        if let Some(city) = cover_letter.city.as_deref().filter(|v| !v.trim().is_empty()) {
            self.bloc_text(
                Weight::Regular,
                10.0,
                rgb(SECONDAIRE.0, SECONDAIRE.1, SECONDAIRE.2),
                14.0,
                city,
            );
        }
        if !cover_letter.email.trim().is_empty() {
            self.bloc_text(
                Weight::Regular,
                10.0,
                rgb(SECONDAIRE.0, SECONDAIRE.1, SECONDAIRE.2),
                14.0,
                cover_letter.email.trim(),
            );
        }
        self.y += 18.0;
        let location = cover_letter
            .city
            .as_deref()
            .filter(|v| !v.trim().is_empty())
            .map_or_else(
                || format!("Le {}", date_du_day()),
                |city| format!("{city}, le {}", date_du_day()),
            );
        self.bloc_text(
            Weight::Regular,
            10.0,
            rgb(SECONDAIRE.0, SECONDAIRE.1, SECONDAIRE.2),
            16.0,
            &location,
        );
        self.y += 10.0;
        self.bloc_text(
            Weight::Medium,
            11.0,
            rgb(TEXT.0, TEXT.1, TEXT.2),
            18.0,
            &cover_letter.subject,
        );
        self.y += 8.0;
        for paragraphe in cover_letter.corps.split("\n\n") {
            let text = paragraphe.trim();
            if text.is_empty() {
                continue;
            }
            self.paragraphe(
                Weight::Regular,
                11.0,
                rgb(TEXT.0, TEXT.1, TEXT.2),
                16.5,
                text,
            );
            self.y += 8.0;
        }
        self.y += 12.0;
        self.bloc_text(
            Weight::Medium,
            11.0,
            rgb(TEXT.0, TEXT.1, TEXT.2),
            16.0,
            name,
        );
    }

    fn bloc_text(
        &mut self,
        weight: Weight,
        size: f32,
        couleur: Color,
        interligne: f32,
        value: &str,
    ) {
        self.paragraphe(weight, size, couleur, interligne, value);
    }

    fn paragraphe(
        &mut self,
        weight: Weight,
        size: f32,
        couleur: Color,
        interligne: f32,
        value: &str,
    ) {
        for row_brute in value.lines() {
            let rows = if row_brute.trim().is_empty() {
                vec![String::new()]
            } else {
                self.decouper(weight, size, row_brute, CONTENT_W)
            };
            for row in rows {
                self.assurer_place(interligne);
                if !row.is_empty() {
                    self.text(
                        MARGIN,
                        self.y + ASCENT * size,
                        weight,
                        size,
                        couleur.clone(),
                        &row,
                    );
                }
                self.y += interligne;
            }
        }
    }

    fn text(
        &mut self,
        x: f32,
        ligne_de_base_haut: f32,
        weight: Weight,
        size: f32,
        couleur: Color,
        value: &str,
    ) {
        self.ops.push(Op::StartTextSection);
        self.ops.push(Op::SetFont {
            font: PdfFontHandle::External(self.fonts.id(weight).clone()),
            size: Pt(size),
        });
        self.ops.push(Op::SetFillColor { col: couleur });
        self.ops.push(Op::SetTextCursor {
            pos: Point {
                x: Pt(x),
                y: Pt(self.pdf_y(ligne_de_base_haut)),
            },
        });
        self.ops.push(Op::ShowText {
            items: vec![TextItem::Text(value.to_owned())],
        });
        self.ops.push(Op::EndTextSection);
    }

    fn largeur_text(&self, weight: Weight, size: f32, value: &str) -> f32 {
        let font = self.fonts.source(weight);
        let echelle = size / f32::from(font.units_per_em);
        value
            .chars()
            .map(|caractere| {
                font
                    .lookup_glyph_index(caractere as u32)
                    .and_then(|glyphe| font.get_glyph_width(glyphe))
                    .map_or(0.0, |largeur| largeur as f32 * echelle)
            })
            .sum()
    }

    fn decouper(&self, weight: Weight, size: f32, value: &str, largeur_max: f32) -> Vec<String> {
        let mut rows = Vec::new();
        let mut courante = String::new();
        for mot in value.split_whitespace() {
            let candidate = if courante.is_empty() {
                mot.to_owned()
            } else {
                format!("{courante} {mot}")
            };
            if self.largeur_text(weight, size, &candidate) <= largeur_max || courante.is_empty() {
                courante = candidate;
            } else {
                rows.push(std::mem::take(&mut courante));
                courante = mot.to_owned();
            }
        }
        if !courante.is_empty() {
            rows.push(courante);
        }
        rows
    }
}

#[cfg(test)]
#[path = "tests/lettre_pdf/mod.rs"]
mod tests;
