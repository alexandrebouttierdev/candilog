//! Balisage restreint du corps d'une lettre.
//!
//! Le corps est stocké dans une seule colonne `TEXT`, et doit rester lisible tel quel : le
//! balisage se limite donc à ce que l'export PDF sait honorer — paragraphes, gras, taille et
//! alignement. Tout le reste est écarté à l'analyse, ce qui fait de cette fonction
//! l'assainisseur du contenu : ce qui n'est pas dans cette grammaire n'atteint jamais la
//! base, l'aperçu ni le PDF.
//!
//! Une lettre écrite avant l'éditeur n'a aucune balise : elle est lue comme du texte brut,
//! découpé sur les lignes vides.

/// Alignement d'un paragraphe. La justification n'existe pas : le moteur PDF ne sait pas
/// répartir les blancs, et un faux justifié se voit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LetterAlign {
    #[default]
    Left,
    Center,
    Right,
}

/// Taille d'un paragraphe, relative au corps de la lettre.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LetterSize {
    Small,
    #[default]
    Normal,
    Large,
}

impl LetterSize {
    /// Facteur appliqué à la taille de base du corps.
    #[must_use]
    pub fn scale(self) -> f32 {
        match self {
            Self::Small => 0.9,
            Self::Normal => 1.0,
            Self::Large => 1.15,
        }
    }
}

/// Fragment de texte homogène.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LetterRun {
    pub text: String,
    pub bold: bool,
    pub underline: bool,
}

/// Paragraphe : des fragments, un alignement, une taille.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LetterParagraph {
    pub runs: Vec<LetterRun>,
    pub align: LetterAlign,
    pub size: LetterSize,
}

impl LetterParagraph {
    /// Texte nu du paragraphe, sans mise en forme.
    #[must_use]
    pub fn plain(&self) -> String {
        self.runs.iter().map(|run| run.text.as_str()).collect()
    }
}

/// Analyse un corps de lettre, balisé ou non.
#[must_use]
pub fn parse_letter(content: &str) -> Vec<LetterParagraph> {
    if !content.contains("<p") {
        return content
            .split("\n\n")
            .map(str::trim)
            .filter(|bloc| !bloc.is_empty())
            .map(|bloc| LetterParagraph {
                runs: vec![LetterRun {
                    text: bloc.to_owned(),
                    bold: false,
                    underline: false,
                }],
                ..LetterParagraph::default()
            })
            .collect();
    }
    Analyseur::new(content).paragraphes()
}

/// Réécrit les paragraphes dans le balisage canonique : c'est cette forme qui est stockée.
#[must_use]
pub fn to_markup(paragraphs: &[LetterParagraph]) -> String {
    let mut sortie = String::new();
    for paragraphe in paragraphs {
        sortie.push_str("<p");
        match paragraphe.align {
            LetterAlign::Left => {}
            LetterAlign::Center => sortie.push_str(" align=\"center\""),
            LetterAlign::Right => sortie.push_str(" align=\"right\""),
        }
        match paragraphe.size {
            LetterSize::Normal => {}
            LetterSize::Small => sortie.push_str(" size=\"small\""),
            LetterSize::Large => sortie.push_str(" size=\"large\""),
        }
        sortie.push('>');
        for run in &paragraphe.runs {
            if run.text.is_empty() {
                continue;
            }
            if run.bold {
                sortie.push_str("<b>");
            }
            if run.underline {
                sortie.push_str("<u>");
            }
            sortie.push_str(&echapper(&run.text));
            if run.underline {
                sortie.push_str("</u>");
            }
            if run.bold {
                sortie.push_str("</b>");
            }
        }
        sortie.push_str("</p>");
    }
    sortie
}

/// Normalise un corps de lettre : ce qui sort est du balisage canonique, sans surprise.
#[must_use]
pub fn sanitize_letter(content: &str) -> String {
    let paragraphs = parse_letter(content);
    if paragraphs.is_empty() {
        return String::new();
    }
    to_markup(&paragraphs)
}

fn echapper(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

struct Analyseur<'a> {
    reste: &'a str,
}

