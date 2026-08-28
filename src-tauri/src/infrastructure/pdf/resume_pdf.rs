//! Export PDF du CV au design Candilog, calqué sur `exemple_resume.html`.
//!
//! Minutes 100 % autonome : les polices et les icônes sont embarquées dans le
//! binaire, aucune dépendance système n'est requise côté utilisateur.

use crate::core::errors::{AppError, AppResult};
use printpdf::{
    Color, FontId, Line, LinePoint, Mm, Op, PaintMode, ParsedFont, PdfDocument, PdfFontHandle,
    PdfPage, PdfSaveOptions, Point, Polygon, PolygonRing, Pt, RawImage, Rgb, TextItem,
    WindingOrder, XObjectId, XObjectTransform,
};
use std::path::Path;

/// Graisse de texte, miroir des quatre instances statiques embarquées.
#[derive(Clone, Copy)]
enum Weight {
    Regular,
    Medium,
    SemiBold,
    Bold,
}

/// Fonts embarquées (Geist statique) et leurs identifiants PDF.
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

/// Icônes de contact embarquées (rasterisées depuis le template).
struct Icones {
    phone: XObjectId,
    mail: XObjectId,
    pin: XObjectId,
    linkedin: XObjectId,
    globe: XObjectId,
    briefcase: XObjectId,
}

/// Une expérience professionnelle prête à afficher.
#[derive(Debug, Clone, Default)]
pub struct ResumeExperience {
    pub title: String,
    pub company: String,
    /// Row de méta : lieu · période (peut être vide).
    pub meta: String,
    /// Réalisations, une puce par entrée.
    pub bullets: Vec<String>,
}

/// Un projet technique.
#[derive(Debug, Clone, Default)]
pub struct ResumeProject {
    pub name: String,
    pub meta: String,
    pub bullets: Vec<String>,
}

/// Une formation.
#[derive(Debug, Clone, Default)]
pub struct ResumeEducation {
    pub degree: String,
    pub school: String,
    pub date: String,
}

/// Une langue parlée.
#[derive(Debug, Clone, Default)]
pub struct ResumeLanguage {
    pub name: String,
    pub level: String,
}

/// Modèle de données du CV à exporter.
#[derive(Debug, Clone, Default)]
pub struct ResumePdf {
    pub name: String,
    pub subtitle: String,
    pub phone: Option<String>,
    pub email: String,
    pub city: Option<String>,
    pub linkedin: Option<String>,
    pub website: Option<String>,
    pub profile: String,
    pub skills: Vec<String>,
    pub experiences: Vec<ResumeExperience>,
    pub projects: Vec<ResumeProject>,
    pub education: Vec<ResumeEducation>,
    pub languages: Vec<ResumeLanguage>,
}

// ---------------------------------------------------------------------------
// Palette du template (spec `exemple_resume.html`).
// ---------------------------------------------------------------------------

fn rgb(r: f32, g: f32, b: f32) -> Color {
    Color::Rgb(Rgb {
        r: r / 255.0,
        g: g / 255.0,
        b: b / 255.0,
        icc_profile: None,
    })
}

const ACCENT: (f32, f32, f32) = (0.0, 102.0, 204.0);
const TEXT: (f32, f32, f32) = (26.0, 26.0, 26.0);
const SECONDAIRE: (f32, f32, f32) = (63.0, 63.0, 70.0);
const MUTED: (f32, f32, f32) = (85.0, 85.0, 90.0);
const BORDURE: (f32, f32, f32) = (226.0, 226.0, 229.0);
const CHIP_BG: (f32, f32, f32) = (245.0, 245.0, 247.0);

// ---------------------------------------------------------------------------
// Métriques de page et de typographie (px convertis en points, 1 px = 0,75 pt).
// ---------------------------------------------------------------------------

const PAGE_W: f32 = 595.28;
const PAGE_H: f32 = 841.89;
const PAGE_MARGIN: f32 = 14.17; // @page { margin: 0,5 cm }
const CONTENT_X: f32 = PAGE_MARGIN + 22.4 * PX; // padding horizontal 1,4 rem du template
const CONTENT_W: f32 = PAGE_W - 2.0 * CONTENT_X;

const PX: f32 = 0.75;
const fn pt(px: f32) -> f32 {
    px * PX
}

