//! Export PDF d'une lettre de motivation, calqué sur le template A4 HTML.
//!
//! Polices IBM Plex embarquées, une page, même géométrie que `LetterPaper`.

use crate::core::errors::{AppError, AppResult};
use crate::core::utils::text::{elider, segments_de_cesure};
use crate::features::documents::domain::{parse_letter, LetterAlign, LetterParagraph, LetterRun};
use crate::infrastructure::pdf::page::{
    ensure_inside, Density, LayoutBounds, Margins, A4, MIN_BODY_FONT_PT,
};
use chrono::{Datelike, Local};
use printpdf::{
    Color, FontId, Line, LinePoint, Op, PaintMode, ParsedFont, PdfDocument, PdfFontHandle, PdfPage,
    PdfSaveOptions, Point, Polygon, PolygonRing, Pt, Rgb, TextItem, WindingOrder,
};
use std::path::Path;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Weight {
    Regular,
    SemiBold,
    Bold,
}

struct Fonts {
    regular: ParsedFont,
    semibold: ParsedFont,
    mono_regular: ParsedFont,
    mono_medium: ParsedFont,
    regular_id: FontId,
    semibold_id: FontId,
    mono_regular_id: FontId,
    mono_medium_id: FontId,
}

impl Fonts {
    fn source(&self, weight: Weight) -> &ParsedFont {
        match weight {
            Weight::Regular => &self.regular,
            Weight::SemiBold | Weight::Bold => &self.semibold,
        }
    }