impl<'a> Analyseur<'a> {
    fn new(content: &'a str) -> Self {
        Self { reste: content }
    }

    fn paragraphes(mut self) -> Vec<LetterParagraph> {
        let mut sortie = Vec::new();
        while let Some(debut) = self.reste.find("<p") {
            self.reste = &self.reste[debut..];
            let Some(fin_balise) = self.reste.find('>') else {
                break;
            };
            let ouverture = &self.reste[..fin_balise];
            self.reste = &self.reste[fin_balise + 1..];
            let contenu = match self.reste.find("</p>") {
                Some(index) => {
                    let contenu = &self.reste[..index];
                    self.reste = &self.reste[index + 4..];
                    contenu
                }
                None => {
                    let contenu = self.reste;
                    self.reste = "";
                    contenu
                }
            };
            let runs = fragments(contenu);
            if runs.iter().all(|run| run.text.trim().is_empty()) {
                sortie.push(LetterParagraph {
                    runs: Vec::new(),
                    align: alignement(ouverture),
                    size: taille(ouverture),
                });
                continue;
            }
            sortie.push(LetterParagraph {
                runs,
                align: alignement(ouverture),
                size: taille(ouverture),
            });
        }
        sortie
    }
}

fn alignement(ouverture: &str) -> LetterAlign {
    if ouverture.contains("align=\"center\"") {
        LetterAlign::Center
    } else if ouverture.contains("align=\"right\"") {
        LetterAlign::Right
    } else {
        LetterAlign::Left
    }
}

fn taille(ouverture: &str) -> LetterSize {
    if ouverture.contains("size=\"small\"") {
        LetterSize::Small
    } else if ouverture.contains("size=\"large\"") {
        LetterSize::Large
    } else {
        LetterSize::Normal
    }
}

/// Découpe le contenu d'un paragraphe en fragments gras / non gras.
///
/// Toute balise inconnue est ignorée sans supprimer son texte : un collage venu d'un
/// traitement de texte perd sa mise en forme, jamais ses mots.
fn fragments(contenu: &str) -> Vec<LetterRun> {
    let mut runs: Vec<LetterRun> = Vec::new();
    let mut courant = String::new();
    let mut bold = false;
    let mut underline = false;
    let mut reste = contenu;

    let pousser = |texte: &mut String, bold: bool, underline: bool, runs: &mut Vec<LetterRun>| {
        if texte.is_empty() {
            return;
        }
        match runs.last_mut() {
            Some(dernier) if dernier.bold == bold && dernier.underline == underline => {
                dernier.text.push_str(texte);
            }
            _ => runs.push(LetterRun {
                text: texte.clone(),
                bold,
                underline,
            }),
        }
        texte.clear();
    };

    while let Some(index) = reste.find('<') {
        courant.push_str(&decoder(&reste[..index]));
        reste = &reste[index..];
        let Some(fin) = reste.find('>') else {
            courant.push_str(&decoder(reste));
            reste = "";
            break;
        };
        let balise = reste[..=fin].to_ascii_lowercase();
        reste = &reste[fin + 1..];
        if balise.starts_with("<b>") || balise.starts_with("<strong") {
            pousser(&mut courant, bold, underline, &mut runs);
            bold = true;
        } else if balise.starts_with("</b>") || balise.starts_with("</strong") {
            pousser(&mut courant, bold, underline, &mut runs);
            bold = false;
        } else if balise.starts_with("<u>") || balise.starts_with("<ins") {
            pousser(&mut courant, bold, underline, &mut runs);
            underline = true;
        } else if balise.starts_with("</u>") || balise.starts_with("</ins") {
            pousser(&mut courant, bold, underline, &mut runs);
            underline = false;
        } else if balise.starts_with("<br") {
            courant.push('\n');
        }
    }
    courant.push_str(&decoder(reste));
    pousser(&mut courant, bold, underline, &mut runs);
    runs
}

fn decoder(value: &str) -> String {
    value
        .replace("&nbsp;", " ")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&#39;", "'")
        .replace("&quot;", "\"")
        .replace("&amp;", "&")
}

#[cfg(test)]
#[path = "tests/letter_markup/mod.rs"]
mod tests;
