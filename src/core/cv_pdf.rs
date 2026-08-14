//! Export PDF du CV au design Candilog, calqué sur `exemple_cv.html`.
//!
//! Rendu 100 % autonome : les polices et les icônes sont embarquées dans le
//! binaire, aucune dépendance système n'est requise côté utilisateur.

use crate::shared::error::{AppError, AppResult};
use printpdf::{
    Color, FontId, Line, LinePoint, Mm, Op, PaintMode, ParsedFont, PdfDocument, PdfFontHandle,
    PdfPage, PdfSaveOptions, Point, Polygon, PolygonRing, Pt, RawImage, Rgb, TextItem,
    WindingOrder, XObjectId, XObjectTransform,
};
use std::path::Path;

/// Graisse de texte, miroir des quatre instances statiques embarquées.
#[derive(Clone, Copy)]
enum Poids {
    Regular,
    Medium,
    SemiBold,
    Bold,
}

/// Polices embarquées (Geist statique) et leurs identifiants PDF.
struct Polices {
    regular: ParsedFont,
    medium: ParsedFont,
    semibold: ParsedFont,
    bold: ParsedFont,
    regular_id: FontId,
    medium_id: FontId,
    semibold_id: FontId,
    bold_id: FontId,
}

impl Polices {
    fn source(&self, poids: Poids) -> &ParsedFont {
        match poids {
            Poids::Regular => &self.regular,
            Poids::Medium => &self.medium,
            Poids::SemiBold => &self.semibold,
            Poids::Bold => &self.bold,
        }
    }