    fn id(&self, weight: Weight) -> &FontId {
        match weight {
            Weight::Regular => &self.regular_id,
            Weight::SemiBold | Weight::Bold => &self.semibold_id,
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

const MM: f32 = 72.0 / 25.4;
/// Le template A4 est coté en pixels CSS : un pixel vaut trois quarts de point.
const PX: f32 = 0.75;
const ASCENT: f32 = 0.8;
/// `.letter-name` : resserré, et jamais coupé en plein mot.
const TYPO_NOM: Typo = Typo {
    tracking: -0.026,
    coupe_les_mots: false,
};
/// `.letter-role` : capitales très espacées.
const TYPO_ROLE: Typo = Typo {
    tracking: 0.13,
    coupe_les_mots: true,
};
/// `.letter-coord-label` : libellés en capitales espacées.
const TYPO_LIBELLE: Typo = Typo {
    tracking: 0.12,
    coupe_les_mots: true,
};
/// `.letter-attachment` : mention de pièce jointe.
const TYPO_PIECE: Typo = Typo {
    tracking: 0.02,
    coupe_les_mots: true,
};
/// `.letter-headline` : intitulé de candidature, légèrement resserré.
const TYPO_INTITULE: Typo = Typo {
    tracking: -0.012,
    coupe_les_mots: false,
};
/// Réserve sous la ligne de base : une jambe de `p` ou de `g` descend encore sous le texte.
const DESCENT: f32 = 0.25;
const INK: (f32, f32, f32) = (20.0, 22.0, 27.0);
const BODY: (f32, f32, f32) = (58.0, 63.0, 76.0);
const MUTED: (f32, f32, f32) = (74.0, 80.0, 96.0);
const SUBTLE: (f32, f32, f32) = (118.0, 124.0, 139.0);
const FAINT: (f32, f32, f32) = (138.0, 144.0, 160.0);
const ACCENT: (f32, f32, f32) = (63.0, 77.0, 204.0);
const PANEL: (f32, f32, f32) = (244.0, 245.0, 251.0);
const HEADING: (f32, f32, f32) = (35.0, 38.0, 47.0);
const DATE: (f32, f32, f32) = (92.0, 98.0, 111.0);

const LETTER_DENSITY: [Density; 4] = [
    Density {
        font_scale: 1.0,
        spacing_scale: 1.0,
    },
    Density {
        font_scale: 0.98,
        spacing_scale: 0.86,
    },
    Density {
        font_scale: 0.95,
        spacing_scale: 0.74,
    },
    Density {
        font_scale: 0.92,
        spacing_scale: 0.64,
    },
];

fn mm(value: f32) -> f32 {
    value * MM
}

/// Convertit une cote du template (pixels CSS) en points PDF.
const fn pt(px: f32) -> f32 {
    px * PX
}

/// CoverLetter prête à exporter : identité du profil + destinataire + corps.
#[derive(Debug, Clone, Default)]
pub struct CoverLetterPdf {
    pub first_name: String,
    pub last_name: String,
    pub title: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub phone: Option<String>,
    pub email: String,
    pub company: Option<String>,
    pub recipient: Option<String>,
    pub recipient_address: Option<String>,
    pub job_title: Option<String>,
    pub job_reference: Option<String>,
    pub corps: String,
}

impl CoverLetterPdf {
    fn display_name(&self) -> String {
        let complet = format!("{} {}", self.first_name, self.last_name)
            .trim()
            .to_owned();
        if complet.is_empty() {
            "Candilog".into()
        } else {
            complet
        }
    }
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
        for density in LETTER_DENSITY {
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
        let mut document = PdfDocument::new("Lettre Candilog");
        let fonts = load_fonts(&mut document)?;

        let mut plan = Plan {
            ops: Vec::new(),
            fonts: &fonts,
            y: mm(18.0),
            col_x: mm(58.0 + 14.0),
            col_w: mm(210.0 - 58.0 - 14.0 - 18.0),
            density,
            bounds: LayoutBounds::default(),
            overflow: false,
            typo: Typo::default(),
        };
        plan.composer(self);
        let margins = Margins {
            top: mm(18.0),
            right: mm(18.0),
            bottom: mm(16.0),
            left: mm(16.0),
        };
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

fn load_fonts(document: &mut PdfDocument) -> AppResult<Fonts> {
    let decodage = |octets: &[u8]| -> AppResult<ParsedFont> {
        ParsedFont::from_bytes(octets, 0, &mut Vec::new())
            .ok_or_else(|| AppError::Serialization("Police lettre illisible".into()))
    };
    let regular = decodage(include_bytes!(
        "../../../assets/fonts/ibm-plex/IBMPlexSans-Regular.ttf"
    ))?;
    let semibold = decodage(include_bytes!(
        "../../../assets/fonts/ibm-plex/IBMPlexSans-SemiBold.ttf"
    ))?;
    let mono_regular = decodage(include_bytes!(
        "../../../assets/fonts/ibm-plex/IBMPlexMono-Regular.ttf"
    ))?;
    let mono_medium = decodage(include_bytes!(
        "../../../assets/fonts/ibm-plex/IBMPlexMono-Medium.ttf"
    ))?;
    Ok(Fonts {
        regular_id: document.add_font(&regular),
        semibold_id: document.add_font(&semibold),
        mono_regular_id: document.add_font(&mono_regular),
        mono_medium_id: document.add_font(&mono_medium),
        regular,
        semibold,
        mono_regular,
        mono_medium,
    })
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

/// Propriétés du template qui ne se réduisent pas à une taille de police.
#[derive(Clone, Copy)]
struct Typo {
    /// `letter-spacing`, exprimé en em comme dans la feuille de style.
    tracking: f32,
    /// `overflow-wrap: anywhere` : seuls quelques blocs coupent en plein mot.
    coupe_les_mots: bool,
}

impl Default for Typo {
    fn default() -> Self {
        Self {
            tracking: 0.0,
            coupe_les_mots: true,
        }
    }
}

struct Plan<'a> {
    ops: Vec<Op>,
    fonts: &'a Fonts,
    y: f32,
    col_x: f32,
    col_w: f32,
    density: Density,
    bounds: LayoutBounds,
    overflow: bool,
    typo: Typo,
}

impl Plan<'_> {
    fn pdf_y(&self, y_haut: f32) -> f32 {
        A4.height_pt - y_haut
    }

    /// Dessine un bloc avec le trait typographique demandé, puis restaure le précédent.
    fn avec_typo(&mut self, typo: Typo, dessin: impl FnOnce(&mut Self)) {
        let precedent = self.typo;
        self.typo = typo;
        dessin(self);
        self.typo = precedent;
    }

    /// Espacement à ajouter entre deux glyphes, en points, pour une taille donnée.
    fn tracking_pt(&self, size_actual: f32) -> f32 {
        self.typo.tracking * size_actual
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
        let nom = cover_letter.display_name();
        self.fill_rect(
            0.0,
            0.0,
            mm(58.0),
            A4.height_pt,
            rgb(PANEL.0, PANEL.1, PANEL.2),
        );

        self.col_x = mm(16.0);
        self.col_w = mm(58.0) - mm(16.0) - mm(12.0);
        self.y = mm(18.0);
        self.colonne_identite(cover_letter, &nom);

        self.col_x = mm(58.0) + mm(14.0);
        self.col_w = mm(210.0) - mm(58.0) - mm(14.0) - mm(18.0);
        self.y = mm(18.0);
        self.colonne_lettre(cover_letter, &nom);
    }

    fn colonne_identite(&mut self, cover_letter: &CoverLetterPdf, nom: &str) {
        let prenom = cover_letter.first_name.trim();
        let nom_famille = cover_letter.last_name.trim();
        if !prenom.is_empty() && !nom_famille.is_empty() {
            self.avec_typo(TYPO_NOM, |plan| {
                plan.bloc_text(
                    Weight::SemiBold,
                    pt(25.0),
                    rgb(INK.0, INK.1, INK.2),
                    pt(26.5),
                    prenom,
                );
                plan.bloc_text(
                    Weight::SemiBold,
                    pt(25.0),
                    rgb(INK.0, INK.1, INK.2),
                    pt(26.5),
                    nom_famille,
                );
            });
        } else {
            self.avec_typo(TYPO_NOM, |plan| {
                plan.bloc_text(
                    Weight::SemiBold,
                    pt(25.0),
                    rgb(INK.0, INK.1, INK.2),
                    pt(26.5),
                    nom,
                );
            });
        }
        if let Some(title) = cover_letter.title.as_deref() {
            self.avance(pt(6.0));
            self.avec_typo(TYPO_ROLE, |plan| {
                plan.bloc_mono(
                    true,
                    pt(9.6),
                    rgb(ACCENT.0, ACCENT.1, ACCENT.2),
                    pt(14.4),
                    &title.to_uppercase(),
                );
            });
        }
        self.avance(pt(22.0));
        self.coordonnee(
            "Adresse",
            cover_letter.address.as_deref(),
            cover_letter.city.as_deref(),
        );
        self.coordonnee("Téléphone", cover_letter.phone.as_deref(), None);
        self.coordonnee(
            "Courriel",
            {
                let email = cover_letter.email.trim();
                if email.is_empty() {
                    None
                } else {
                    Some(email)
                }
            },
            None,
        );

        // La mention est calée sur la marge basse : la réserve doit valoir la hauteur
        // réelle du bloc, sinon sa dernière ligne franchit la marge et l'export est refusé.
        const PIECE_SIZE: f32 = pt(9.4);
        const PIECE_INTERLIGNE: f32 = pt(14.1);
        let piece = "Pièce jointe :\ncurriculum vitæ";
        let interligne_piece = self
            .spacing(PIECE_INTERLIGNE)
            .max(self.font_size(PIECE_SIZE) * 1.1);
        let hauteur_piece = interligne_piece * piece.lines().count() as f32;
        let y_piece = A4.height_pt - mm(16.0) - hauteur_piece;
        if y_piece > self.y {
            self.y = y_piece;
        }
        self.avec_typo(TYPO_PIECE, |plan| {
            plan.bloc_mono(
                false,
                PIECE_SIZE,
                rgb(SUBTLE.0, SUBTLE.1, SUBTLE.2),
                PIECE_INTERLIGNE,
                piece,
            );
        });
    }

    fn coordonnee(&mut self, label: &str, ligne: Option<&str>, extra: Option<&str>) {
        let principale = ligne.map(str::trim).filter(|value| !value.is_empty());
        let secondaire = extra.map(str::trim).filter(|value| !value.is_empty());
        if principale.is_none() && secondaire.is_none() {
            return;
        }
        self.avec_typo(TYPO_LIBELLE, |plan| {
            plan.bloc_mono(
                true,
                pt(8.6),
                rgb(FAINT.0, FAINT.1, FAINT.2),
                pt(12.9),
                &label.to_uppercase(),
            );
        });
        if let Some(value) = principale {
            self.bloc_text(
                Weight::Regular,
                pt(10.8),
                rgb(MUTED.0, MUTED.1, MUTED.2),
                pt(16.2),
                value,
            );
        }
        if let Some(value) = secondaire {
            self.bloc_text(
                Weight::Regular,
                pt(10.8),
                rgb(MUTED.0, MUTED.1, MUTED.2),
                pt(16.2),
                value,
            );
        }
        self.avance(pt(9.0));
    }

    fn colonne_lettre(&mut self, cover_letter: &CoverLetterPdf, nom: &str) {
        let date = date_du_day();
        if let Some(company) = cover_letter.company.as_deref() {
            self.bloc_text(
                Weight::SemiBold,
                pt(12.2),
                rgb(HEADING.0, HEADING.1, HEADING.2),
                pt(18.3),
                company,
            );
        }
        if let Some(recipient) = cover_letter.recipient.as_deref() {
            self.bloc_text(
                Weight::Regular,
                pt(11.3),
                rgb(MUTED.0, MUTED.1, MUTED.2),
                pt(17.0),
                recipient,
            );
        }
        if let Some(address) = cover_letter.recipient_address.as_deref() {
            self.bloc_text(
                Weight::Regular,
                pt(11.3),
                rgb(MUTED.0, MUTED.1, MUTED.2),
                pt(17.0),
                address,
            );
        }
        let ligne_date = cover_letter
            .city
            .as_deref()
            .map_or_else(|| format!("Le {date}"), |city| format!("{city}, le {date}"));
        self.bloc_mono(
            false,
            pt(10.1),
            rgb(DATE.0, DATE.1, DATE.2),
            pt(15.2),
            &ligne_date,
        );
        self.avance(pt(20.0));
        if let Some(poste) = cover_letter.job_title.as_deref() {
            self.avec_typo(TYPO_INTITULE, |plan| {
                plan.bloc_text(
                    Weight::SemiBold,
                    pt(14.6),
                    rgb(INK.0, INK.1, INK.2),
                    pt(19.7),
                    &format!("Candidature au poste {}", elider("de", poste)),
                );
            });
        }
        if let Some(reference) = cover_letter.job_reference.as_deref() {
            self.bloc_mono(
                false,
                pt(10.0),
                rgb(SUBTLE.0, SUBTLE.1, SUBTLE.2),
                pt(15.0),
                &format!("Référence de l'offre : {reference}"),
            );
        }
        self.avance(pt(20.0));
        for paragraphe in parse_letter(&cover_letter.corps) {
            if paragraphe.runs.is_empty() {
                self.avance(pt(12.0));
                continue;
            }
            self.paragraphe_riche(&paragraphe, rgb(BODY.0, BODY.1, BODY.2));
            self.avance(pt(12.0));
        }

        let taille_signature = self.font_size(pt(13.0));
        // La borne basse mesure la descendante : la réserver ici, sinon la signature
        // dépasse la marge de quelques dixièmes de point et l'export est refusé.
        let y_signature = (A4.height_pt - mm(16.0) - (ASCENT + DESCENT) * taille_signature)
            .max(self.y + self.spacing(pt(8.0)));
        self.y = y_signature;
        let largeur = self.largeur_text(Weight::SemiBold, pt(13.0), nom);
        let x = self.col_x + (self.col_w - largeur).max(0.0);
        self.text(
            x,
            self.y + ASCENT * taille_signature,
            Weight::SemiBold,
            pt(13.0),
            rgb(INK.0, INK.1, INK.2),
            nom,
        );
    }

    fn fill_rect(&mut self, x: f32, y_haut: f32, largeur: f32, hauteur: f32, couleur: Color) {
        let point = |px: f32, py: f32| LinePoint {
            p: Point {
                x: Pt(px),
                y: Pt(self.pdf_y(py)),
            },
            bezier: false,
        };
        let points = vec![
            point(x, y_haut),
            point(x + largeur, y_haut),
            point(x + largeur, y_haut + hauteur),
            point(x, y_haut + hauteur),
        ];
        self.ops.push(Op::SetFillColor { col: couleur });
        self.ops.push(Op::DrawPolygon {
            polygon: Polygon {
                rings: vec![PolygonRing { points }],
                mode: PaintMode::Fill,
                winding_order: WindingOrder::default(),
            },
        });
    }

    fn bloc_mono(&mut self, medium: bool, size: f32, couleur: Color, interligne: f32, value: &str) {
        let actual_size = self.font_size(size);
        let actual_line_height = self.spacing(interligne).max(actual_size * 1.1);
        // Le template rend ces blocs dans une colonne : sans repli, un titre long
        // traversait la page et surimprimait la lettre.
        let lignes: Vec<String> = value
            .lines()
            .flat_map(|row| self.replier_mono(medium, actual_size, row, self.col_w))
            .collect();
        for row in lignes {
            if self.y + actual_line_height > A4.height_pt - mm(16.0) {
                self.overflow = true;
                self.bounds.max_y = self.y + actual_line_height;
                return;
            }
            if !row.is_empty() {
                self.text_mono(
                    self.col_x,
                    self.y + ASCENT * actual_size,
                    medium,
                    size,
                    couleur.clone(),
                    &row,
                );
            }
            self.y += actual_line_height;
            self.bounds.max_y = self.bounds.max_y.max(self.y);
        }
    }

    fn text_mono(
        &mut self,
        x: f32,
        ligne_de_base_haut: f32,
        medium: bool,
        size: f32,
        couleur: Color,
        value: &str,
    ) {
        if self.overflow {
            return;
        }
        let size = self.font_size(size);
        let (font, id) = if medium {
            (&self.fonts.mono_medium, &self.fonts.mono_medium_id)
        } else {
            (&self.fonts.mono_regular, &self.fonts.mono_regular_id)
        };
        let value = &Self::assainir(font, value);
        let largeur = self.largeur_glyphes(font, size, value);
        self.bounds.max_x = self.bounds.max_x.max(x + largeur);
        self.bounds.max_y = self.bounds.max_y.max(ligne_de_base_haut + size * DESCENT);
        self.ops.push(Op::StartTextSection);
        self.ops.push(Op::SetCharacterSpacing {
            multiplier: self.tracking_pt(size),
        });
        self.ops.push(Op::SetFont {
            font: PdfFontHandle::External(id.clone()),
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

    /// Compose un paragraphe mis en forme : gras et souligné par fragment, taille et
    /// alignement par paragraphe.
    ///
    /// La coupure de ligne se fait sur des **mots** et non sur des fragments : un mot dont
    /// une partie seulement est en gras reste insécable, sinon « auto**matique** » se
    /// couperait en deux avec un blanc au milieu.
    fn paragraphe_riche(&mut self, paragraphe: &LetterParagraph, couleur: Color) {
        let size = pt(11.0) * paragraphe.size.scale();
        let interligne = pt(16.5) * paragraphe.size.scale();
        let actual_size = self.font_size(size);
        let hauteur_ligne = self.spacing(interligne).max(actual_size * 1.1);
        let espace = self.largeur_text(Weight::Regular, size, " ");

        for ligne in self.decouper_mots(&paragraphe.runs, size) {
            if self.y + hauteur_ligne > A4.height_pt - mm(16.0) {
                self.overflow = true;
                self.bounds.max_y = self.y + hauteur_ligne;
                return;
            }
            let largeur: f32 = ligne.iter().map(|mot| mot.largeur).sum::<f32>()
                + espace * (ligne.len().saturating_sub(1)) as f32;
            let mut x = match paragraphe.align {
                LetterAlign::Left => self.col_x,
                LetterAlign::Center => self.col_x + (self.col_w - largeur).max(0.0) / 2.0,
                LetterAlign::Right => self.col_x + (self.col_w - largeur).max(0.0),
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
                for fragment in self.decouper_token(weight, size, token, self.col_w) {
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
            if !courante.is_empty() && largeur + ajout > self.col_w {
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
                self.decouper(weight, size, row_brute, self.col_w)
            };
            for row in rows {
                if self.y + actual_line_height > A4.height_pt - mm(16.0) {
                    self.overflow = true;
                    self.bounds.max_y = self.y + actual_line_height;
                    return;
                }
                if !row.is_empty() {
                    self.text(
                        self.col_x,
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
        let value = &Self::assainir(self.fonts.source(weight), value);
        self.bounds.max_x = self
            .bounds
            .max_x
            .max(x + self.largeur_text_actual(weight, size, value));
        self.bounds.max_y = self.bounds.max_y.max(ligne_de_base_haut + size * DESCENT);
        self.ops.push(Op::StartTextSection);
        self.ops.push(Op::SetCharacterSpacing {
            multiplier: self.tracking_pt(size),
        });
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
        self.largeur_glyphes(font, size, value)
    }

    /// Remplace par une espace tout caractère absent de la police.
    ///
    /// Un retour à la ligne saisi dans un champ mono-ligne, ou un signe hors de la
    /// couverture des IBM Plex, sortait sinon en rectangle vide dans le PDF.
    fn assainir(font: &ParsedFont, value: &str) -> String {
        value
            .chars()
            .map(|caractere| {
                if font.lookup_glyph_index(caractere as u32).is_some() {
                    caractere
                } else {
                    ' '
                }
            })
            .collect()
    }

    /// Largeur d'un texte déjà mis à l'échelle, espacement des lettres compris.
    fn largeur_glyphes(&self, font: &ParsedFont, size: f32, value: &str) -> f32 {
        let echelle = size / f32::from(font.units_per_em);
        let glyphes: f32 = value
            .chars()
            .map(|caractere| {
                font.lookup_glyph_index(caractere as u32)
                    .and_then(|glyphe| font.get_glyph_width(glyphe))
                    .map_or(0.0, |largeur| largeur as f32 * echelle)
            })
            .sum();
        // `Tc` s'applique après chaque glyphe, y compris le dernier, comme dans le PDF.
        glyphes + self.tracking_pt(size) * value.chars().count() as f32
    }

    /// Largeur d'un texte en chasse fixe, à la taille finale.
    fn largeur_mono_actual(&self, medium: bool, size: f32, value: &str) -> f32 {
        let font = if medium {
            &self.fonts.mono_medium
        } else {
            &self.fonts.mono_regular
        };
        self.largeur_glyphes(font, size, value)
    }

    fn decouper(&self, weight: Weight, size: f32, value: &str, largeur_max: f32) -> Vec<String> {
        let mut rows = Vec::new();
        let mut courante = String::new();
        for mot in value.split_whitespace() {
            // `suite` distingue un fragment qui prolonge le mot précédent — après une coupe
            // au trait d'union — d'un mot voisin : recoller les deux par une espace donnait
            // « Maréchal- de- Lattre » sur la ligne où le nom tenait pourtant entier.
            for (suite, fragment) in self
                .fragments_du_mot(weight, size, mot, largeur_max)
                .into_iter()
                .enumerate()
            {
                let candidate = if courante.is_empty() {
                    fragment.clone()
                } else if suite > 0 {
                    format!("{courante}{fragment}")
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

    /// Replie un texte en chasse fixe dans une largeur, à la taille finale.
    fn replier_mono(
        &self,
        medium: bool,
        size_actual: f32,
        value: &str,
        largeur_max: f32,
    ) -> Vec<String> {
        if value.is_empty() || self.largeur_mono_actual(medium, size_actual, value) <= largeur_max {
            return vec![value.to_owned()];
        }
        let mut rows = Vec::new();
        let mut courante = String::new();
        for mot in value.split_whitespace() {
            for (suite, fragment) in self
                .fragments_du_mot_mono(medium, size_actual, mot, largeur_max)
                .into_iter()
                .enumerate()
            {
                let candidate = if courante.is_empty() {
                    fragment.clone()
                } else if suite > 0 {
                    format!("{courante}{fragment}")
                } else {
                    format!("{courante} {fragment}")
                };
                if self.largeur_mono_actual(medium, size_actual, &candidate) <= largeur_max {
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

    /// Coupe un mot en chasse fixe plus large que la colonne, graphème par graphème.
    fn fragments_du_mot_mono(
        &self,
        medium: bool,
        size_actual: f32,
        token: &str,
        largeur_max: f32,
    ) -> Vec<String> {
        if self.largeur_mono_actual(medium, size_actual, token) <= largeur_max {
            return vec![token.to_owned()];
        }
        let segments = segments_de_cesure(token);
        if segments.len() > 1 {
            return segments
                .into_iter()
                .flat_map(|segment| {
                    self.fragments_du_mot_mono(medium, size_actual, segment, largeur_max)
                })
                .collect();
        }
        let mut fragments = Vec::new();
        let mut current = String::new();
        for grapheme in token.graphemes(true) {
            let candidate = format!("{current}{grapheme}");
            if !current.is_empty()
                && self.largeur_mono_actual(medium, size_actual, &candidate) > largeur_max
            {
                fragments.push(std::mem::take(&mut current));
            }
            current.push_str(grapheme);
        }
        if !current.is_empty() {
            fragments.push(current);
        }
        fragments
    }

    /// Fragments d'un mot trop long : un seul si le template interdit de le couper.
    fn fragments_du_mot(
        &self,
        weight: Weight,
        size: f32,
        token: &str,
        largeur_max: f32,
    ) -> Vec<String> {
        // `coupe_les_mots` dit s'il est *souhaitable* de couper, pas s'il est permis de
        // sortir du cadre : un mot plus large que sa colonne — un patronyme composé dans la
        // colonne d'identité — doit être coupé, faute de quoi il déborde sur la lettre.
        if !self.typo.coupe_les_mots && self.largeur_text(weight, size, token) <= largeur_max {
            return vec![token.to_owned()];
        }
        self.decouper_token(weight, size, token, largeur_max)
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
        // Le trait d'union est la première occasion de césure, comme dans le navigateur.
        let segments = segments_de_cesure(token);
        if segments.len() > 1 {
            return segments
                .into_iter()
                .flat_map(|segment| self.decouper_token(weight, size, segment, largeur_max))
                .collect();
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
