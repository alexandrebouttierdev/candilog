//! Export PDF d'une lettre de motivation, polices Geist embarquées.

use crate::core::errors::{AppError, AppResult};
use crate::infrastructure::pdf::page::{
    ensure_inside, Density, LayoutBounds, Margins, A4, DENSITY_PROFILES, MIN_BODY_FONT_PT,
};
use chrono::{Datelike, Local};
use printpdf::{
    Color, FontId, Op, ParsedFont, PdfDocument, PdfFontHandle, PdfPage, PdfSaveOptions, Point, Pt,
    Rgb, TextItem,
};
use std::path::Path;
use unicode_segmentation::UnicodeSegmentation;

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

const MARGIN: f32 = 56.7;
const CONTENT_W: f32 = A4.width_pt - 2.0 * MARGIN;
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
        let bytes = self.render_bytes()?;
        std::fs::write(path, bytes)
            .map_err(|error| AppError::Database(format!("Impossible d'exporter le PDF : {error}")))
    }

    /// Planifie entièrement une page A4 avant de la sérialiser.
    ///
    /// # Errors
    /// Refuse le document si aucune densité lisible ne tient sur une page.
    pub fn render_bytes(&self) -> AppResult<Vec<u8>> {
        for density in DENSITY_PROFILES {
            if let Some(bytes) = self.render_density(density)? {
                return Ok(bytes);
            }
        }
        Err(AppError::Validation(
            "La lettre ne tient pas sur une page A4. Raccourcissez son contenu avant l'export."
                .into(),
        ))
    }

    fn render_density(&self, density: Density) -> AppResult<Option<Vec<u8>>> {
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
            fonts: &fonts,
            y: MARGIN,
            density,
            bounds: LayoutBounds::default(),
            overflow: false,
        };
        plan.composer(self);
        let margins = Margins::uniform(MARGIN);
        if plan.overflow || ensure_inside(plan.bounds, margins, "overflow").is_err() {
            return Ok(None);
        }
        let page = PdfPage::new(
            printpdf::Mm(A4.width_mm),
            printpdf::Mm(A4.height_mm),
            plan.ops,
        );
        let octets = document
            .with_pages(vec![page])
            .save(&PdfSaveOptions::default(), &mut avertissements);
        Ok(Some(octets))
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
    fonts: &'a Fonts,
    y: f32,
    density: Density,
    bounds: LayoutBounds,
    overflow: bool,
}

impl Plan<'_> {
    fn pdf_y(&self, y_haut: f32) -> f32 {
        A4.height_pt - y_haut
    }

    fn font_size(&self, size: f32) -> f32 {
        (size * self.density.font_scale).max(MIN_BODY_FONT_PT.min(size))
    }

    fn spacing(&self, value: f32) -> f32 {
        value * self.density.spacing_scale
    }

    fn avance(&mut self, value: f32) {
        self.y += self.spacing(value);
        self.bounds.max_y = self.bounds.max_y.max(self.y);
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
        if let Some(city) = cover_letter
            .city
            .as_deref()
            .filter(|v| !v.trim().is_empty())
        {
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
        self.avance(18.0);
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
        self.avance(10.0);
        self.bloc_text(
            Weight::Medium,
            11.0,
            rgb(TEXT.0, TEXT.1, TEXT.2),
            18.0,
            &cover_letter.subject,
        );
        self.avance(8.0);
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
            self.avance(8.0);
        }
        self.avance(12.0);
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
        let actual_size = self.font_size(size);
        let actual_line_height = self.spacing(interligne).max(actual_size * 1.1);
        for row_brute in value.lines() {
            let rows = if row_brute.trim().is_empty() {
                vec![String::new()]
            } else {
                self.decouper(weight, size, row_brute, CONTENT_W)
            };
            for row in rows {
                if self.y + actual_line_height > A4.height_pt - MARGIN {
                    self.overflow = true;
                    self.bounds.max_y = self.y + actual_line_height;
                    return;
                }
                if !row.is_empty() {
                    self.text(
                        MARGIN,
                        self.y + ASCENT * actual_size,
                        weight,
                        size,
                        couleur.clone(),
                        &row,
                    );
                }
                self.y += actual_line_height;
                self.bounds.max_y = self.bounds.max_y.max(self.y);
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
        if self.overflow {
            return;
        }
        let size = self.font_size(size);
        self.bounds.max_x = self
            .bounds
            .max_x
            .max(x + self.largeur_text_actual(weight, size, value));
        self.bounds.max_y = self.bounds.max_y.max(ligne_de_base_haut + size * 0.25);
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
        self.largeur_text_actual(weight, self.font_size(size), value)
    }

    fn largeur_text_actual(&self, weight: Weight, size: f32, value: &str) -> f32 {
        let font = self.fonts.source(weight);
        let echelle = size / f32::from(font.units_per_em);
        value
            .chars()
            .map(|caractere| {
                font.lookup_glyph_index(caractere as u32)
                    .and_then(|glyphe| font.get_glyph_width(glyphe))
                    .map_or(0.0, |largeur| largeur as f32 * echelle)
            })
            .sum()
    }

    fn decouper(&self, weight: Weight, size: f32, value: &str, largeur_max: f32) -> Vec<String> {
        let mut rows = Vec::new();
        let mut courante = String::new();
        for mot in value.split_whitespace() {
            for fragment in self.decouper_token(weight, size, mot, largeur_max) {
                let candidate = if courante.is_empty() {
                    fragment.clone()
                } else {
                    format!("{courante} {fragment}")
                };
                if self.largeur_text(weight, size, &candidate) <= largeur_max {
                    courante = candidate;
                } else {
                    if !courante.is_empty() {
                        rows.push(std::mem::take(&mut courante));
                    }
                    courante = fragment;
                }
            }
        }
        if !courante.is_empty() {
            rows.push(courante);
        }
        rows
    }

    fn decouper_token(
        &self,
        weight: Weight,
        size: f32,
        token: &str,
        largeur_max: f32,
    ) -> Vec<String> {
        if self.largeur_text(weight, size, token) <= largeur_max {
            return vec![token.to_owned()];
        }
        let mut fragments = Vec::new();
        let mut current = String::new();
        for grapheme in token.graphemes(true) {
            let candidate = format!("{current}{grapheme}");
            if !current.is_empty() && self.largeur_text(weight, size, &candidate) > largeur_max {
                fragments.push(std::mem::take(&mut current));
            }
            current.push_str(grapheme);
        }
        if !current.is_empty() {
            fragments.push(current);
        }
        fragments
    }
}

#[cfg(test)]
#[path = "tests/lettre_pdf/mod.rs"]
mod tests;
