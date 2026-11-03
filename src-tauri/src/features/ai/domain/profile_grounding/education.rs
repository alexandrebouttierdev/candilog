//! Formations présentes dans le CV mais oubliées par le modèle.

use super::*;

/// Ajoute une formation manquante après le recadrage, sans second appel au modèle.
///
/// Le score compte des lignes de formation face aux indices de diplôme du CV tronqué.
/// On n'ajoute une ligne qu'en queue, pour un indice non déjà couvert, et seulement si
/// la fenêtre contient exactement un diplôme ou une école recopié tel quel. Une fenêtre
/// ambiguë est ignorée. Les formations déjà émises ne sont ni réécrites ni réordonnées.
pub fn completer_formations_manquantes(source: &str, profile: &mut Profile) {
    let attendu = compter_indices_diplome(source);
    if attendu == 0 || profile.education.len() >= attendu {
        return;
    }
    let occurrences = occurrences_diplome(source);
    let mut couverts = vec![false; profile.education.len()];
    for occurrence in &occurrences {
        if profile.education.len() >= attendu {
            break;
        }
        if indice_deja_couvert(profile, &mut couverts, &occurrence.indice) {
            continue;
        }
        let Some((diplome, ecole)) = formation_unique(source, &occurrences, occurrence) else {
            continue;
        };
        if diplome.is_empty() && ecole.is_empty() {
            continue;
        }
        if formation_deja_emise(profile, &diplome, &ecole) {
            continue;
        }
        profile.education.push(Education {
            degree: diplome,
            school: ecole,
            ..Education::default()
        });
        couverts.push(true);
    }
}

pub(super) const INDICES_DIPLOME: [&str; 19] = [
    "baccalauréat",
    "baccalaureat",
    "master",
    "licence",
    "bachelor",
    "doctorat",
    "phd",
    "mba",
    "dut",
    "bts",
    "deug",
    "deust",
    "ingénieur",
    "ingenieur",
    "msc",
    "maîtrise",
    "maitrise",
    "diplôme",
    "diplome",
];

pub(super) const INDICES_REPES: [&str; 11] = [
    "master",
    "licence",
    "bachelor",
    "bts",
    "dut",
    "mba",
    "doctorat",
    "phd",
    "ingénieur",
    "ingenieur",
    "msc",
];

pub(super) struct OccurrenceDiplome {
    debut: usize,
    fin: usize,
    indice: String,
}

pub(super) fn compter_indices_diplome(source: &str) -> usize {
    let jetons = jetons_diplome(source);
    let vus = INDICES_DIPLOME
        .iter()
        .filter(|indice| jetons.iter().any(|jeton| jeton == *indice))
        .count();
    let repetes = jetons
        .iter()
        .filter(|jeton| INDICES_REPES.contains(&jeton.as_str()))
        .count();
    repetes.max(vus.min(6))
}

pub(super) fn jetons_diplome(source: &str) -> Vec<String> {
    let minuscule = source.to_lowercase();
    let espaces = minuscule.replace(['\n', '\t', '/', ',', ';', ':', '(', ')', '[', ']'], " ");
    espaces
        .split_whitespace()
        .map(|jeton| {
            jeton
                .trim_matches(|caractere: char| !caractere.is_alphanumeric())
                .to_owned()
        })
        .filter(|jeton| !jeton.is_empty())
        .collect()
}

pub(super) fn occurrences_diplome(source: &str) -> Vec<OccurrenceDiplome> {
    let mut occurrences = Vec::new();
    let mut debut_mot = None;
    for (index, caractere) in source.char_indices() {
        if caractere.is_alphanumeric() || caractere == '\'' || caractere == '’' {
            debut_mot.get_or_insert(index);
            continue;
        }
        if let Some(debut) = debut_mot.take() {
            retenir_indice(source, debut, index, &mut occurrences);
        }
    }
    if let Some(debut) = debut_mot {
        retenir_indice(source, debut, source.len(), &mut occurrences);
    }
    occurrences
}

pub(super) fn retenir_indice(
    source: &str,
    debut: usize,
    fin: usize,
    occurrences: &mut Vec<OccurrenceDiplome>,
) {
    let Some(brut) = source.get(debut..fin) else {
        return;
    };
    let epure = brut_indice(brut);
    if INDICES_DIPLOME.contains(&epure.as_str()) {
        let relatif = brut.find(epure.as_str()).unwrap_or(0);
        let copie_debut = debut + relatif;
        occurrences.push(OccurrenceDiplome {
            debut: copie_debut,
            fin: copie_debut + epure.len(),
            indice: epure,
        });
    }
}

pub(super) fn brut_indice(brut: &str) -> String {
    brut.trim_matches(|caractere: char| !caractere.is_alphanumeric())
        .to_lowercase()
}

