//! Export PDF du CV au design Candilog, calqué sur le template HTML fourni.
//!
//! Document 100 % autonome : polices IBM Plex embarquées, aucune dépendance système.

use crate::core::errors::{AppError, AppResult};
use crate::infrastructure::pdf::page::{
    ensure_inside, Density, LayoutBounds, Margins, A4, DENSITY_PROFILES, MIN_BODY_FONT_PT,
};
use printpdf::{
    Color, FontId, LinePoint, Mm, Op, PaintMode, ParsedFont, PdfDocument, PdfFontHandle, PdfPage,
    PdfSaveOptions, Point, Polygon, PolygonRing, Pt, Rgb, TextItem, WindingOrder,
};
use std::path::Path;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Clone, Copy)]
enum SansWeight {
    Regular,
    Medium,
    SemiBold,
}

#[derive(Clone, Copy)]
enum MonoWeight {
    Regular,
    Medium,
}

#[derive(Clone, Copy)]
enum FontFace {
    Sans(SansWeight),
    Mono(MonoWeight),
}

struct Fonts {
    sans_regular: ParsedFont,
    sans_medium: ParsedFont,
    sans_semibold: ParsedFont,
    mono_regular: ParsedFont,
    mono_medium: ParsedFont,
    sans_regular_id: FontId,
    sans_medium_id: FontId,
    sans_semibold_id: FontId,
    mono_regular_id: FontId,
    mono_medium_id: FontId,
}

impl Fonts {
    fn source(&self, face: FontFace) -> &ParsedFont {
        match face {
            FontFace::Sans(SansWeight::Regular) => &self.sans_regular,
            FontFace::Sans(SansWeight::Medium) => &self.sans_medium,
            FontFace::Sans(SansWeight::SemiBold) => &self.sans_semibold,
            FontFace::Mono(MonoWeight::Regular) => &self.mono_regular,
            FontFace::Mono(MonoWeight::Medium) => &self.mono_medium,
        }
    }

