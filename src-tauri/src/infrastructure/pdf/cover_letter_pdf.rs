//! Export PDF d'une lettre de motivation, polices Geist embarquées.

use crate::core::errors::{AppError, AppResult};
use crate::features::documents::domain::{parse_letter, LetterAlign, LetterParagraph, LetterRun};
use crate::infrastructure::pdf::page::{
    ensure_inside, Density, LayoutBounds, Margins, A4, DENSITY_PROFILES, MIN_BODY_FONT_PT,
};
use chrono::{Datelike, Local};
use printpdf::{
    Color, FontId, Line, LinePoint, Op, ParsedFont, PdfDocument, PdfFontHandle, PdfPage,
    PdfSaveOptions, Point, Pt, Rgb, TextItem,
};
use std::path::Path;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Weight {
    Regular,
    Medium,
    SemiBold,
    Bold,
}

struct Fonts {
    regular: ParsedFont,
    medium: ParsedFont,
    semibold: ParsedFont,
    bold: ParsedFont,
    regular_id: FontId,
    medium_id: FontId,
    semibold_id: FontId,
    bold_id: FontId,
}

impl Fonts {
    fn source(&self, weight: Weight) -> &ParsedFont {
        match weight {
            Weight::Regular => &self.regular,
            Weight::Medium => &self.medium,
            Weight::SemiBold => &self.semibold,
            Weight::Bold => &self.bold,
        }
    }