/// Ascendance typographique approximative, pour poser la ligne de base.
const ASCENT: f32 = 0.8;

impl ResumePdf {
    /// Exporte le CV dans un PDF A4 autonome.
    ///
    /// # Errors
    /// Retourne une erreur si une police ou une icône embarquée ne peut pas
    /// être décodée, ou si le document ne peut pas être enregistré.
    pub fn render_pdf(&self, path: &Path) -> AppResult<()> {
        let mut avertissements = Vec::new();

        let (regular, medium, semibold, bold) = load_fonts()?;
        let mut document = PdfDocument::new("CV Candilog");

        let (regular_id, medium_id, semibold_id, bold_id) = (
            document.add_font(&regular),
            document.add_font(&medium),
            document.add_font(&semibold),
            document.add_font(&bold),
        );
        let fonts = Fonts {
            regular,
            medium,
            semibold,
            bold,
            regular_id,
            medium_id,
            semibold_id,
            bold_id,
        };

        let icones = Icones {
            phone: add_icon(
                &mut document,
                include_bytes!("../../../assets/icons/cv/phone.png"),
            )?,
            mail: add_icon(
                &mut document,
                include_bytes!("../../../assets/icons/cv/mail.png"),
            )?,
            pin: add_icon(
                &mut document,
                include_bytes!("../../../assets/icons/cv/pin.png"),
            )?,
            linkedin: add_icon(
                &mut document,
                include_bytes!("../../../assets/icons/cv/linkedin.png"),
            )?,
            globe: add_icon(
                &mut document,
                include_bytes!("../../../assets/icons/cv/globe.png"),
            )?,
            briefcase: add_icon(
                &mut document,
                include_bytes!("../../../assets/icons/cv/briefcase.png"),
            )?,
        };

        let mut plan = Plan {
            ops: Vec::new(),
            fonts: &fonts,
            icones: &icones,
            y: PAGE_MARGIN,
        };

        plan.entete(self);
        plan.section_profile(self);
        plan.section_skills(self);
        plan.section_experiences(self);
        plan.section_projects(self);
        plan.section_education_languages(self);

        let page = PdfPage::new(Mm(210.0), Mm(297.0), plan.ops);
        let octets = document
            .with_pages(vec![page])
            .save(&PdfSaveOptions::default(), &mut avertissements);
        std::fs::write(path, octets)
            .map_err(|error| AppError::Database(format!("Impossible d'exporter le PDF : {error}")))
    }
}

fn load_fonts() -> AppResult<(ParsedFont, ParsedFont, ParsedFont, ParsedFont)> {
    let decodage = |octets: &[u8]| -> AppResult<ParsedFont> {
        ParsedFont::from_bytes(octets, 0, &mut Vec::new())
            .ok_or_else(|| AppError::Serialization("Police CV illisible".into()))
    };
    Ok((
        decodage(include_bytes!("../../../assets/fonts/Geist-Regular.ttf"))?,
        decodage(include_bytes!("../../../assets/fonts/Geist-Medium.ttf"))?,
        decodage(include_bytes!("../../../assets/fonts/Geist-SemiBold.ttf"))?,
        decodage(include_bytes!("../../../assets/fonts/Geist-Bold.ttf"))?,
    ))
}

fn add_icon(document: &mut PdfDocument, octets: &[u8]) -> AppResult<XObjectId> {
    let image = RawImage::decode_from_bytes(octets, &mut Vec::new())
        .map_err(|error| AppError::Serialization(format!("Icône CV illisible : {error}")))?;
    Ok(document.add_image(&image))
}

// ---------------------------------------------------------------------------
// Tracé.
// ---------------------------------------------------------------------------

/// Tampon d'opérations PDF, avec une origine haut-gauche pour le positionnement.
struct Plan<'a> {
    ops: Vec<Op>,
    fonts: &'a Fonts,
    icones: &'a Icones,
    /// Curseur vertical (haut de la prochaine ligne), en points depuis le haut.
    y: f32,
}

