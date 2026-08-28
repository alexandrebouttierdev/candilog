//! Export PDF d'une lettre de motivation, polices Geist embarquées.

use crate::core::errors::{AppError, AppResult};
use chrono::{Datelike, Local};
use printpdf::{
    Color, FontId, Op, ParsedFont, PdfDocument, PdfFontHandle, PdfPage, PdfSaveOptions, Point, Pt,
    Rgb, TextItem,
};
use std::path::Path;

#[derive(Clone, Copy)]
enum Poids {
    Regular,
    Medium,
    SemiBold,
}

struct Polices {
    regular: ParsedFont,
    medium: ParsedFont,
    semibold: ParsedFont,
    regular_id: FontId,
    medium_id: FontId,
    semibold_id: FontId,
}

impl Polices {
    fn source(&self, poids: Poids) -> &ParsedFont {
        match poids {
            Poids::Regular => &self.regular,
            Poids::Medium => &self.medium,
            Poids::SemiBold => &self.semibold,
        }
    }

    fn identifiant(&self, poids: Poids) -> &FontId {
        match poids {
            Poids::Regular => &self.regular_id,
            Poids::Medium => &self.medium_id,
            Poids::SemiBold => &self.semibold_id,
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
const TEXTE: (f32, f32, f32) = (26.0, 26.0, 26.0);
const SECONDAIRE: (f32, f32, f32) = (91.0, 96.0, 112.0);

/// Lettre prête à exporter : identité du profil + corps généré.
#[derive(Debug, Clone, Default)]
pub struct LettrePdf {
    pub nom: String,
    pub ville: Option<String>,
    pub email: String,
    pub objet: String,
    pub corps: String,
}

impl LettrePdf {
    /// # Errors
    /// Police illisible ou écriture du fichier impossible.
    pub fn render_pdf(&self, path: &Path) -> AppResult<()> {
        let mut avertissements = Vec::new();
        let (regular, medium, semibold) = charger_polices()?;
        let mut document = PdfDocument::new("Lettre Candilog");
        let polices = Polices {
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
            polices: &polices,
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

fn charger_polices() -> AppResult<(ParsedFont, ParsedFont, ParsedFont)> {
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

fn date_du_jour() -> String {
    let maintenant = Local::now();
    let mois = [
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
    let index = maintenant.month0() as usize;
    format!(
        "{} {} {}",
        maintenant.day(),
        mois.get(index).copied().unwrap_or(""),
        maintenant.year()
    )
}

struct Plan<'a> {
    ops: Vec<Op>,
    pages: Vec<Vec<Op>>,
    polices: &'a Polices,
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

    fn composer(&mut self, lettre: &LettrePdf) {
        let nom = if lettre.nom.trim().is_empty() {
            "Candilog"
        } else {
            lettre.nom.trim()
        };
        self.bloc_texte(
            Poids::SemiBold,
            13.0,
            rgb(TEXTE.0, TEXTE.1, TEXTE.2),
            18.0,
            nom,
        );
        if let Some(ville) = lettre.ville.as_deref().filter(|v| !v.trim().is_empty()) {
            self.bloc_texte(
                Poids::Regular,
                10.0,
                rgb(SECONDAIRE.0, SECONDAIRE.1, SECONDAIRE.2),
                14.0,
                ville,
            );
        }
        if !lettre.email.trim().is_empty() {
            self.bloc_texte(
                Poids::Regular,
                10.0,
                rgb(SECONDAIRE.0, SECONDAIRE.1, SECONDAIRE.2),
                14.0,
                lettre.email.trim(),
            );
        }
        self.y += 18.0;
        let lieu = lettre
            .ville
            .as_deref()
            .filter(|v| !v.trim().is_empty())
            .map_or_else(
                || format!("Le {}", date_du_jour()),
                |ville| format!("{ville}, le {}", date_du_jour()),
            );
        self.bloc_texte(
            Poids::Regular,
            10.0,
            rgb(SECONDAIRE.0, SECONDAIRE.1, SECONDAIRE.2),
            16.0,
            &lieu,
        );
        self.y += 10.0;
        self.bloc_texte(
            Poids::Medium,
            11.0,
            rgb(TEXTE.0, TEXTE.1, TEXTE.2),
            18.0,
            &lettre.objet,
        );
        self.y += 8.0;
        for paragraphe in lettre.corps.split("\n\n") {
            let texte = paragraphe.trim();
            if texte.is_empty() {
                continue;
            }
            self.paragraphe(
                Poids::Regular,
                11.0,
                rgb(TEXTE.0, TEXTE.1, TEXTE.2),
                16.5,
                texte,
            );
            self.y += 8.0;
        }
        self.y += 12.0;
        self.bloc_texte(
            Poids::Medium,
            11.0,
            rgb(TEXTE.0, TEXTE.1, TEXTE.2),
            16.0,
            nom,
        );
    }

    fn bloc_texte(
        &mut self,
        poids: Poids,
        taille: f32,
        couleur: Color,
        interligne: f32,
        valeur: &str,
    ) {
        self.paragraphe(poids, taille, couleur, interligne, valeur);
    }

    fn paragraphe(
        &mut self,
        poids: Poids,
        taille: f32,
        couleur: Color,
        interligne: f32,
        valeur: &str,
    ) {
        for ligne_brute in valeur.lines() {
            let lignes = if ligne_brute.trim().is_empty() {
                vec![String::new()]
            } else {
                self.decouper(poids, taille, ligne_brute, CONTENT_W)
            };
            for ligne in lignes {
                self.assurer_place(interligne);
                if !ligne.is_empty() {
                    self.texte(
                        MARGIN,
                        self.y + ASCENT * taille,
                        poids,
                        taille,
                        couleur.clone(),
                        &ligne,
                    );
                }
                self.y += interligne;
            }
        }
    }

    fn texte(
        &mut self,
        x: f32,
        ligne_de_base_haut: f32,
        poids: Poids,
        taille: f32,
        couleur: Color,
        valeur: &str,
    ) {
        self.ops.push(Op::StartTextSection);
        self.ops.push(Op::SetFont {
            font: PdfFontHandle::External(self.polices.identifiant(poids).clone()),
            size: Pt(taille),
        });
        self.ops.push(Op::SetFillColor { col: couleur });
        self.ops.push(Op::SetTextCursor {
            pos: Point {
                x: Pt(x),
                y: Pt(self.pdf_y(ligne_de_base_haut)),
            },
        });
        self.ops.push(Op::ShowText {
            items: vec![TextItem::Text(valeur.to_owned())],
        });
        self.ops.push(Op::EndTextSection);
    }

    fn largeur_texte(&self, poids: Poids, taille: f32, valeur: &str) -> f32 {
        let police = self.polices.source(poids);
        let echelle = taille / f32::from(police.units_per_em);
        valeur
            .chars()
            .map(|caractere| {
                police
                    .lookup_glyph_index(caractere as u32)
                    .and_then(|glyphe| police.get_glyph_width(glyphe))
                    .map_or(0.0, |largeur| largeur as f32 * echelle)
            })
            .sum()
    }

    fn decouper(&self, poids: Poids, taille: f32, valeur: &str, largeur_max: f32) -> Vec<String> {
        let mut lignes = Vec::new();
        let mut courante = String::new();
        for mot in valeur.split_whitespace() {
            let candidate = if courante.is_empty() {
                mot.to_owned()
            } else {
                format!("{courante} {mot}")
            };
            if self.largeur_texte(poids, taille, &candidate) <= largeur_max || courante.is_empty() {
                courante = candidate;
            } else {
                lignes.push(std::mem::take(&mut courante));
                courante = mot.to_owned();
            }
        }
        if !courante.is_empty() {
            lignes.push(courante);
        }
        lignes
    }
}

#[cfg(test)]
#[path = "tests/lettre_pdf/mod.rs"]
mod tests;
