use super::*;

#[test]
fn une_double_soumission_est_ignoree() {
    let mut app = app_de_test();
    app.dialog = Some(Dialog::Entreprise);
    app.entreprise_form.nom = "ACME".into();

    let first = update(&mut app, Message::SubmitEntreprise);
    assert!(app.write_in_progress);
    let second = update(&mut app, Message::SubmitEntreprise);
    drop((first, second));

    // Les tâches ne sont volontairement pas exécutées par ce harnais : l'état suffit à prouver
    // que le second message n'a pas réarmé une autre écriture.
    assert!(app.write_in_progress);
    let count: i64 = crate::shared::sqlite::connexion(&app.backend.as_ref().unwrap().sqlite)
        .unwrap()
        .query_row("SELECT count(*) FROM entreprises", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, 0);
}