    fn id(&self, weight: Weight) -> &FontId {
        match weight {
            Weight::Regular => &self.regular_id,
            Weight::Medium => &self.medium_id,
            Weight::SemiBold => &self.semibold_id,
            Weight::Bold => &self.bold_id,
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
        let (regular, medium, semibold, bold) = load_fonts()?;
        let mut document = PdfDocument::new("Lettre Candilog");
        let fonts = Fonts {
            regular_id: document.add_font(&regular),
            medium_id: document.add_font(&medium),
            semibold_id: document.add_font(&semibold),
            bold_id: document.add_font(&bold),
            regular,
            medium,
            semibold,
            bold,
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

fn load_fonts() -> AppResult<(ParsedFont, ParsedFont, ParsedFont, ParsedFont)> {
    let decodage = |octets: &[u8]| -> AppResult<ParsedFont> {
        ParsedFont::from_bytes(octets, 0, &mut Vec::new())
            .ok_or_else(|| AppError::Serialization("Police lettre illisible".into()))
    };
    Ok((
        decodage(include_bytes!("../../../assets/fonts/Geist-Regular.ttf"))?,
        decodage(include_bytes!("../../../assets/fonts/Geist-Medium.ttf"))?,
        decodage(include_bytes!("../../../assets/fonts/Geist-SemiBold.ttf"))?,
        decodage(include_bytes!("../../../assets/fonts/Geist-Bold.ttf"))?,
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

/// Fragment homogène déjà mesuré.
struct Segment {
    weight: Weight,
    underline: bool,
    text: String,
    largeur: f32,
}

/// Mot insécable : un ou plusieurs fragments collés, avec leur largeur totale.
struct Mot {
    segments: Vec<Segment>,
    largeur: f32,
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
        for paragraphe in parse_letter(&cover_letter.corps) {
            if paragraphe.runs.is_empty() {
                self.avance(8.0);
                continue;
            }
            self.paragraphe_riche(&paragraphe, rgb(TEXT.0, TEXT.1, TEXT.2));
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

    /// Compose un paragraphe mis en forme : gras et souligné par fragment, taille et
    /// alignement par paragraphe.
    ///
    /// La coupure de ligne se fait sur des **mots** et non sur des fragments : un mot dont
    /// une partie seulement est en gras reste insécable, sinon « auto**matique** » se
    /// couperait en deux avec un blanc au milieu.
    fn paragraphe_riche(&mut self, paragraphe: &LetterParagraph, couleur: Color) {
        let size = 11.0 * paragraphe.size.scale();
        let interligne = 16.5 * paragraphe.size.scale();
        let actual_size = self.font_size(size);
        let hauteur_ligne = self.spacing(interligne).max(actual_size * 1.1);
        let espace = self.largeur_text(Weight::Regular, size, " ");

        for ligne in self.decouper_mots(&paragraphe.runs, size) {
            if self.y + hauteur_ligne > A4.height_pt - MARGIN {
                self.overflow = true;
                self.bounds.max_y = self.y + hauteur_ligne;
                return;
            }
            let largeur: f32 = ligne.iter().map(|mot| mot.largeur).sum::<f32>()
                + espace * (ligne.len().saturating_sub(1)) as f32;
            let mut x = match paragraphe.align {
                LetterAlign::Left => MARGIN,
                LetterAlign::Center => MARGIN + (CONTENT_W - largeur).max(0.0) / 2.0,
                LetterAlign::Right => MARGIN + (CONTENT_W - largeur).max(0.0),
            };
            let ligne_de_base = self.y + ASCENT * actual_size;
            for segment in self.fusionner(&ligne, size) {
                self.text(
                    x,
                    ligne_de_base,
                    segment.weight,
                    size,
                    couleur.clone(),
                    &segment.text,
                );
                if segment.underline {
                    self.souligner(x, x + segment.largeur, ligne_de_base + 1.6, couleur.clone());
                }
                x += segment.largeur;
            }
            self.y += hauteur_ligne;
            self.bounds.max_y = self.bounds.max_y.max(self.y);
        }
    }

    /// Recolle les fragments voisins de même style, espaces compris.
    ///
    /// Sans cette fusion, chaque mot serait un ordre d'affichage distinct : le PDF pèserait
    /// plus lourd et surtout, copié ou relu par un ATS, le texte perdrait ses espaces.
    fn fusionner(&self, ligne: &[Mot], size: f32) -> Vec<Segment> {
        let mut sortie: Vec<Segment> = Vec::new();
        let ajouter = |segment: Segment, sortie: &mut Vec<Segment>| match sortie.last_mut() {
            Some(dernier)
                if dernier.weight == segment.weight && dernier.underline == segment.underline =>
            {
                dernier.text.push_str(&segment.text);
                dernier.largeur += segment.largeur;
            }
            _ => sortie.push(segment),
        };
        for (index, mot) in ligne.iter().enumerate() {
            if index > 0 {
                let (weight, underline) =
                    sortie.last().map_or((Weight::Regular, false), |dernier| {
                        (dernier.weight, dernier.underline)
                    });
                ajouter(
                    Segment {
                        weight,
                        underline,
                        text: " ".into(),
                        largeur: self.largeur_text(weight, size, " "),
                    },
                    &mut sortie,
                );
            }
            for segment in &mot.segments {
                ajouter(
                    Segment {
                        weight: segment.weight,
                        underline: segment.underline,
                        text: segment.text.clone(),
                        largeur: segment.largeur,
                    },
                    &mut sortie,
                );
            }
        }
        sortie
    }

    /// Regroupe les fragments en mots insécables, puis les mots en lignes.
    fn decouper_mots(&self, runs: &[LetterRun], size: f32) -> Vec<Vec<Mot>> {
        let mut mots: Vec<Mot> = Vec::new();
        let mut colle = false;
        for run in runs {
            let weight = if run.bold {
                Weight::Bold
            } else {
                Weight::Regular
            };
            let attache_debut = colle && !run.text.starts_with(char::is_whitespace);
            let mut premier = true;
            for token in run.text.split_whitespace() {
                for fragment in self.decouper_token(weight, size, token, CONTENT_W) {
                    let largeur = self.largeur_text(weight, size, &fragment);
                    let segment = Segment {
                        weight,
                        underline: run.underline,
                        text: fragment,
                        largeur,
                    };
                    match mots.last_mut() {
                        Some(mot) if premier && attache_debut => {
                            mot.largeur += segment.largeur;
                            mot.segments.push(segment);
                        }
                        _ => mots.push(Mot {
                            largeur: segment.largeur,
                            segments: vec![segment],
                        }),
                    }
                    premier = false;
                }
            }
            colle = !run.text.ends_with(char::is_whitespace) && !run.text.is_empty();
        }

        let espace = self.largeur_text(Weight::Regular, size, " ");
        let mut lignes: Vec<Vec<Mot>> = Vec::new();
        let mut courante: Vec<Mot> = Vec::new();
        let mut largeur = 0.0;
        for mot in mots {
            let ajout = if courante.is_empty() {
                mot.largeur
            } else {
                espace + mot.largeur
            };
            if !courante.is_empty() && largeur + ajout > CONTENT_W {
                lignes.push(std::mem::take(&mut courante));
                largeur = mot.largeur;
            } else {
                largeur += ajout;
            }
            courante.push(mot);
        }
        if !courante.is_empty() {
            lignes.push(courante);
        }
        lignes
    }

    fn souligner(&mut self, x1: f32, x2: f32, y_haut: f32, couleur: Color) {
        if self.overflow {
            return;
        }
        self.ops.push(Op::SetOutlineColor { col: couleur });
        self.ops.push(Op::SetOutlineThickness { pt: Pt(0.6) });
        self.ops.push(Op::DrawLine {
            line: Line {
                points: vec![
                    LinePoint {
                        p: Point {
                            x: Pt(x1),
                            y: Pt(self.pdf_y(y_haut)),
                        },
                        bezier: false,
                    },
                    LinePoint {
                        p: Point {
                            x: Pt(x2),
                            y: Pt(self.pdf_y(y_haut)),
                        },
                        bezier: false,
                    },
                ],
                is_closed: false,
            },
        });
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
