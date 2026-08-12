use super::*;

#[test]
fn la_selection_de_modele_modifie_le_brouillon() {
    let mut app = app_de_test();

    let _ = update(
        &mut app,
        Message::SettingsModelChanged("deepseek-v4-flash".into()),
    );

    assert_eq!(app.settings_form.draft.llm.model, "deepseek-v4-flash");
    assert_ne!(app.data.settings.llm.model, "deepseek-v4-flash");
}
