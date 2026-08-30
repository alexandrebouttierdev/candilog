use super::*;

#[test]
fn une_lettre_sans_balise_reste_du_texte_brut() {
    let paragraphs = parse_letter("Madame, Monsieur,\n\nJe vous écris.");

    assert_eq!(paragraphs.len(), 2);
    assert_eq!(paragraphs[0].plain(), "Madame, Monsieur,");
    assert_eq!(paragraphs[1].plain(), "Je vous écris.");
    assert!(!paragraphs[0].runs[0].bold);
    assert_eq!(paragraphs[0].align, LetterAlign::Left);
    assert_eq!(paragraphs[0].size, LetterSize::Normal);
}

#[test]
fn le_gras_le_souligne_lalignement_et_la_taille_sont_relus() {
    let paragraphs =
        parse_letter("<p align=\"center\" size=\"large\">Bonjour <b>Nova</b> et <u>Atlas</u></p>");

    assert_eq!(paragraphs.len(), 1);
    assert_eq!(paragraphs[0].align, LetterAlign::Center);
    assert_eq!(paragraphs[0].size, LetterSize::Large);
    assert_eq!(
        paragraphs[0].runs,
        vec![
            LetterRun {
                text: "Bonjour ".into(),
                bold: false,
                underline: false
            },
            LetterRun {
                text: "Nova".into(),
                bold: true,
                underline: false
            },
            LetterRun {
                text: " et ".into(),
                bold: false,
                underline: false
            },
            LetterRun {
                text: "Atlas".into(),
                bold: false,
                underline: true
            },
        ]
    );
}

#[test]
fn une_balise_inconnue_perd_sa_mise_en_forme_pas_ses_mots() {
    // Un collage venu d'un traitement de texte arrive plein de balises et de styles : ce
    // qui compte est de ne jamais perdre le texte de l'utilisateur.
    let paragraphs =
        parse_letter("<p><span style=\"color:red\">Rouge</span><script>alert(1)</script> vif</p>");

    assert_eq!(paragraphs[0].plain(), "Rougealert(1) vif");
    assert!(paragraphs[0].runs.iter().all(|run| !run.bold));
}

#[test]
fn le_balisage_canonique_echappe_les_chevrons_du_texte() {
    let paragraphs = parse_letter("<p>5 &lt; 7 &amp; 8 &gt; 6</p>");

    assert_eq!(paragraphs[0].plain(), "5 < 7 & 8 > 6");
    assert_eq!(to_markup(&paragraphs), "<p>5 &lt; 7 &amp; 8 &gt; 6</p>");
}

#[test]
fn lassainissement_est_stable_par_aller_retour() {
    let source =
        "<p align=\"right\" size=\"small\">Je suis <b>disponible</b></p><p>Cordialement,</p>";

    let une_fois = sanitize_letter(source);

    assert_eq!(une_fois, source);
    assert_eq!(sanitize_letter(&une_fois), une_fois);
}

#[test]
fn un_texte_brut_assaini_devient_du_balisage_canonique() {
    assert_eq!(
        sanitize_letter("Madame, Monsieur,\n\nJe vous écris."),
        "<p>Madame, Monsieur,</p><p>Je vous écris.</p>"
    );
}
