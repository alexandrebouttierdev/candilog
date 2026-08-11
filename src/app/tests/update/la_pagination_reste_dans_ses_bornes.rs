//! Cas de test isolé.

use super::*;

/// Les compteurs de page n'étaient bornés ni à la hausse ni à la baisse : « Précédent » sur la
/// première page amenait `ats_page` à 0, et « Suivant » pouvait croître sans limite. Sur trois
/// pages d'historique, dix clics sur « Suivant » portaient le compteur à 13 sans que rien ne
/// bouge à l'écran, et il fallait ensuite dix clics sur « Précédent » pour que la pagination
/// réagisse de nouveau. Le bornage n'existait que dans la couche d'affichage.
#[test]
fn la_pagination_reste_dans_ses_bornes() {
    let mut app = app_de_test();

    // Historique vide : une seule page, les deux extrémités sont confondues.
    envoyer(&mut app, std::iter::repeat_n(Message::AtsPageNext, 10));
    assert_eq!(
        app.ats_page, 1,
        "« Suivant » ne doit pas dépasser la dernière page"
    );

    envoyer(&mut app, std::iter::repeat_n(Message::AtsPagePrev, 10));
    assert_eq!(
        app.ats_page, 1,
        "« Précédent » ne doit jamais amener la page à 0"
    );

    envoyer(&mut app, std::iter::repeat_n(Message::LlmPageNext, 5));
    assert_eq!(app.llm_page, 1);
    envoyer(&mut app, std::iter::repeat_n(Message::LlmPagePrev, 5));
    assert_eq!(app.llm_page, 1);
}