    fn identifiant(&self, poids: Poids) -> &FontId {
        match poids {
            Poids::Regular => &self.regular_id,
            Poids::Medium => &self.medium_id,
            Poids::SemiBold => &self.semibold_id,
            Poids::Bold => &self.bold_id,
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
pub struct CvExperience {
    pub title: String,
    pub company: String,
    /// Ligne de méta : lieu · période (peut être vide).
    pub meta: String,
    /// Réalisations, une puce par entrée.
    pub bullets: Vec<String>,
}

/// Un projet technique.
#[derive(Debug, Clone, Default)]
pub struct CvProject {
    pub name: String,
    pub meta: String,
    pub bullets: Vec<String>,
}

/// Une formation.
#[derive(Debug, Clone, Default)]
pub struct CvEducation {
    pub degree: String,
    pub school: String,
    pub date: String,
}

/// Une langue parlée.
#[derive(Debug, Clone, Default)]
pub struct CvLanguage {
    pub name: String,
    pub level: String,
}

/// Modèle de données du CV à exporter.
#[derive(Debug, Clone, Default)]
pub struct CvPdf {
    pub name: String,
    pub subtitle: String,
    pub phone: Option<String>,
    pub email: String,
    pub city: Option<String>,
    pub linkedin: Option<String>,
    pub website: Option<String>,
    pub profil: String,
    pub skills: Vec<String>,
    pub experiences: Vec<CvExperience>,
    pub projects: Vec<CvProject>,
    pub education: Vec<CvEducation>,
    pub languages: Vec<CvLanguage>,
}

// ---------------------------------------------------------------------------
// Palette du template (spec `exemple_cv.html`).
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
const TEXTE: (f32, f32, f32) = (26.0, 26.0, 26.0);
const SECONDAIRE: (f32, f32, f32) = (63.0, 63.0, 70.0);
const MUTED: (f32, f32, f32) = (85.0, 85.0, 90.0);
const BORDURE: (f32, f32, f32) = (226.0, 226.0, 229.0);
const CHIP_BG: (f32, f32, f32) = (245.0, 245.0, 247.0);

// ---------------------------------------------------------------------------
// Métriques de page et de typographie (px convertis en points, 1 px = 0,75 pt).
// ---------------------------------------------------------------------------

const PAGE_W: f32 = 595.28;
const PAGE_H: f32 = 841.89;
const MARGE: f32 = 14.17; // 0,5 cm
const CONTENU_W: f32 = PAGE_W - 2.0 * MARGE;

const PX: f32 = 0.75;
const fn pt(px: f32) -> f32 {
    px * PX
}

/// Ascendance typographique approximative, pour poser la ligne de base.
const ASCENT: f32 = 0.8;

impl CvPdf {
    /// Exporte le CV dans un PDF A4 autonome.
    ///
    /// # Errors
    /// Retourne une erreur si une police ou une icône embarquée ne peut pas
    /// être décodée, ou si le document ne peut pas être enregistré.
    pub fn render_pdf(&self, path: &Path) -> AppResult<()> {
        let mut avertissements = Vec::new();

        let (regular, medium, semibold, bold) = charger_polices()?;
        let mut document = PdfDocument::new("CV Candilog");

        let (regular_id, medium_id, semibold_id, bold_id) = (
            document.add_font(&regular),
            document.add_font(&medium),
            document.add_font(&semibold),
            document.add_font(&bold),
        );
        let polices = Polices {
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
            phone: ajouter_icone(
                &mut document,
                include_bytes!("../../assets/icons/cv/phone.png"),
            )?,
            mail: ajouter_icone(
                &mut document,
                include_bytes!("../../assets/icons/cv/mail.png"),
            )?,
            pin: ajouter_icone(
                &mut document,
                include_bytes!("../../assets/icons/cv/pin.png"),
            )?,
            linkedin: ajouter_icone(
                &mut document,
                include_bytes!("../../assets/icons/cv/linkedin.png"),
            )?,
            globe: ajouter_icone(
                &mut document,
                include_bytes!("../../assets/icons/cv/globe.png"),
            )?,
            briefcase: ajouter_icone(
                &mut document,
                include_bytes!("../../assets/icons/cv/briefcase.png"),
            )?,
        };

        let mut plan = Plan {
            ops: Vec::new(),
            polices: &polices,
            icones: &icones,
            y: MARGE,
        };

        plan.entete(self);
        plan.section_profil(self);
        plan.section_competences(self);
        plan.section_experiences(self);
        plan.section_projets(self);
        plan.section_formation_langues(self);

        let page = PdfPage::new(Mm(210.0), Mm(297.0), plan.ops);
        let octets = document
            .with_pages(vec![page])
            .save(&PdfSaveOptions::default(), &mut avertissements);
        std::fs::write(path, octets)
            .map_err(|error| AppError::Database(format!("Impossible d'exporter le PDF : {error}")))
    }
}

fn charger_polices() -> AppResult<(ParsedFont, ParsedFont, ParsedFont, ParsedFont)> {
    let decodage = |octets: &[u8]| -> AppResult<ParsedFont> {
        ParsedFont::from_bytes(octets, 0, &mut Vec::new())
            .ok_or_else(|| AppError::Serialization("Police CV illisible".into()))
    };
    Ok((
        decodage(include_bytes!("../../assets/fonts/Geist-Regular.ttf"))?,
        decodage(include_bytes!("../../assets/fonts/Geist-Medium.ttf"))?,
        decodage(include_bytes!("../../assets/fonts/Geist-SemiBold.ttf"))?,
        decodage(include_bytes!("../../assets/fonts/Geist-Bold.ttf"))?,
    ))
}

fn ajouter_icone(document: &mut PdfDocument, octets: &[u8]) -> AppResult<XObjectId> {
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
    polices: &'a Polices,
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

    fn ligne_h(&mut self, x1: f32, x2: f32, y_haut: f32, couleur: Color, epaisseur: f32) {
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

    fn icone(&mut self, x: f32, y_haut: f32, taille: f32, id: &XObjectId) {
        // Les PNG sont rasterisés en 48 px ; on place au facteur taille/48.
        let echelle = taille / 48.0;
        self.ops.push(Op::UseXobject {
            id: id.clone(),
            transform: XObjectTransform {
                translate_x: Some(Pt(x)),
                translate_y: Some(Pt(self.pdf_y(y_haut + taille))),
                scale_x: Some(echelle),
                scale_y: Some(echelle),
                dpi: Some(72.0),
                ..XObjectTransform::default()
            },
        });
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

    /// Trace un paragraphe et rend la hauteur consommée.
    #[allow(clippy::too_many_arguments)]
    fn paragraphe(
        &mut self,
        x: f32,
        poids: Poids,
        taille: f32,
        couleur: Color,
        interligne: f32,
        largeur_max: f32,
        valeur: &str,
    ) -> f32 {
        let mut y = self.y;
        for ligne in self.decouper(poids, taille, valeur, largeur_max) {
            self.texte(
                x,
                y + ASCENT * taille,
                poids,
                taille,
                couleur.clone(),
                &ligne,
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
    fn entete(&mut self, cv: &CvPdf) {
        let x = MARGE;
        let haut_padding = pt(13.6);
        let bas_padding = pt(12.0);
        self.y = MARGE + haut_padding;

        self.texte(
            x,
            self.y + ASCENT * pt(32.0),
            Poids::Bold,
            pt(32.0),
            rgb(TEXTE.0, TEXTE.1, TEXTE.2),
            &cv.name,
        );
        self.y += pt(32.0) * 1.1;

        self.texte(
            x,
            self.y + ASCENT * pt(13.12),
            Poids::SemiBold,
            pt(13.12),
            rgb(ACCENT.0, ACCENT.1, ACCENT.2),
            &cv.subtitle,
        );
        self.y += pt(13.12) * 1.4;

        // Ligne de séparation du header.
        let sep_y = self.y + pt(7.2);
        self.ligne_h(
            x,
            x + CONTENU_W,
            sep_y,
            rgb(BORDURE.0, BORDURE.1, BORDURE.2),
            1.0,
        );
        self.y = sep_y + pt(5.4);

        // Coordonnées.
        let mut contact_x = x;
        let elements = coordonnees(cv, self.icones);
        for (icone, texte) in elements {
            let largeur_element = pt(12.0)
                + pt(4.2)
                + self.largeur_texte(Poids::Medium, pt(10.88), &texte)
                + pt(14.4);
            if contact_x + largeur_element > x + CONTENU_W && contact_x > x {
                contact_x = x;
                self.y += pt(10.88) + pt(3.6);
            }
            self.icone(contact_x, self.y, pt(12.0), &icone);
            contact_x += pt(12.0) + pt(4.2);
            self.texte(
                contact_x,
                self.y + ASCENT * pt(10.88),
                Poids::Medium,
                pt(10.88),
                rgb(SECONDAIRE.0, SECONDAIRE.1, SECONDAIRE.2),
                &texte,
            );
            contact_x += self.largeur_texte(Poids::Medium, pt(10.88), &texte) + pt(14.4);
        }
        self.y += pt(10.88) + bas_padding;
    }

    fn titre_section(&mut self, x: f32, titre: &str) {
        self.texte(
            x,
            self.y + ASCENT * pt(9.92),
            Poids::Bold,
            pt(9.92),
            rgb(ACCENT.0, ACCENT.1, ACCENT.2),
            &titre.to_uppercase(),
        );
        self.y += pt(9.92) + pt(4.48);
        let largeur = self.largeur_texte(Poids::Bold, pt(9.92), &titre.to_uppercase());
        self.ligne_h(
            x,
            x + largeur,
            self.y,
            rgb(ACCENT.0, ACCENT.1, ACCENT.2),
            1.5,
        );
        self.y += pt(1.5) + pt(4.48);
    }

    fn section_profil(&mut self, cv: &CvPdf) {
        self.avance(pt(4.0));
        self.titre_section(MARGE, "Profil");
        self.paragraphe(
            MARGE,
            Poids::Regular,
            pt(12.16),
            rgb(SECONDAIRE.0, SECONDAIRE.1, SECONDAIRE.2),
            pt(12.16) * 1.45,
            CONTENU_W,
            &cv.profil,
        );
    }

    fn section_competences(&mut self, cv: &CvPdf) {
        self.avance(pt(10.0));
        self.titre_section(MARGE, "Compétences techniques");
        let mut x = MARGE;
        let y_base = self.y;
        for competence in &cv.skills {
            let largeur = self.largeur_texte(Poids::Medium, pt(10.56), competence) + 2.0 * pt(6.4);
            if x + largeur > MARGE + CONTENU_W {
                x = MARGE;
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
            self.texte(
                x + pt(6.4),
                self.y + pt(1.92) + ASCENT * pt(10.56),
                Poids::Medium,
                pt(10.56),
                rgb(TEXTE.0, TEXTE.1, TEXTE.2),
                competence,
            );
            x += largeur + pt(2.64);
        }
        if cv.skills.is_empty() {
            self.y = y_base;
        } else {
            self.y += pt(10.56) + 2.0 * pt(1.92) + pt(2.64);
        }
    }

    fn section_experiences(&mut self, cv: &CvPdf) {
        self.avance(pt(10.0));
        self.titre_section(MARGE, "Expérience professionnelle");
        for experience in &cv.experiences {
            self.experience(experience);
            self.avance(pt(6.0));
        }
    }

    fn experience(&mut self, experience: &CvExperience) {
        let x = MARGE;
        self.texte(
            x,
            self.y + ASCENT * pt(13.12),
            Poids::Bold,
            pt(13.12),
            rgb(TEXTE.0, TEXTE.1, TEXTE.2),
            &experience.title,
        );
        self.y += pt(13.12) * 1.35;

        let mut meta = experience.company.clone();
        if !experience.meta.is_empty() {
            meta = format!("{} · {}", meta, experience.meta);
        }
        let briefcase = self.icones.briefcase.clone();
        self.icone(x, self.y, pt(11.0), &briefcase);
        self.texte(
            x + pt(11.0) + pt(3.0),
            self.y + ASCENT * pt(11.2),
            Poids::Regular,
            pt(11.2),
            rgb(SECONDAIRE.0, SECONDAIRE.1, SECONDAIRE.2),
            &meta,
        );
        self.y += pt(11.2) * 1.4;

        for puce in &experience.bullets {
            self.puce(x, puce);
        }
    }

    fn puce(&mut self, x: f32, valeur: &str) {
        let marque = self.y + ASCENT * pt(11.52);
        self.texte(
            x,
            marque,
            Poids::Regular,
            pt(11.52),
            rgb(MUTED.0, MUTED.1, MUTED.2),
            "·",
        );
        let decalage = self.largeur_texte(Poids::Regular, pt(11.52), "·") + pt(3.0);
        self.paragraphe(
            x + decalage,
            Poids::Regular,
            pt(11.52),
            rgb(SECONDAIRE.0, SECONDAIRE.1, SECONDAIRE.2),
            pt(11.52) * 1.38,
            CONTENU_W - decalage,
            valeur,
        );
    }

    fn section_projets(&mut self, cv: &CvPdf) {
        if cv.projects.is_empty() {
            return;
        }
        self.avance(pt(10.0));
        self.titre_section(MARGE, "Projets techniques");
        for projet in &cv.projects {
            let x = MARGE;
            self.texte(
                x,
                self.y + ASCENT * pt(13.12),
                Poids::Bold,
                pt(13.12),
                rgb(TEXTE.0, TEXTE.1, TEXTE.2),
                &projet.name,
            );
            self.y += pt(13.12) * 1.35;
            if !projet.meta.is_empty() {
                self.texte(
                    x,
                    self.y + ASCENT * pt(11.2),
                    Poids::Regular,
                    pt(11.2),
                    rgb(SECONDAIRE.0, SECONDAIRE.1, SECONDAIRE.2),
                    &projet.meta,
                );
                self.y += pt(11.2) * 1.4;
            }
            for puce in &projet.bullets {
                self.puce(x, puce);
            }
            self.avance(pt(6.0));
        }
    }

    fn section_formation_langues(&mut self, cv: &CvPdf) {
        self.avance(pt(10.0));
        let x_gauche = MARGE;
        let x_droite = MARGE + CONTENU_W / 2.0 + pt(11.2);
        let y_debut = self.y;

        self.titre_section(x_gauche, "Formation");
        for formation in &cv.education {
            self.texte(
                x_gauche,
                self.y + ASCENT * pt(12.16),
                Poids::Bold,
                pt(12.16),
                rgb(TEXTE.0, TEXTE.1, TEXTE.2),
                &formation.degree,
            );
            self.y += pt(12.16) * 1.4;
            self.texte(
                x_gauche,
                self.y + ASCENT * pt(10.88),
                Poids::Regular,
                pt(10.88),
                rgb(SECONDAIRE.0, SECONDAIRE.1, SECONDAIRE.2),
                &formation.school,
            );
            self.y += pt(10.88) * 1.4;
            if !formation.date.is_empty() {
                self.texte(
                    x_gauche,
                    self.y + ASCENT * pt(9.6),
                    Poids::Regular,
                    pt(9.6),
                    rgb(MUTED.0, MUTED.1, MUTED.2),
                    &formation.date,
                );
                self.y += pt(9.6) * 1.4;
            }
            self.avance(pt(4.0));
        }
        let fin_formation = self.y;

        self.y = y_debut;
        self.titre_section(x_droite, "Disponibilité & langues");
        for langue in &cv.languages {
            self.texte(
                x_droite,
                self.y + ASCENT * pt(12.16),
                Poids::Bold,
                pt(12.16),
                rgb(TEXTE.0, TEXTE.1, TEXTE.2),
                &format!("{} · {}", langue.name, langue.level),
            );
            self.y += pt(12.16) * 1.4;
        }
        let fin_langues = self.y;

        self.y = fin_formation.max(fin_langues);
    }

    fn avance(&mut self, distance: f32) {
        self.y += distance;
    }
}

/// Construit la liste des coordonnées (icône, texte) du header.
fn coordonnees(cv: &CvPdf, icones: &Icones) -> Vec<(XObjectId, String)> {
    let mut elements = Vec::new();
    if let Some(telephone) = &cv.phone {
        if !telephone.trim().is_empty() {
            elements.push((icones.phone.clone(), telephone.clone()));
        }
    }
    elements.push((icones.mail.clone(), cv.email.clone()));
    if let Some(ville) = &cv.city {
        if !ville.trim().is_empty() {
            elements.push((icones.pin.clone(), ville.clone()));
        }
    }
    if let Some(linkedin) = &cv.linkedin {
        if !linkedin.trim().is_empty() {
            elements.push((icones.linkedin.clone(), linkedin.clone()));
        }
    }
    if let Some(site) = &cv.website {
        if !site.trim().is_empty() {
            elements.push((icones.globe.clone(), site.clone()));
        }
    }
    elements
}

#[cfg(test)]
#[path = "tests/cv_pdf/mod.rs"]
mod tests;
