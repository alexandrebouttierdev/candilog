//! Export PDF du CV au design Candilog, calqué sur le template HTML fourni.
//!
//! Document 100 % autonome : polices IBM Plex embarquées, aucune dépendance système.

use crate::core::errors::{AppError, AppResult};
use crate::core::utils::text::segments_de_cesure;
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
/// Colonne des étiquettes de section : elle doit porter le plus long libellé du gabarit
/// (« PROFESSIONNELLES ») à sa taille et à son interlettrage réels.
const LABEL_W: f32 = pt(116.0);
const SECTION_GAP: f32 = pt(18.0);
const CONTENT_X: f32 = MARGIN_LEFT + LABEL_W + SECTION_GAP;
const CONTENT_W: f32 = PAGE_W - MARGIN_RIGHT - CONTENT_X;
const HEADER_W: f32 = PAGE_W - MARGIN_LEFT - MARGIN_RIGHT;
const PX: f32 = 0.75;
const ASCENT: f32 = 0.8;
/// Pastilles de compétence : taille de police et rembourrage du gabarit HTML.
const CHIP_SIZE: f32 = pt(10.4);
const CHIP_PADDING_X: f32 = pt(8.0);
const CHIP_PADDING_Y: f32 = pt(2.5);
/// Écart horizontal entre deux pastilles, et écart vertical entre deux rangées.
const CHIP_GAP_X: f32 = pt(5.0);
const CHIP_GAP_Y: f32 = pt(3.5);
/// Interlettrages du gabarit, en cadratins. Le PDF les ignorait : les mêmes libellés
/// sortaient 17 % plus étroits qu'à l'aperçu, et la feuille imprimée n'était plus celle
/// que l'utilisateur avait validée à l'écran.
const TRACKING_NOM: f32 = -0.028;
const TRACKING_SOUS_TITRE: f32 = 0.15;
const TRACKING_LIBELLE: f32 = 0.11;
const TRACKING_PERIODE: f32 = 0.01;
const TRACKING_TITRE: f32 = -0.006;
const TRACKING_GROUPE: f32 = -0.004;

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
        let mut dernier = Debordement::Hauteur;
        for density in DENSITY_PROFILES {
            match self.render_density(density)? {
                Ok(bytes) => return Ok(bytes),
                Err(cause) => dernier = cause,
            }
        }
        // Le message nomme la cause : un CV refusé pour une seule ligne trop large n'est pas
        // trop long, et conseiller de le raccourcir n'y change rien.
        Err(AppError::Validation(match dernier {
            Debordement::Hauteur => {
                "Le CV ne tient pas sur une page A4. Raccourcissez son contenu avant l'export."
                    .into()
            }
            Debordement::Largeur => {
                "Une ligne du CV dépasse la largeur de la page. Raccourcissez l'intitulé, l'entreprise ou le lien concerné avant l'export.".into()
            }
        }))
    }

    fn render_density(&self, density: Density) -> AppResult<Result<Vec<u8>, Debordement>> {
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
            debordement: None,
            tracking: 0.0,
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
            return Ok(Err(plan.debordement.unwrap_or(Debordement::Hauteur)));
        }

        let page = PdfPage::new(Mm(A4.width_mm), Mm(A4.height_mm), plan.ops);
        Ok(Ok(document
            .with_pages(vec![page])
            .save(&PdfSaveOptions::default(), &mut avertissements)))
    }
}

/// Axe par lequel une page a débordé, pour expliquer un refus d'export.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Debordement {
    Largeur,
    Hauteur,
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

/// Style d'un flux d'éléments posés côte à côte, avec repli de ligne.
struct StyleFlux {
    x: f32,
    largeur_max: f32,
    face: FontFace,
    size: f32,
    couleur: Color,
    gap: f32,
    interligne: f32,
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
    /// Axe du premier dépassement rencontré, conservé pour expliquer le refus d'export.
    debordement: Option<Debordement>,
    /// Interlettrage courant, en cadratins, appliqué au dessin comme à la mesure.
    tracking: f32,
}