    fn id(&self, face: FontFace) -> &FontId {
        match face {
            FontFace::Sans(SansWeight::Regular) => &self.sans_regular_id,
            FontFace::Sans(SansWeight::Medium) => &self.sans_medium_id,
            FontFace::Sans(SansWeight::SemiBold) => &self.sans_semibold_id,
            FontFace::Mono(MonoWeight::Regular) => &self.mono_regular_id,
            FontFace::Mono(MonoWeight::Medium) => &self.mono_medium_id,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ResumeExperience {
    pub title: String,
    pub company: String,
    pub location: Option<String>,
    pub period: String,
    pub bullets: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ResumeProject {
    pub name: String,
    pub meta: String,
    pub url: Option<String>,
    pub bullets: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ResumeEducation {
    pub degree: String,
    pub school: String,
    pub location: Option<String>,
    pub period: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ResumeLanguage {
    pub name: String,
    pub level: String,
}

#[derive(Debug, Clone, Default)]
pub struct ResumeCertification {
    pub name: String,
    pub issuer: Option<String>,
    pub date: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ResumeSkillGroup {
    pub name: String,
    pub items: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ResumePdf {
    pub name: String,
    pub subtitle: String,
    pub headline: Option<String>,
    pub phone: Option<String>,
    pub email: String,
    pub city: Option<String>,
    pub linkedin: Option<String>,
    pub website: Option<String>,
    pub github: Option<String>,
    pub extra: Vec<String>,
    pub profile: String,
    pub skill_groups: Vec<ResumeSkillGroup>,
    pub experiences: Vec<ResumeExperience>,
    pub projects: Vec<ResumeProject>,
    pub education: Vec<ResumeEducation>,
    pub certifications: Vec<ResumeCertification>,
    pub languages: Vec<ResumeLanguage>,
}

const MM: f32 = 72.0 / 25.4;
const PAGE_W: f32 = A4.width_pt;
const PAGE_H: f32 = A4.height_pt;
const MARGIN_LEFT: f32 = 16.0 * MM;
const MARGIN_RIGHT: f32 = 16.0 * MM;
const MARGIN_TOP: f32 = 14.0 * MM;
const MARGIN_BOTTOM: f32 = 15.0 * MM;
const LABEL_W: f32 = pt(104.0);
const SECTION_GAP: f32 = pt(18.0);
const CONTENT_X: f32 = MARGIN_LEFT + LABEL_W + SECTION_GAP;
const CONTENT_W: f32 = PAGE_W - MARGIN_RIGHT - CONTENT_X;
const HEADER_W: f32 = PAGE_W - MARGIN_LEFT - MARGIN_RIGHT;
const PX: f32 = 0.75;
const ASCENT: f32 = 0.8;

const INK: (f32, f32, f32) = (20.0, 22.0, 27.0);
const BODY: (f32, f32, f32) = (58.0, 63.0, 76.0);
const MUTED: (f32, f32, f32) = (69.0, 74.0, 87.0);
const SUBTLE: (f32, f32, f32) = (118.0, 124.0, 139.0);
const ACCENT: (f32, f32, f32) = (63.0, 77.0, 204.0);
const ACCENT_SOFT: (f32, f32, f32) = (154.0, 163.0, 236.0);
const CHIP_BG: (f32, f32, f32) = (243.0, 244.0, 249.0);
const CHIP_TEXT: (f32, f32, f32) = (52.0, 58.0, 71.0);
const COMPANY: (f32, f32, f32) = (51.0, 56.0, 69.0);
const HEADLINE: (f32, f32, f32) = (74.0, 80.0, 96.0);
const GROUP_TITLE: (f32, f32, f32) = (35.0, 38.0, 47.0);
const CONTACT: (f32, f32, f32) = (92.0, 98.0, 111.0);

const fn pt(px: f32) -> f32 {
    px * PX
}

fn rgb(r: f32, g: f32, b: f32) -> Color {
    Color::Rgb(Rgb {
        r: r / 255.0,
        g: g / 255.0,
        b: b / 255.0,
        icc_profile: None,
    })
}

fn page_margins() -> Margins {
    Margins {
        top: MARGIN_TOP,
        right: MARGIN_RIGHT,
        bottom: MARGIN_BOTTOM,
        left: MARGIN_LEFT,
    }
}

impl ResumePdf {
    /// Exporte le CV dans un PDF A4 autonome.
    ///
    /// # Errors
    /// Retourne une erreur si une police embarquée ne peut pas être décodée,
    /// ou si le document ne peut pas être enregistré.
    pub fn render_pdf(&self, path: &Path) -> AppResult<()> {
        std::fs::write(path, self.render_bytes()?)
            .map_err(|error| AppError::Database(format!("Impossible d'exporter le PDF : {error}")))
    }

    /// Produit les octets après validation complète de la page.
    ///
    /// # Errors
    /// Refuse un contenu qui dépasse la page A4.
    pub fn render_bytes(&self) -> AppResult<Vec<u8>> {
        for density in DENSITY_PROFILES {
            if let Some(bytes) = self.render_density(density)? {
                return Ok(bytes);
            }
        }
        Err(AppError::Validation(
            "Le CV ne tient pas sur une page A4. Raccourcissez son contenu avant l'export.".into(),
        ))
    }

    fn render_density(&self, density: Density) -> AppResult<Option<Vec<u8>>> {
        let mut avertissements = Vec::new();
        let mut document = PdfDocument::new("CV Candilog");
        let fonts = load_fonts(&mut document)?;

        let mut plan = Plan {
            ops: Vec::new(),
            fonts: &fonts,
            y: MARGIN_TOP,
            density,
            bounds: LayoutBounds::default(),
            overflow: false,
        };

        plan.entete(self);
        plan.section_profile(self);
        plan.section_experiences(self);
        plan.section_projects(self);
        plan.section_skills(self);
        plan.section_education(self);
        plan.section_certifications(self);
        plan.section_languages(self);

        if plan.overflow || ensure_inside(plan.bounds, page_margins(), "overflow").is_err() {
            return Ok(None);
        }

        let page = PdfPage::new(Mm(A4.width_mm), Mm(A4.height_mm), plan.ops);
        Ok(Some(
            document
                .with_pages(vec![page])
                .save(&PdfSaveOptions::default(), &mut avertissements),
        ))
    }
}

fn load_fonts(document: &mut PdfDocument) -> AppResult<Fonts> {
    let decodage = |octets: &[u8]| -> AppResult<ParsedFont> {
        ParsedFont::from_bytes(octets, 0, &mut Vec::new())
            .ok_or_else(|| AppError::Serialization("Police CV illisible".into()))
    };
    let sans_regular = decodage(include_bytes!(
        "../../../assets/fonts/ibm-plex/IBMPlexSans-Regular.ttf"
    ))?;
    let sans_medium = decodage(include_bytes!(
        "../../../assets/fonts/ibm-plex/IBMPlexSans-Medium.ttf"
    ))?;
    let sans_semibold = decodage(include_bytes!(
        "../../../assets/fonts/ibm-plex/IBMPlexSans-SemiBold.ttf"
    ))?;
    let mono_regular = decodage(include_bytes!(
        "../../../assets/fonts/ibm-plex/IBMPlexMono-Regular.ttf"
    ))?;
    let mono_medium = decodage(include_bytes!(
        "../../../assets/fonts/ibm-plex/IBMPlexMono-Medium.ttf"
    ))?;
    Ok(Fonts {
        sans_regular_id: document.add_font(&sans_regular),
        sans_medium_id: document.add_font(&sans_medium),
        sans_semibold_id: document.add_font(&sans_semibold),
        mono_regular_id: document.add_font(&mono_regular),
        mono_medium_id: document.add_font(&mono_medium),
        sans_regular,
        sans_medium,
        sans_semibold,
        mono_regular,
        mono_medium,
    })
}

struct StyleParagraphe {
    x: f32,
    face: FontFace,
    size: f32,
    couleur: Color,
    interligne: f32,
    largeur_max: f32,
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
    fn font_size(&self, size: f32) -> f32 {
        (size * self.density.font_scale).max(MIN_BODY_FONT_PT.min(size))
    }

    fn spacing(&self, value: f32) -> f32 {
        value * self.density.spacing_scale
    }

    fn pdf_y(&self, y_haut: f32) -> f32 {
        PAGE_H - y_haut
    }

    fn avance(&mut self, distance: f32) {
        self.y += self.spacing(distance);
        self.bounds.max_y = self.bounds.max_y.max(self.y);
        if self.y > PAGE_H - MARGIN_BOTTOM {
            self.overflow = true;
        }
    }

    fn register(&mut self, max_x: f32, max_y: f32) {
        self.bounds.max_x = self.bounds.max_x.max(max_x);
        self.bounds.max_y = self.bounds.max_y.max(max_y);
        if max_x > PAGE_W - MARGIN_RIGHT || max_y > PAGE_H - MARGIN_BOTTOM {
            self.overflow = true;
        }
    }

    fn text(
        &mut self,
        x: f32,
        ligne_de_base_haut: f32,
        face: FontFace,
        size: f32,
        couleur: Color,
        value: &str,
    ) {
        if self.overflow || value.is_empty() {
            return;
        }
        let value = &self.assainir(face, value);
        let size = self.font_size(size);
        let max_x = x + self.largeur_text_actual(face, size, value);
        let max_y = ligne_de_base_haut + size * 0.25;
        self.register(max_x, max_y);
        if self.overflow {
            return;
        }
        self.ops.push(Op::StartTextSection);
        self.ops.push(Op::SetFont {
            font: PdfFontHandle::External(self.fonts.id(face).clone()),
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

    fn largeur_text(&self, face: FontFace, size: f32, value: &str) -> f32 {
        self.largeur_text_actual(face, size, &self.assainir(face, value))
    }

    fn largeur_text_actual(&self, face: FontFace, size: f32, value: &str) -> f32 {
        let font = self.fonts.source(face);
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

    /// Remplace par une espace tout caractère absent de la police.
    ///
    /// Un texte collé depuis le web amène des espaces insécables étroites et autres
    /// signes que les IBM Plex embarquées ne portent pas : sans cela ils sortaient en
    /// rectangle vide dans le PDF, alors que l'aperçu HTML retombait sur une autre police.
    fn assainir(&self, face: FontFace, value: &str) -> String {
        let font = self.fonts.source(face);
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

    fn decouper(&self, face: FontFace, size: f32, value: &str, largeur_max: f32) -> Vec<String> {
        let mut rows = Vec::new();
        let mut courante = String::new();
        for mot in value.split_whitespace() {
            for fragment in self.decouper_token(face, size, mot, largeur_max) {
                let candidate = if courante.is_empty() {
                    fragment.clone()
                } else {
                    format!("{courante} {fragment}")
                };
                if self.largeur_text(face, size, &candidate) <= largeur_max {
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
        face: FontFace,
        size: f32,
        token: &str,
        largeur_max: f32,
    ) -> Vec<String> {
        if self.largeur_text(face, size, token) <= largeur_max {
            return vec![token.to_owned()];
        }
        let mut fragments = Vec::new();
        let mut current = String::new();
        for grapheme in token.graphemes(true) {
            let candidate = format!("{current}{grapheme}");
            if !current.is_empty() && self.largeur_text(face, size, &candidate) > largeur_max {
                fragments.push(std::mem::take(&mut current));
            }
            current.push_str(grapheme);
        }
        if !current.is_empty() {
            fragments.push(current);
        }
        fragments
    }

    fn paragraphe(&mut self, style: StyleParagraphe, value: &str) -> f32 {
        let mut y = self.y;
        let actual_size = self.font_size(style.size);
        let actual_line_height = self.spacing(style.interligne).max(actual_size * 1.1);
        for row in self.decouper(style.face, style.size, value, style.largeur_max) {
            if y + actual_line_height > PAGE_H - MARGIN_BOTTOM {
                self.overflow = true;
                break;
            }
            self.text(
                style.x,
                y + ASCENT * actual_size,
                style.face,
                style.size,
                style.couleur.clone(),
                &row,
            );
            y += actual_line_height;
        }
        let consommee = y - self.y;
        self.y = y;
        self.bounds.max_y = self.bounds.max_y.max(self.y);
        consommee
    }

    fn chip(&mut self, x: f32, label: &str) -> f32 {
        let padding_x = pt(8.0);
        let padding_y = pt(2.5);
        let size = pt(10.4);
        let largeur =
            self.largeur_text(FontFace::Sans(SansWeight::Regular), size, label) + 2.0 * padding_x;
        let hauteur = self.font_size(size) + 2.0 * padding_y;
        self.rect_arrondi(
            x,
            self.y,
            largeur,
            hauteur,
            pt(2.0),
            rgb(CHIP_BG.0, CHIP_BG.1, CHIP_BG.2),
        );
        self.text(
            x + padding_x,
            self.y + padding_y + ASCENT * self.font_size(size),
            FontFace::Sans(SansWeight::Regular),
            size,
            rgb(CHIP_TEXT.0, CHIP_TEXT.1, CHIP_TEXT.2),
            label,
        );
        largeur
    }

    fn rect_arrondi(
        &mut self,
        x: f32,
        y_haut: f32,
        largeur: f32,
        hauteur: f32,
        rayon: f32,
        couleur: Color,
    ) {
        if self.overflow {
            return;
        }
        let hauteur = self.spacing(hauteur);
        let rayon = rayon.min(largeur / 2.0).min(hauteur / 2.0);
        self.register(x + largeur, y_haut + hauteur);
        if self.overflow {
            return;
        }
        let point = |px: f32, py: f32, bezier: bool| LinePoint {
            p: Point {
                x: Pt(px),
                y: Pt(PAGE_H - py),
            },
            bezier,
        };
        let gauche = x;
        let droite = x + largeur;
        let haut = y_haut;
        let bas = y_haut + hauteur;
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

    fn puce(&mut self, x: f32, value: &str) {
        let size = pt(11.3);
        let marque_y = self.y + ASCENT * self.font_size(size) - pt(1.0);
        self.rect_arrondi(
            x,
            marque_y,
            pt(3.0),
            pt(3.0),
            pt(1.0),
            rgb(ACCENT_SOFT.0, ACCENT_SOFT.1, ACCENT_SOFT.2),
        );
        self.paragraphe(
            StyleParagraphe {
                x: x + pt(11.0),
                face: FontFace::Sans(SansWeight::Regular),
                size,
                couleur: rgb(BODY.0, BODY.1, BODY.2),
                interligne: pt(11.3) * 1.45,
                largeur_max: CONTENT_W - pt(11.0),
            },
            value,
        );
    }

    fn section_label(&mut self, title: &str) -> f32 {
        let y_label = self.y + self.spacing(pt(2.0));
        let face = FontFace::Mono(MonoWeight::Medium);
        let size = pt(9.2);
        let actual_size = self.font_size(size);
        let interligne = self.spacing(pt(9.2 * 1.4)).max(actual_size * 1.1);
        // Le libellé vit dans une colonne de `LABEL_W` : sans repli, un intitulé long
        // débordait sur le contenu placé à sa droite et le surimprimait.
        let mut y = y_label;
        for row in self.decouper(face, size, &title.to_uppercase(), LABEL_W) {
            self.text(
                MARGIN_LEFT,
                y + ASCENT * actual_size,
                face,
                size,
                rgb(ACCENT.0, ACCENT.1, ACCENT.2),
                &row,
            );
            y += interligne;
        }
        y_label
    }

    fn begin_section(&mut self, title: &str) -> f32 {
        self.avance(pt(13.0));
        self.section_label(title)
    }
}

impl Plan<'_> {
    fn entete(&mut self, resume: &ResumePdf) {
        self.y = MARGIN_TOP;

        self.text(
            MARGIN_LEFT,
            self.y + ASCENT * self.font_size(pt(31.0)),
            FontFace::Sans(SansWeight::SemiBold),
            pt(31.0),
            rgb(INK.0, INK.1, INK.2),
            &resume.name,
        );
        self.avance(pt(31.0) * 1.02);

        if !resume.subtitle.trim().is_empty() {
            self.text(
                MARGIN_LEFT,
                self.y + ASCENT * self.font_size(pt(10.4)),
                FontFace::Mono(MonoWeight::Medium),
                pt(10.4),
                rgb(ACCENT.0, ACCENT.1, ACCENT.2),
                &resume.subtitle.to_uppercase(),
            );
            self.avance(pt(10.4) * 1.5);
        }

        if let Some(headline) = resume.headline.as_deref() {
            if !headline.trim().is_empty() {
                self.paragraphe(
                    StyleParagraphe {
                        x: MARGIN_LEFT,
                        face: FontFace::Sans(SansWeight::Regular),
                        size: pt(11.4),
                        couleur: rgb(HEADLINE.0, HEADLINE.1, HEADLINE.2),
                        interligne: pt(11.4) * 1.45,
                        largeur_max: HEADER_W.min(pt(64.0) * 6.0),
                    },
                    headline,
                );
                self.avance(pt(2.0));
            }
        }

        self.contact_row_1(resume);
        self.contact_row_2(resume);
        self.avance(pt(4.0));
    }

    fn contact_row_1(&mut self, resume: &ResumePdf) {
        let mut parts = Vec::new();
        if let Some(city) = resume.city.as_deref() {
            if !city.trim().is_empty() {
                parts.push(city.to_owned());
            }
        }
        if let Some(phone) = resume.phone.as_deref() {
            if !phone.trim().is_empty() {
                parts.push(phone.to_owned());
            }
        }
        if !resume.email.trim().is_empty() {
            parts.push(resume.email.clone());
        }
        self.contact_line(&parts);
    }

    fn contact_row_2(&mut self, resume: &ResumePdf) {
        let mut parts = Vec::new();
        if let Some(site) = resume.website.as_deref() {
            if !site.trim().is_empty() {
                parts.push(site.to_owned());
            }
        }
        if let Some(linkedin) = resume.linkedin.as_deref() {
            if !linkedin.trim().is_empty() {
                parts.push(linkedin.to_owned());
            }
        }
        if let Some(github) = resume.github.as_deref() {
            if !github.trim().is_empty() {
                parts.push(github.to_owned());
            }
        }
        parts.extend(
            resume
                .extra
                .iter()
                .filter(|value| !value.trim().is_empty())
                .cloned(),
        );
        self.contact_line(&parts);
    }

    fn contact_line(&mut self, parts: &[String]) {
        if parts.is_empty() {
            return;
        }
        let size = pt(10.1);
        let mut x = MARGIN_LEFT;
        let gap = pt(18.0);
        for (index, part) in parts.iter().enumerate() {
            if index > 0 {
                x += gap;
            }
            self.text(
                x,
                self.y + ASCENT * self.font_size(size),
                FontFace::Mono(MonoWeight::Regular),
                size,
                rgb(CONTACT.0, CONTACT.1, CONTACT.2),
                part,
            );
            x += self.largeur_text(
                FontFace::Mono(MonoWeight::Regular),
                self.font_size(size),
                part,
            );
        }
        self.avance(size * 1.45);
    }

    fn section_profile(&mut self, resume: &ResumePdf) {
        if resume.profile.trim().is_empty() {
            return;
        }
        let y_start = self.begin_section("Profil");
        self.y = y_start;
        self.paragraphe(
            StyleParagraphe {
                x: CONTENT_X,
                face: FontFace::Sans(SansWeight::Regular),
                size: pt(11.6),
                couleur: rgb(BODY.0, BODY.1, BODY.2),
                interligne: pt(11.6) * 1.52,
                largeur_max: CONTENT_W,
            },
            &resume.profile,
        );
    }

    fn section_experiences(&mut self, resume: &ResumePdf) {
        if resume.experiences.is_empty() {
            return;
        }
        let y_start = self.begin_section("Expériences professionnelles");
        self.y = y_start;
        for (index, experience) in resume.experiences.iter().enumerate() {
            if index > 0 {
                self.avance(pt(9.0));
            }
            self.experience_item(experience);
        }
    }

    fn experience_item(&mut self, experience: &ResumeExperience) {
        let title_size = pt(12.6);
        let period_size = pt(9.5);
        self.text(
            CONTENT_X,
            self.y + ASCENT * self.font_size(title_size),
            FontFace::Sans(SansWeight::SemiBold),
            title_size,
            rgb(INK.0, INK.1, INK.2),
            &experience.title,
        );
        if !experience.period.trim().is_empty() {
            let period_x = CONTENT_X + CONTENT_W
                - self.largeur_text(
                    FontFace::Mono(MonoWeight::Regular),
                    self.font_size(period_size),
                    &experience.period,
                );
            self.text(
                period_x,
                self.y + ASCENT * self.font_size(period_size),
                FontFace::Mono(MonoWeight::Regular),
                period_size,
                rgb(SUBTLE.0, SUBTLE.1, SUBTLE.2),
                &experience.period,
            );
        }
        self.avance(title_size * 1.35);

        let mut company_line = experience.company.clone();
        if let Some(location) = experience.location.as_deref() {
            if !location.trim().is_empty() {
                company_line = format!("{company_line} · {location}");
            }
        }
        self.text(
            CONTENT_X,
            self.y + ASCENT * self.font_size(pt(11.2)),
            FontFace::Sans(SansWeight::Medium),
            pt(11.2),
            rgb(COMPANY.0, COMPANY.1, COMPANY.2),
            &company_line,
        );
        self.avance(pt(11.2) * 1.4);

        for bullet in &experience.bullets {
            self.puce(CONTENT_X, bullet);
        }
    }

    fn section_projects(&mut self, resume: &ResumePdf) {
        if resume.projects.is_empty() {
            return;
        }
        let y_start = self.begin_section("Projets");
        self.y = y_start;
        for (index, project) in resume.projects.iter().enumerate() {
            if index > 0 {
                self.avance(pt(8.0));
            }
            self.project_item(project);
        }
    }

    fn project_item(&mut self, project: &ResumeProject) {
        self.text(
            CONTENT_X,
            self.y + ASCENT * self.font_size(pt(12.2)),
            FontFace::Sans(SansWeight::SemiBold),
            pt(12.2),
            rgb(INK.0, INK.1, INK.2),
            &project.name,
        );
        self.avance(pt(12.2) * 1.35);

        let mut meta_parts = Vec::new();
        if !project.meta.trim().is_empty() {
            meta_parts.push(project.meta.clone());
        }
        if let Some(url) = project.url.as_deref() {
            if !url.trim().is_empty() {
                meta_parts.push(url.to_owned());
            }
        }
        if !meta_parts.is_empty() {
            self.text(
                CONTENT_X,
                self.y + ASCENT * self.font_size(pt(11.2)),
                FontFace::Sans(SansWeight::Regular),
                pt(11.2),
                rgb(MUTED.0, MUTED.1, MUTED.2),
                &meta_parts.join(" · "),
            );
            self.avance(pt(11.2) * 1.4);
        }

        for bullet in &project.bullets {
            self.puce(CONTENT_X, bullet);
        }
    }

    fn section_skills(&mut self, resume: &ResumePdf) {
        let groups: Vec<_> = resume
            .skill_groups
            .iter()
            .filter(|group| group.items.iter().any(|item| !item.trim().is_empty()))
            .collect();
        if groups.is_empty() {
            return;
        }
        let y_start = self.begin_section("Compétences");
        self.y = y_start;
        for (index, group) in groups.iter().enumerate() {
            if index > 0 {
                self.avance(pt(5.0));
            }
            self.text(
                CONTENT_X,
                self.y + ASCENT * self.font_size(pt(11.0)),
                FontFace::Sans(SansWeight::SemiBold),
                pt(11.0),
                rgb(GROUP_TITLE.0, GROUP_TITLE.1, GROUP_TITLE.2),
                &group.name,
            );
            let group_w = self.largeur_text(
                FontFace::Sans(SansWeight::SemiBold),
                self.font_size(pt(11.0)),
                &group.name,
            );
            let chips_x = CONTENT_X + group_w + pt(14.0);
            let mut x = chips_x;
            let mut line_y = self.y;
            for item in group.items.iter().filter(|item| !item.trim().is_empty()) {
                let largeur = self.largeur_text(
                    FontFace::Sans(SansWeight::Regular),
                    self.font_size(pt(10.4)),
                    item,
                ) + 2.0 * pt(8.0);
                if x + largeur > CONTENT_X + CONTENT_W && x > chips_x {
                    line_y += pt(10.4) + pt(3.5);
                    x = chips_x;
                }
                let saved_y = self.y;
                self.y = line_y;
                self.chip(x, item);
                x += largeur + pt(5.0);
                self.y = saved_y;
            }
            self.y = line_y + pt(14.0);
        }
    }

    fn section_education(&mut self, resume: &ResumePdf) {
        if resume.education.is_empty() {
            return;
        }
        let y_start = self.begin_section("Formation");
        self.y = y_start;
        for (index, education) in resume.education.iter().enumerate() {
            if index > 0 {
                self.avance(pt(6.0));
            }
            self.text(
                CONTENT_X,
                self.y + ASCENT * self.font_size(pt(12.0)),
                FontFace::Sans(SansWeight::SemiBold),
                pt(12.0),
                rgb(INK.0, INK.1, INK.2),
                &education.degree,
            );
            if !education.period.trim().is_empty() {
                let period_x = CONTENT_X + CONTENT_W
                    - self.largeur_text(
                        FontFace::Mono(MonoWeight::Regular),
                        self.font_size(pt(9.5)),
                        &education.period,
                    );
                self.text(
                    period_x,
                    self.y + ASCENT * self.font_size(pt(9.5)),
                    FontFace::Mono(MonoWeight::Regular),
                    pt(9.5),
                    rgb(SUBTLE.0, SUBTLE.1, SUBTLE.2),
                    &education.period,
                );
            }
            self.avance(pt(12.0) * 1.35);

            let mut school_line = education.school.clone();
            if let Some(location) = education.location.as_deref() {
                if !location.trim().is_empty() {
                    school_line = format!("{school_line} · {location}");
                }
            }
            self.text(
                CONTENT_X,
                self.y + ASCENT * self.font_size(pt(11.2)),
                FontFace::Sans(SansWeight::Regular),
                pt(11.2),
                rgb(MUTED.0, MUTED.1, MUTED.2),
                &school_line,
            );
            self.avance(pt(11.2) * 1.4);

            if let Some(description) = education.description.as_deref() {
                if !description.trim().is_empty() {
                    self.paragraphe(
                        StyleParagraphe {
                            x: CONTENT_X,
                            face: FontFace::Sans(SansWeight::Regular),
                            size: pt(10.9),
                            couleur: rgb(SUBTLE.0, SUBTLE.1, SUBTLE.2),
                            interligne: pt(10.9) * 1.4,
                            largeur_max: CONTENT_W,
                        },
                        description,
                    );
                }
            }
        }
    }

    fn section_certifications(&mut self, resume: &ResumePdf) {
        if resume.certifications.is_empty() {
            return;
        }
        let y_start = self.begin_section("Certifications");
        self.y = y_start;
        for certification in &resume.certifications {
            let mut line = certification.name.clone();
            if let Some(issuer) = certification.issuer.as_deref() {
                if !issuer.trim().is_empty() {
                    line = format!("{line} · {issuer}");
                }
            }
            if let Some(date) = certification.date.as_deref() {
                if !date.trim().is_empty() {
                    line = format!("{line} · {date}");
                }
            }
            self.paragraphe(
                StyleParagraphe {
                    x: CONTENT_X,
                    face: FontFace::Sans(SansWeight::Regular),
                    size: pt(11.2),
                    couleur: rgb(BODY.0, BODY.1, BODY.2),
                    interligne: pt(11.2) * 1.42,
                    largeur_max: CONTENT_W,
                },
                &line,
            );
        }
    }

    fn section_languages(&mut self, resume: &ResumePdf) {
        if resume.languages.is_empty() {
            return;
        }
        let y_start = self.begin_section("Langues");
        self.y = y_start;
        let mut x = CONTENT_X;
        let gap = pt(22.0);
        let size = pt(11.2);
        for (index, language) in resume.languages.iter().enumerate() {
            let label = format!("{} · {}", language.name, language.level);
            if index > 0 {
                x += gap;
            }
            self.text(
                x,
                self.y + ASCENT * self.font_size(size),
                FontFace::Sans(SansWeight::Regular),
                size,
                rgb(BODY.0, BODY.1, BODY.2),
                &label,
            );
            x += self.largeur_text(
                FontFace::Sans(SansWeight::Regular),
                self.font_size(size),
                &label,
            );
        }
        self.avance(size * 1.42);
    }
}

#[cfg(test)]
#[path = "tests/cv_pdf/mod.rs"]
mod cv_pdf;