impl Plan<'_> {
    fn pdf_y(&self, y_haut: f32) -> f32 {
        PAGE_H - y_haut
    }

    fn polygone_arrondi(
        &mut self,
        x: f32,
        y_haut: f32,
        largeur: f32,
        hauteur: f32,
        rayon: f32,
        couleur: Color,
    ) {
        let rayon = rayon.min(largeur / 2.0).min(hauteur / 2.0);
        let gauche = x;
        let droite = x + largeur;
        let haut = y_haut;
        let bas = y_haut + hauteur;
        let point = |px: f32, py: f32, bezier: bool| LinePoint {
            p: Point {
                x: Pt(px),
                y: Pt(PAGE_H - py),
            },
            bezier,
        };
        let points = vec![
            point(gauche + rayon, haut, false),
            point(droite - rayon, haut, false),
            point(droite, haut, true),
            point(droite, haut + rayon, false),
            point(droite, bas - rayon, false),
            point(droite, bas, true),
            point(droite - rayon, bas, false),
            point(gauche + rayon, bas, false),
            point(gauche, bas, true),
            point(gauche, bas - rayon, false),
            point(gauche, haut + rayon, false),
            point(gauche, haut, true),
            point(gauche + rayon, haut, false),
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

    fn row_h(&mut self, x1: f32, x2: f32, y_haut: f32, couleur: Color, epaisseur: f32) {
        self.ops.push(Op::SetOutlineColor { col: couleur });
        self.ops.push(Op::SetOutlineThickness { pt: Pt(epaisseur) });
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

    fn icon(&mut self, x: f32, y_haut: f32, size: f32, id: &XObjectId) {
        // Les PNG sont rasterisés en 48 px ; on place au facteur taille/48.
        let echelle = size / 48.0;
        self.ops.push(Op::UseXobject {
            id: id.clone(),
            transform: XObjectTransform {
                translate_x: Some(Pt(x)),
                translate_y: Some(Pt(self.pdf_y(y_haut + size))),
                scale_x: Some(echelle),
                scale_y: Some(echelle),
                dpi: Some(72.0),
                ..XObjectTransform::default()
            },
        });
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

    /// Trace un paragraphe et rend la hauteur consommée.
    #[allow(clippy::too_many_arguments)]
    fn paragraphe(
        &mut self,
        x: f32,
        weight: Weight,
        size: f32,
        couleur: Color,
        interligne: f32,
        largeur_max: f32,
        value: &str,
    ) -> f32 {
        let mut y = self.y;
        for row in self.decouper(weight, size, value, largeur_max) {
            self.text(
                x,
                y + ASCENT * size,
                weight,
                size,
                couleur.clone(),
                &row,
            );
            y += interligne;
        }
        let consommee = y - self.y;
        self.y = y;
        consommee
    }
}

// ---------------------------------------------------------------------------
// Sections.
// ---------------------------------------------------------------------------

impl Plan<'_> {
    fn entete(&mut self, resume: &ResumePdf) {
        let x = CONTENT_X;
        let haut_padding = pt(13.6);
        let bas_padding = pt(12.0);
        self.y = PAGE_MARGIN + haut_padding;

        self.text(
            x,
            self.y + ASCENT * pt(32.0),
            Weight::Bold,
            pt(32.0),
            rgb(TEXT.0, TEXT.1, TEXT.2),
            &resume.name,
        );
        self.y += pt(32.0) * 1.1;

        self.text(
            x,
            self.y + ASCENT * pt(13.12),
            Weight::SemiBold,
            pt(13.12),
            rgb(ACCENT.0, ACCENT.1, ACCENT.2),
            &resume.subtitle,
        );
        self.y += pt(13.12) * 1.4;

        // Row de séparation du header.
        let sep_y = self.y + pt(7.2);
        self.row_h(
            x,
            x + CONTENT_W,
            sep_y,
            rgb(BORDURE.0, BORDURE.1, BORDURE.2),
            1.0,
        );
        self.y = sep_y + pt(5.4);

        // Coordonnées.
        let mut contact_x = x;
        let elements = coordonnees(resume, self.icones);
        for (icon, text) in elements {
            let largeur_element = pt(12.0)
                + pt(4.2)
                + self.largeur_text(Weight::Medium, pt(10.88), &text)
                + pt(14.4);
            if contact_x + largeur_element > x + CONTENT_W && contact_x > x {
                contact_x = x;
                self.y += pt(10.88) + pt(3.6);
            }
            self.icon(contact_x, self.y, pt(12.0), &icon);
            contact_x += pt(12.0) + pt(4.2);
            self.text(
                contact_x,
                self.y + ASCENT * pt(10.88),
                Weight::Medium,
                pt(10.88),
                rgb(SECONDAIRE.0, SECONDAIRE.1, SECONDAIRE.2),
                &text,
            );
            contact_x += self.largeur_text(Weight::Medium, pt(10.88), &text) + pt(14.4);
        }
        self.y += pt(10.88) + bas_padding;
    }

    fn title_section(&mut self, x: f32, title: &str) {
        self.text(
            x,
            self.y + ASCENT * pt(9.92),
            Weight::Bold,
            pt(9.92),
            rgb(ACCENT.0, ACCENT.1, ACCENT.2),
            &title.to_uppercase(),
        );
        self.y += pt(9.92) + pt(4.48);
        let largeur = self.largeur_text(Weight::Bold, pt(9.92), &title.to_uppercase());
        self.row_h(
            x,
            x + largeur,
            self.y,
            rgb(ACCENT.0, ACCENT.1, ACCENT.2),
            1.5,
        );
        self.y += pt(1.5) + pt(4.48);
    }

    fn section_profile(&mut self, resume: &ResumePdf) {
        self.avance(pt(4.0));
        self.title_section(CONTENT_X, "Profile");
        self.paragraphe(
            CONTENT_X,
            Weight::Regular,
            pt(12.16),
            rgb(SECONDAIRE.0, SECONDAIRE.1, SECONDAIRE.2),
            pt(12.16) * 1.45,
            CONTENT_W,
            &resume.profile,
        );
    }

    fn section_skills(&mut self, resume: &ResumePdf) {
        self.avance(pt(10.0));
        self.title_section(CONTENT_X, "Compétences techniques");
        let mut x = CONTENT_X;
        let y_base = self.y;
        for skill in &resume.skills {
            let largeur = self.largeur_text(Weight::Medium, pt(10.56), skill) + 2.0 * pt(6.4);
            if x + largeur > CONTENT_X + CONTENT_W {
                x = CONTENT_X;
                self.y += pt(10.56) + 2.0 * pt(1.92) + pt(2.64);
            }
            self.polygone_arrondi(
                x,
                self.y,
                largeur,
                pt(10.56) + 2.0 * pt(1.92),
                pt(4.0),
                rgb(CHIP_BG.0, CHIP_BG.1, CHIP_BG.2),
            );
            self.text(
                x + pt(6.4),
                self.y + pt(1.92) + ASCENT * pt(10.56),
                Weight::Medium,
                pt(10.56),
                rgb(TEXT.0, TEXT.1, TEXT.2),
                skill,
            );
            x += largeur + pt(2.64);
        }
        if resume.skills.is_empty() {
            self.y = y_base;
        } else {
            self.y += pt(10.56) + 2.0 * pt(1.92) + pt(2.64);
        }
    }

    fn section_experiences(&mut self, resume: &ResumePdf) {
        self.avance(pt(10.0));
        self.title_section(CONTENT_X, "Expérience professionnelle");
        for experience in &resume.experiences {
            self.experience(experience);
            self.avance(pt(6.0));
        }
    }

    fn experience(&mut self, experience: &ResumeExperience) {
        let x = CONTENT_X;
        self.text(
            x,
            self.y + ASCENT * pt(13.12),
            Weight::Bold,
            pt(13.12),
            rgb(TEXT.0, TEXT.1, TEXT.2),
            &experience.title,
        );
        self.y += pt(13.12) * 1.35;

        let mut meta = experience.company.clone();
        if !experience.meta.is_empty() {
            meta = format!("{} · {}", meta, experience.meta);
        }
        let briefcase = self.icones.briefcase.clone();
        self.icon(x, self.y, pt(11.0), &briefcase);
        self.text(
            x + pt(11.0) + pt(3.0),
            self.y + ASCENT * pt(11.2),
            Weight::Regular,
            pt(11.2),
            rgb(SECONDAIRE.0, SECONDAIRE.1, SECONDAIRE.2),
            &meta,
        );
        self.y += pt(11.2) * 1.4;

        for puce in &experience.bullets {
            self.puce(x, puce);
        }
    }

    fn puce(&mut self, x: f32, value: &str) {
        let marque = self.y + ASCENT * pt(11.52);
        self.text(
            x,
            marque,
            Weight::Regular,
            pt(11.52),
            rgb(MUTED.0, MUTED.1, MUTED.2),
            "·",
        );
        let decalage = self.largeur_text(Weight::Regular, pt(11.52), "·") + pt(3.0);
        self.paragraphe(
            x + decalage,
            Weight::Regular,
            pt(11.52),
            rgb(SECONDAIRE.0, SECONDAIRE.1, SECONDAIRE.2),
            pt(11.52) * 1.38,
            CONTENT_W - decalage,
            value,
        );
    }

    fn section_projects(&mut self, resume: &ResumePdf) {
        if resume.projects.is_empty() {
            return;
        }
        self.avance(pt(10.0));
        self.title_section(CONTENT_X, "Projets techniques");
        for project in &resume.projects {
            let x = CONTENT_X;
            self.text(
                x,
                self.y + ASCENT * pt(13.12),
                Weight::Bold,
                pt(13.12),
                rgb(TEXT.0, TEXT.1, TEXT.2),
                &project.name,
            );
            self.y += pt(13.12) * 1.35;
            if !project.meta.is_empty() {
                self.text(
                    x,
                    self.y + ASCENT * pt(11.2),
                    Weight::Regular,
                    pt(11.2),
                    rgb(SECONDAIRE.0, SECONDAIRE.1, SECONDAIRE.2),
                    &project.meta,
                );
                self.y += pt(11.2) * 1.4;
            }
            for puce in &project.bullets {
                self.puce(x, puce);
            }
            self.avance(pt(6.0));
        }
    }

    fn section_education_languages(&mut self, resume: &ResumePdf) {
        self.avance(pt(10.0));
        let x_gauche = CONTENT_X;
        let x_droite = CONTENT_X + CONTENT_W / 2.0 + pt(11.2);
        let y_start = self.y;

        self.title_section(x_gauche, "Education");
        for education in &resume.education {
            self.text(
                x_gauche,
                self.y + ASCENT * pt(12.16),
                Weight::Bold,
                pt(12.16),
                rgb(TEXT.0, TEXT.1, TEXT.2),
                &education.degree,
            );
            self.y += pt(12.16) * 1.4;
            self.text(
                x_gauche,
                self.y + ASCENT * pt(10.88),
                Weight::Regular,
                pt(10.88),
                rgb(SECONDAIRE.0, SECONDAIRE.1, SECONDAIRE.2),
                &education.school,
            );
            self.y += pt(10.88) * 1.4;
            if !education.date.is_empty() {
                self.text(
                    x_gauche,
                    self.y + ASCENT * pt(9.6),
                    Weight::Regular,
                    pt(9.6),
                    rgb(MUTED.0, MUTED.1, MUTED.2),
                    &education.date,
                );
                self.y += pt(9.6) * 1.4;
            }
            self.avance(pt(4.0));
        }
        let end_education = self.y;

        self.y = y_start;
        self.title_section(x_droite, "Disponibilité & langues");
        for language in &resume.languages {
            self.text(
                x_droite,
                self.y + ASCENT * pt(12.16),
                Weight::Bold,
                pt(12.16),
                rgb(TEXT.0, TEXT.1, TEXT.2),
                &format!("{} · {}", language.name, language.level),
            );
            self.y += pt(12.16) * 1.4;
        }
        let end_languages = self.y;

        self.y = end_education.max(end_languages);
    }

    fn avance(&mut self, distance: f32) {
        self.y += distance;
    }
}

/// Construit la liste des coordonnées (icône, texte) du header.
fn coordonnees(resume: &ResumePdf, icones: &Icones) -> Vec<(XObjectId, String)> {
    let mut elements = Vec::new();
    if let Some(phone) = &resume.phone {
        if !phone.trim().is_empty() {
            elements.push((icones.phone.clone(), phone.clone()));
        }
    }
    elements.push((icones.mail.clone(), resume.email.clone()));
    if let Some(city) = &resume.city {
        if !city.trim().is_empty() {
            elements.push((icones.pin.clone(), city.clone()));
        }
    }
    if let Some(linkedin) = &resume.linkedin {
        if !linkedin.trim().is_empty() {
            elements.push((icones.linkedin.clone(), linkedin.clone()));
        }
    }
    if let Some(site) = &resume.website {
        if !site.trim().is_empty() {
            elements.push((icones.globe.clone(), site.clone()));
        }
    }
    elements
}

#[cfg(test)]
#[path = "tests/cv_pdf/mod.rs"]
mod tests;
