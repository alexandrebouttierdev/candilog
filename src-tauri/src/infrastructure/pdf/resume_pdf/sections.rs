//! Sections du CV posées sur la page : en-tête, coordonnées, profil, expériences,
//! projets, compétences, formations, certifications et langues.

use super::*;

impl Plan<'_> {
    /// Pose la photo en haut à droite et réduit d'autant la largeur de l'en-tête.
    ///
    /// `dpi` est calculé pour que la taille naturelle de l'image corresponde exactement à
    /// la taille voulue : printpdf ramène sinon toute image à 300 ppp, indépendamment du
    /// cadre demandé.
    pub(super) fn photo(
        &mut self,
        identifiant: &printpdf::XObjectId,
        largeur: f32,
        hauteur: f32,
        largeur_px: u32,
    ) {
        let x = PAGE_W - MARGIN_RIGHT - largeur;
        let y_bas = self.pdf_y(MARGIN_TOP + hauteur);
        self.ops.push(Op::UseXobject {
            id: identifiant.clone(),
            transform: XObjectTransform {
                translate_x: Some(Pt(x)),
                translate_y: Some(Pt(y_bas)),
                dpi: Some(largeur_px as f32 * 72.0 / largeur),
                ..XObjectTransform::default()
            },
        });
        self.header_w = HEADER_W - largeur - PHOTO_GAP;
    }

    pub(super) fn entete(&mut self, resume: &ResumePdf) {
        self.y = MARGIN_TOP;
        // Copie locale : les blocs de repli empruntent `self` mutablement.
        let header_w = self.header_w;

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
                    largeur_max: header_w,
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
                        largeur_max: header_w,
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
                        largeur_max: header_w.min(pt(64.0) * 6.0),
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

    pub(super) fn contact_row_1(&mut self, resume: &ResumePdf) {
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

    pub(super) fn contact_row_2(&mut self, resume: &ResumePdf) {
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

    pub(super) fn contact_line(&mut self, parts: &[String]) {
        let size = pt(10.1);
        self.flux_horizontal(
            &StyleFlux {
                x: MARGIN_LEFT,
                largeur_max: self.header_w,
                face: FontFace::Mono(MonoWeight::Regular),
                size,
                couleur: rgb(CONTACT.0, CONTACT.1, CONTACT.2),
                gap: pt(18.0),
                interligne: size * 1.45,
            },
            parts,
        );
    }

    pub(super) fn section_profile(&mut self, resume: &ResumePdf) {
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

    pub(super) fn section_experiences(&mut self, resume: &ResumePdf) {
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

    pub(super) fn experience_item(&mut self, experience: &ResumeExperience) {
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
    pub(super) fn periode_a_droite(&mut self, period: &str, size: f32) -> f32 {
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

    pub(super) fn section_projects(&mut self, resume: &ResumePdf) {
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

    pub(super) fn project_item(&mut self, project: &ResumeProject) {
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

    pub(super) fn section_skills(&mut self, resume: &ResumePdf) {
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
            let generic_name = group.name.trim().eq_ignore_ascii_case("Compétences");
            let mut group_w = 0.0;
            if !generic_name {
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
            }
            let chips_x = if generic_name {
                CONTENT_X
            } else {
                CONTENT_X + group_w + pt(14.0)
            };
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

    pub(super) fn section_education(&mut self, resume: &ResumePdf) {
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

    pub(super) fn section_certifications(&mut self, resume: &ResumePdf) {
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
            if let Some(description) = certification
                .description
                .as_deref()
                .map(str::trim)
                .filter(|text| !text.is_empty())
            {
                self.paragraphe(
                    StyleParagraphe {
                        x: CONTENT_X,
                        face: FontFace::Sans(SansWeight::Regular),
                        size: pt(10.2),
                        couleur: rgb(BODY.0, BODY.1, BODY.2),
                        interligne: pt(10.2) * 1.35,
                        largeur_max: CONTENT_W,
                    },
                    description,
                );
            }
        }
    }

    pub(super) fn section_languages(&mut self, resume: &ResumePdf) {
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