pub(super) fn indice_deja_couvert(profile: &Profile, couverts: &mut [bool], indice: &str) -> bool {
    for (index, formation) in profile.education.iter().enumerate() {
        if couverts.get(index).copied().unwrap_or(false) {
            continue;
        }
        if contient_indice(&formation.degree, indice) || contient_indice(&formation.school, indice)
        {
            couverts[index] = true;
            return true;
        }
    }
    false
}

pub(super) fn contient_indice(valeur: &str, indice: &str) -> bool {
    jetons_diplome(valeur).iter().any(|jeton| jeton == indice)
}

pub(super) fn formation_deja_emise(profile: &Profile, diplome: &str, ecole: &str) -> bool {
    let diplome = diplome.to_lowercase();
    let ecole = ecole.to_lowercase();
    profile.education.iter().any(|formation| {
        !diplome.is_empty() && formation.degree.to_lowercase() == diplome
            || !ecole.is_empty() && formation.school.to_lowercase() == ecole && diplome.is_empty()
    })
}

pub(super) fn formation_unique(
    source: &str,
    occurrences: &[OccurrenceDiplome],
    occurrence: &OccurrenceDiplome,
) -> Option<(String, String)> {
    if est_intitule_de_poste(source, occurrence) {
        return None;
    }
    let (fenetre_debut, fenetre_fin) = fenetre_occurrence(source, occurrences, occurrence);
    let texte = source.get(fenetre_debut..fenetre_fin)?;
    if texte.contains('…') {
        return None;
    }
    let mut diplomes = Vec::new();
    let mut ecoles = Vec::new();
    for ligne in lignes_de(source, fenetre_debut, fenetre_fin) {
        if est_entete_formation(ligne) {
            continue;
        }
        for fragment in fragments_formation(ligne) {
            if fragment.chars().count() > 72 {
                continue;
            }
            let jetons = jetons_diplome(fragment);
            if jetons.iter().any(|jeton| jeton == &occurrence.indice) {
                if jetons
                    .iter()
                    .filter(|jeton| INDICES_DIPLOME.contains(&jeton.as_str()))
                    .count()
                    == 1
                    && fragment.split_whitespace().count() <= 8
                    && !phrase_narrative(fragment)
                {
                    pousser_unique(&mut diplomes, fragment);
                }
                continue;
            }
            if a_marqueur_ecole(fragment) && fragment.split_whitespace().count() <= 8 {
                pousser_unique(&mut ecoles, fragment);
            }
        }
    }
    if diplomes.len() > 1 || ecoles.len() > 1 {
        return None;
    }
    let diplome = diplomes.first().copied().unwrap_or("").trim();
    let ecole = ecoles.first().copied().unwrap_or("").trim();
    if diplome.is_empty() && ecole.is_empty() {
        return None;
    }
    if !diplome.is_empty() && !source.contains(diplome) {
        return None;
    }
    if !ecole.is_empty() && !source.contains(ecole) {
        return None;
    }
    Some((diplome.to_owned(), ecole.to_owned()))
}

pub(super) fn est_intitule_de_poste(source: &str, occurrence: &OccurrenceDiplome) -> bool {
    if !matches!(occurrence.indice.as_str(), "ingénieur" | "ingenieur") {
        return false;
    }
    let ligne = ligne_contenant(source, occurrence.debut).unwrap_or("");
    if a_marqueur_ecole(ligne) || ligne.to_lowercase().contains("dipl") {
        return false;
    }
    let autour = contexte_lignes(source, occurrence.debut, 1);
    autour.contains("201") || autour.contains("199") || autour.contains("202")
}

pub(super) fn fenetre_occurrence(
    source: &str,
    occurrences: &[OccurrenceDiplome],
    occurrence: &OccurrenceDiplome,
) -> (usize, usize) {
    let precedente = occurrences
        .iter()
        .rev()
        .find(|autre| autre.debut < occurrence.debut)
        .map(|autre| autre.fin);
    let suivante = occurrences
        .iter()
        .find(|autre| autre.debut > occurrence.debut)
        .map(|autre| autre.debut);
    let ligne = ligne_contenant(source, occurrence.debut).unwrap_or("");
    let ligne_debut = source.find(ligne).unwrap_or(occurrence.debut);
    let mut debut = ligne_debut.max(precedente.unwrap_or(0));
    let mut fin = ligne_debut + ligne.len();
    if let Some(suivante) = suivante {
        fin = fin.min(suivante);
    }
    // L'école suit souvent le diplôme. La ligne d'avant appartient au diplôme précédent.
    if precedente.is_none() {
        if let Some(avant) = ligne_precedente(source, occurrence.debut) {
            let avant_debut = source.find(avant).unwrap_or(debut);
            if !occurrences_dans(occurrences, avant_debut, occurrence.debut) {
                debut = debut.min(avant_debut);
            }
        }
    }
    if let Some(apres) = ligne_suivante(source, occurrence.debut) {
        let apres_debut = source[occurrence.fin..]
            .find(apres)
            .map(|relatif| occurrence.fin + relatif)
            .unwrap_or(fin);
        if suivante.is_none_or(|borne| apres_debut < borne)
            && !occurrences_dans(occurrences, fin, apres_debut.saturating_add(apres.len()))
        {
            fin = fin.max((apres_debut + apres.len()).min(suivante.unwrap_or(source.len())));
        }
    }
    (
        debut.min(occurrence.debut),
        fin.max(occurrence.fin).min(source.len()),
    )
}

