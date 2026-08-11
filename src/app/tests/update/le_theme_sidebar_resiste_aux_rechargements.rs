//! Régression : naviguer ne doit pas annuler le thème choisi dans la sidebar.

use super::*;

#[test]
fn le_theme_sidebar_resiste_aux_rechargements() {
    let mut app = app_de_test();
    let instantane_avant_bascule = app.data.clone();
    let ancien_theme = app.is_dark;

    envoyer(&mut app, [Message::ToggleTheme]);

    assert_ne!(app.is_dark, ancien_theme);
    assert_eq!(app.settings_form.draft.theme, app.data.settings.theme);

    let theme_choisi = app.is_dark;
    app.appliquer_instantane(instantane_avant_bascule, &[]);
    assert_eq!(
        app.is_dark, theme_choisi,
        "un instantané concurrent ne doit pas rétablir l'ancien thème"
    );
}