impl Plan<'_> {
    fn font_size(&self, size: f32) -> f32 {
        (size * self.density.font_scale).max(MIN_BODY_FONT_PT.min(size))
    }

    fn spacing(&self, value: f32) -> f32 {
        value * self.density.spacing_scale
    }

    /// Interligne d'un bloc de texte.
    ///
    /// Il suit la **police**, pas l'espacement : le gabarit fixe `line-height` en multiple
    /// de la taille de caractère (`leading-[1.45]`) et ne fait varier avec la densité que
    /// les écarts entre blocs. En le multipliant par l'échelle d'espacement, l'export
    /// tassait ses lignes de 24 % au palier le plus dense — un CV que l'aperçu déclarait
    /// trop long s'exportait alors sans broncher, dans une mise en page que l'utilisateur
    /// n'avait jamais vue.
    fn interligne(&self, value: f32) -> f32 {
        self.font_size(value)
    }

    fn pdf_y(&self, y_haut: f32) -> f32 {
        PAGE_H - y_haut
    }

    /// Dessine avec l'interlettrage donné, puis rend la valeur précédente.
    fn avec_tracking(&mut self, tracking: f32, dessin: impl FnOnce(&mut Self)) {
        let precedent = std::mem::replace(&mut self.tracking, tracking);
        dessin(self);
        self.tracking = precedent;
    }

    /// Espacement additionnel après chaque glyphe, en points, à la taille finale.
    fn tracking_pt(&self, size_actual: f32) -> f32 {
        self.tracking * size_actual
    }

    /// Avance d'une ligne de texte : l'interligne suit la police (cf. [`Plan::interligne`]).
    fn avance_ligne(&mut self, interligne: f32) {
        self.y += self.interligne(interligne);
        self.bounds.max_y = self.bounds.max_y.max(self.y);
        if self.y > PAGE_H - MARGIN_BOTTOM {
            self.deborde(Debordement::Hauteur);
        }
    }

    /// Avance d'un écart entre blocs : celui-ci suit bien l'échelle d'espacement.
    fn avance(&mut self, distance: f32) {
        self.y += self.spacing(distance);
        self.bounds.max_y = self.bounds.max_y.max(self.y);
        if self.y > PAGE_H - MARGIN_BOTTOM {
            self.deborde(Debordement::Hauteur);
        }
    }

    fn register(&mut self, max_x: f32, max_y: f32) {
        self.bounds.max_x = self.bounds.max_x.max(max_x);
        self.bounds.max_y = self.bounds.max_y.max(max_y);
        if max_x > PAGE_W - MARGIN_RIGHT {
            self.deborde(Debordement::Largeur);
        }
        if max_y > PAGE_H - MARGIN_BOTTOM {
            self.deborde(Debordement::Hauteur);
        }
    }

    fn deborde(&mut self, axe: Debordement) {
        self.overflow = true;
        self.debordement.get_or_insert(axe);
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
        self.ops.push(Op::SetCharacterSpacing {
            multiplier: self.tracking_pt(size),
        });
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
        let glyphes: f32 = value
            .chars()
            .map(|caractere| {
                font.lookup_glyph_index(caractere as u32)
                    .and_then(|glyphe| font.get_glyph_width(glyphe))
                    .map_or(0.0, |largeur| largeur as f32 * echelle)
            })
            .sum();
        // `Tc` s'applique après chaque glyphe, dernier compris, comme dans le PDF.
        glyphes + self.tracking_pt(size) * value.chars().count() as f32
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

    /// Replie une valeur sur la largeur disponible.
    ///
    /// `size` est la taille **avant** densité, comme partout ailleurs : la mesure applique
    /// `font_size` elle-même. Mesurer à la taille brute alors que `text` dessine à la taille
    /// mise à l'échelle rendait chaque ligne plus large que la largeur demandée au palier le
    /// plus aéré (`font_scale` 1.04), et suffisait à faire rejeter ce palier pour débordement.
    fn decouper(&self, face: FontFace, size: f32, value: &str, largeur_max: f32) -> Vec<String> {
        let size = self.font_size(size);
        let mut rows = Vec::new();
        let mut courante = String::new();
        for mot in value.split_whitespace() {
            // `suite` distingue un fragment qui prolonge le mot précédent — après une coupe
            // au trait d'union — d'un mot voisin : recoller les deux par une espace donnait
            // « Maréchal- de- Lattre » sur la ligne où le nom tenait pourtant entier.
            for (suite, fragment) in self
                .decouper_token(face, size, mot, largeur_max)
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
        // Le trait d'union est la première occasion de césure, comme dans le navigateur :
        // « Jean-Baptiste » passe à la ligne après le tiret, jamais au milieu de « Baptiste ».
        let segments = segments_de_cesure(token);
        if segments.len() > 1 {
            return segments
                .into_iter()
                .flat_map(|segment| self.decouper_token(face, size, segment, largeur_max))
                .collect();
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
        let actual_line_height = self.interligne(style.interligne);
        for row in self.decouper(style.face, style.size, value, style.largeur_max) {
            if y + actual_line_height > PAGE_H - MARGIN_BOTTOM {
                self.deborde(Debordement::Hauteur);
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

    /// Hauteur d'une pastille de compétence : celle du texte, plus son rembourrage.
    fn hauteur_chip(&self) -> f32 {
        self.font_size(CHIP_SIZE) + 2.0 * CHIP_PADDING_Y
    }

    /// Largeur d'une pastille, mesurée à la taille réellement dessinée.
    fn largeur_chip(&self, label: &str) -> f32 {
        self.largeur_text(
            FontFace::Sans(SansWeight::Regular),
            self.font_size(CHIP_SIZE),
            label,
        ) + 2.0 * CHIP_PADDING_X
    }

    /// Dispose des valeurs côte à côte et passe à la ligne quand la largeur est atteinte.
    ///
    /// Jumeau du `flex-wrap` du gabarit : les coordonnées et les langues étaient posées sur
    /// une ligne unique, sans repli ni mesure, et une adresse électronique un peu longue
    /// sortait de la marge droite — ce qui faisait rejeter la page entière.
    fn flux_horizontal(&mut self, style: &StyleFlux, parts: &[String]) {
        if parts.is_empty() {
            return;
        }
        let mut x = style.x;
        let mut debut_de_ligne = true;
        for part in parts {
            for fragment in self.decouper(style.face, style.size, part, style.largeur_max) {
                let largeur = self.largeur_text(style.face, self.font_size(style.size), &fragment);
                let depart = if debut_de_ligne { x } else { x + style.gap };
                if !debut_de_ligne && depart + largeur > style.x + style.largeur_max {
                    self.avance_ligne(style.interligne);
                    x = style.x;
                } else {
                    x = depart;
                }
                self.text(
                    x,
                    self.y + ASCENT * self.font_size(style.size),
                    style.face,
                    style.size,
                    style.couleur.clone(),
                    &fragment,
                );
                x += largeur;
                debut_de_ligne = false;
            }
        }
        self.avance_ligne(style.interligne);
    }

    fn chip(&mut self, x: f32, label: &str) -> f32 {
        let largeur = self.largeur_chip(label);
        let hauteur = self.hauteur_chip();
        self.rect_arrondi(
            x,
            self.y,
            largeur,
            hauteur,
            pt(2.0),
            rgb(CHIP_BG.0, CHIP_BG.1, CHIP_BG.2),
        );
        let taille = self.font_size(CHIP_SIZE);
        // Le texte est centré dans la pastille : la caler sur le seul rembourrage haut
        // laissait la jambe des lettres descendantes sortir du fond.
        self.text(
            x + CHIP_PADDING_X,
            self.y + (hauteur - taille) / 2.0 + ASCENT * taille,
            FontFace::Sans(SansWeight::Regular),
            CHIP_SIZE,
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
        // La hauteur reçue est déjà celle voulue : l'échelle de densité est appliquée par
        // l'appelant, sur la taille de police du texte que le rectangle habille. La
        // remultiplier ici étirait la pastille de 35 % au palier le plus aéré — les rangées
        // de compétences se chevauchaient — et la réduisait de 38 % au plus dense, où le
        // texte sortait alors sous son fond.
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
        // Le gabarit pose la pastille à `top: 0.55em` de la ligne et la rend ronde
        // (`rounded-full`). L'export la calait sur la ligne de base et l'arrondissait à
        // moitié : elle sortait carrée et une virgule plus bas qu'à l'aperçu.
        let diametre = pt(3.0);
        let marque_y = self.y + 0.55 * self.font_size(size);
        self.rect_arrondi(
            x,
            marque_y,
            diametre,
            diametre,
            diametre / 2.0,
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
        let interligne = self.interligne(pt(9.2 * 1.4));
        // Le libellé vit dans une colonne de `LABEL_W` : sans repli, un intitulé long
        // débordait sur le contenu placé à sa droite et le surimprimait.
        let mut y = y_label;
        self.avec_tracking(TRACKING_LIBELLE, |plan| {
            for row in plan.decouper(face, size, &title.to_uppercase(), LABEL_W) {
                plan.text(
                    MARGIN_LEFT,
                    y + ASCENT * actual_size,
                    face,
                    size,
                    rgb(ACCENT.0, ACCENT.1, ACCENT.2),
                    &row,
                );
                y += interligne;
            }
        });
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

        // Nom et titre sont repliés sur la largeur de la feuille, comme dans l'aperçu : un
        // nom composé ou un intitulé long tenait sur une seule ligne et sortait de la page.
        self.avec_tracking(TRACKING_NOM, |plan| {
            plan.paragraphe(
                StyleParagraphe {
                    x: MARGIN_LEFT,
                    face: FontFace::Sans(SansWeight::SemiBold),
                    size: pt(31.0),
                    couleur: rgb(INK.0, INK.1, INK.2),
                    interligne: pt(31.0) * 1.02,
                    largeur_max: HEADER_W,
                },
                &resume.name,
            );
        });

        if !resume.subtitle.trim().is_empty() {
            self.avec_tracking(TRACKING_SOUS_TITRE, |plan| {
                plan.paragraphe(
                    StyleParagraphe {
                        x: MARGIN_LEFT,
                        face: FontFace::Mono(MonoWeight::Medium),
                        size: pt(10.4),
                        couleur: rgb(ACCENT.0, ACCENT.1, ACCENT.2),
                        interligne: pt(10.4) * 1.5,
                        largeur_max: HEADER_W,
                    },
                    &resume.subtitle.to_uppercase(),
                );
            });
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
        let size = pt(10.1);
        self.flux_horizontal(
            &StyleFlux {
                x: MARGIN_LEFT,
                largeur_max: HEADER_W,
                face: FontFace::Mono(MonoWeight::Regular),
                size,
                couleur: rgb(CONTACT.0, CONTACT.1, CONTACT.2),
                gap: pt(18.0),
                interligne: size * 1.45,
            },
            parts,
        );
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
        let largeur_titre = self.periode_a_droite(&experience.period, pt(9.5));
        self.avec_tracking(TRACKING_TITRE, |plan| {
            plan.paragraphe(
                StyleParagraphe {
                    x: CONTENT_X,
                    face: FontFace::Sans(SansWeight::SemiBold),
                    size: title_size,
                    couleur: rgb(INK.0, INK.1, INK.2),
                    interligne: title_size * 1.35,
                    largeur_max: largeur_titre,
                },
                &experience.title,
            );
        });

        let mut company_line = experience.company.clone();
        if let Some(location) = experience.location.as_deref() {
            if !location.trim().is_empty() {
                company_line = format!("{company_line} · {location}");
            }
        }
        self.paragraphe(
            StyleParagraphe {
                x: CONTENT_X,
                face: FontFace::Sans(SansWeight::Medium),
                size: pt(11.2),
                couleur: rgb(COMPANY.0, COMPANY.1, COMPANY.2),
                interligne: pt(11.2) * 1.4,
                largeur_max: CONTENT_W,
            },
            &company_line,
        );

        for bullet in &experience.bullets {
            self.puce(CONTENT_X, bullet);
        }
    }

    /// Pose la période à droite de la colonne de contenu et retourne la largeur qui reste
    /// pour l'intitulé, écart compris. Sans cette réserve, un intitulé long venait
    /// s'imprimer sous la période.
    fn periode_a_droite(&mut self, period: &str, size: f32) -> f32 {
        if period.trim().is_empty() {
            return CONTENT_W;
        }
        let face = FontFace::Mono(MonoWeight::Regular);
        let mut largeur = 0.0;
        self.avec_tracking(TRACKING_PERIODE, |plan| {
            largeur = plan.largeur_text(face, plan.font_size(size), period);
            // La période est alignée à droite : `register` ne suit que le bord droit, une
            // période démesurée remonterait donc sans bruit jusque dans la colonne des
            // étiquettes. Passé la moitié de la colonne, elle repart de son bord gauche et
            // le dépassement redevient visible.
            let x = if largeur > CONTENT_W * 0.5 {
                CONTENT_X + CONTENT_W * 0.5
            } else {
                CONTENT_X + CONTENT_W - largeur
            };
            plan.text(
                x,
                plan.y + ASCENT * plan.font_size(size),
                face,
                size,
                rgb(SUBTLE.0, SUBTLE.1, SUBTLE.2),
                period,
            );
        });
        (CONTENT_W - largeur - pt(14.0)).max(CONTENT_W * 0.35)
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
        self.paragraphe(
            StyleParagraphe {
                x: CONTENT_X,
                face: FontFace::Sans(SansWeight::SemiBold),
                size: pt(12.2),
                couleur: rgb(INK.0, INK.1, INK.2),
                interligne: pt(12.2) * 1.35,
                largeur_max: CONTENT_W,
            },
            &project.name,
        );

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
            self.paragraphe(
                StyleParagraphe {
                    x: CONTENT_X,
                    face: FontFace::Sans(SansWeight::Regular),
                    size: pt(11.2),
                    couleur: rgb(MUTED.0, MUTED.1, MUTED.2),
                    interligne: pt(11.2) * 1.4,
                    largeur_max: CONTENT_W,
                },
                &meta_parts.join(" · "),
            );
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
            let mut group_w = 0.0;
            self.avec_tracking(TRACKING_GROUPE, |plan| {
                plan.text(
                    CONTENT_X,
                    plan.y + ASCENT * plan.font_size(pt(11.0)),
                    FontFace::Sans(SansWeight::SemiBold),
                    pt(11.0),
                    rgb(GROUP_TITLE.0, GROUP_TITLE.1, GROUP_TITLE.2),
                    &group.name,
                );
                group_w = plan.largeur_text(
                    FontFace::Sans(SansWeight::SemiBold),
                    plan.font_size(pt(11.0)),
                    &group.name,
                );
            });
            let chips_x = CONTENT_X + group_w + pt(14.0);
            // Une rangée avance de la hauteur réelle d'une pastille, plus l'écart vertical
            // du gabarit. L'avance était figée à `10.4 + 3.5` pt, sans rapport avec la
            // hauteur dessinée : dès que la densité aérait les pastilles, chaque rangée
            // recouvrait la précédente et les fonds se fondaient en un bloc gris.
            let hauteur_rangee = self.hauteur_chip();
            let pas_rangee = hauteur_rangee + self.spacing(CHIP_GAP_Y);
            let mut x = chips_x;
            let mut line_y = self.y;
            for item in group.items.iter().filter(|item| !item.trim().is_empty()) {
                let largeur = self.largeur_chip(item);
                if x + largeur > CONTENT_X + CONTENT_W && x > chips_x {
                    line_y += pas_rangee;
                    x = chips_x;
                }
                let saved_y = self.y;
                self.y = line_y;
                self.chip(x, item);
                x += largeur + CHIP_GAP_X;
                self.y = saved_y;
            }
            self.y = line_y + hauteur_rangee;
            self.bounds.max_y = self.bounds.max_y.max(self.y);
            if self.y > PAGE_H - MARGIN_BOTTOM {
                self.deborde(Debordement::Hauteur);
            }
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
            let largeur_diplome = self.periode_a_droite(&education.period, pt(9.5));
            self.paragraphe(
                StyleParagraphe {
                    x: CONTENT_X,
                    face: FontFace::Sans(SansWeight::SemiBold),
                    size: pt(12.0),
                    couleur: rgb(INK.0, INK.1, INK.2),
                    interligne: pt(12.0) * 1.35,
                    largeur_max: largeur_diplome,
                },
                &education.degree,
            );

            let mut school_line = education.school.clone();
            if let Some(location) = education.location.as_deref() {
                if !location.trim().is_empty() {
                    school_line = format!("{school_line} · {location}");
                }
            }
            self.paragraphe(
                StyleParagraphe {
                    x: CONTENT_X,
                    face: FontFace::Sans(SansWeight::Regular),
                    size: pt(11.2),
                    couleur: rgb(MUTED.0, MUTED.1, MUTED.2),
                    interligne: pt(11.2) * 1.4,
                    largeur_max: CONTENT_W,
                },
                &school_line,
            );

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
        let size = pt(11.2);
        let labels: Vec<String> = resume
            .languages
            .iter()
            .map(|language| format!("{} · {}", language.name, language.level))
            .collect();
        self.flux_horizontal(
            &StyleFlux {
                x: CONTENT_X,
                largeur_max: CONTENT_W,
                face: FontFace::Sans(SansWeight::Regular),
                size,
                couleur: rgb(BODY.0, BODY.1, BODY.2),
                gap: pt(22.0),
                interligne: size * 1.42,
            },
            &labels,
        );
    }
}

#[cfg(test)]
#[path = "tests/cv_pdf/mod.rs"]
mod cv_pdf;