pub(super) fn occurrences_dans(
    occurrences: &[OccurrenceDiplome],
    debut: usize,
    fin: usize,
) -> bool {
    occurrences
        .iter()
        .any(|occurrence| occurrence.debut >= debut && occurrence.debut < fin)
}

pub(super) fn lignes_de(source: &str, debut: usize, fin: usize) -> Vec<&str> {
    source
        .get(debut..fin)
        .unwrap_or("")
        .split('\n')
        .map(str::trim)
        .filter(|ligne| !ligne.is_empty())
        .collect()
}

pub(super) fn fragments_formation(ligne: &str) -> Vec<&str> {
    let mut morceaux = vec![ligne];
    for separateur in [" | ", " - ", " – ", " — ", " · ", " • ", "|"] {
        let mut suivant = Vec::new();
        for morceau in morceaux {
            if morceau.contains(separateur) {
                suivant.extend(
                    morceau
                        .split(separateur)
                        .map(str::trim)
                        .filter(|part| !part.is_empty()),
                );
            } else {
                suivant.push(morceau);
            }
        }
        morceaux = suivant;
    }
    morceaux
}

pub(super) fn pousser_unique<'a>(liste: &mut Vec<&'a str>, valeur: &'a str) {
    let epure = valeur.trim();
    if epure.is_empty() {
        return;
    }
    if !liste.iter().any(|autre| autre.eq_ignore_ascii_case(epure)) {
        liste.push(epure);
    }
}

pub(super) fn phrase_narrative(fragment: &str) -> bool {
    let minuscule = fragment.to_lowercase();
    [" puis ", " avec ", " pour ", " dans ", " voir ", " suivi "]
        .iter()
        .any(|mot| minuscule.contains(mot))
}

pub(super) fn a_marqueur_ecole(fragment: &str) -> bool {
    let jetons = jetons_diplome(fragment);
    const MARQUEURS: [&str; 12] = [
        "université",
        "universite",
        "university",
        "école",
        "ecole",
        "institut",
        "iut",
        "lycée",
        "lycee",
        "college",
        "campus",
        "school",
    ];
    jetons
        .iter()
        .any(|jeton| MARQUEURS.contains(&jeton.as_str()))
}

pub(super) fn est_entete_formation(ligne: &str) -> bool {
    let jetons = jetons_diplome(ligne);
    if jetons.len() > 3 {
        return false;
    }
    const ENTETES: [&str; 6] = [
        "formation",
        "formations",
        "education",
        "etudes",
        "diplomes",
        "scolarite",
    ];
    jetons
        .iter()
        .all(|jeton| ENTETES.contains(&jeton.as_str()) || jeton == "diplôme" || jeton == "diplome")
        && jetons.iter().any(|jeton| {
            ENTETES.contains(&jeton.as_str())
                || jeton == "diplômes"
                || jeton == "diplome"
                || jeton == "diplôme"
        })
}

pub(super) fn ligne_contenant(source: &str, offset: usize) -> Option<&str> {
    source.lines().find(|ligne| {
        source
            .find(ligne)
            .is_some_and(|debut| offset >= debut && offset < debut + ligne.len())
    })
}

pub(super) fn contexte_lignes(source: &str, offset: usize, rayon: usize) -> String {
    let lignes: Vec<&str> = source.lines().collect();
    let index = lignes.iter().position(|ligne| {
        source
            .find(ligne)
            .is_some_and(|debut| offset >= debut && offset < debut + ligne.len())
    });
    let Some(index) = index else {
        return String::new();
    };
    let debut = index.saturating_sub(rayon);
    let fin = (index + rayon + 1).min(lignes.len());
    lignes[debut..fin].join("\n")
}

pub(super) fn ligne_precedente(source: &str, offset: usize) -> Option<&str> {
    let lignes: Vec<&str> = source
        .lines()
        .filter(|ligne| !ligne.trim().is_empty())
        .collect();
    let index = lignes.iter().position(|ligne| {
        source
            .find(*ligne)
            .is_some_and(|debut| offset >= debut && offset < debut + ligne.len())
    })?;
    index.checked_sub(1).map(|index| lignes[index])
}

pub(super) fn ligne_suivante(source: &str, offset: usize) -> Option<&str> {
    let lignes: Vec<&str> = source
        .lines()
        .filter(|ligne| !ligne.trim().is_empty())
        .collect();
    let index = lignes.iter().position(|ligne| {
        source
            .find(*ligne)
            .is_some_and(|debut| offset >= debut && offset < debut + ligne.len())
    })?;
    lignes.get(index + 1).copied()
}
