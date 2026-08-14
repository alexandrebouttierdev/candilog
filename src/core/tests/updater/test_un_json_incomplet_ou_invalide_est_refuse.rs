use super::super::analyser_reponse;
use semver::Version;

#[test]
fn un_json_incomplet_ou_invalide_est_refuse() {
    let actuelle = Version::new(0, 2, 0);
    assert_eq!(analyser_reponse("pas du json", &actuelle), None);
    assert_eq!(analyser_reponse("{}", &actuelle), None);
    assert_eq!(
        analyser_reponse(
            r#"{"tag_name": "zzz", "html_url": "x", "assets": []}"#,
            &actuelle
        ),
        None,
        "un tag non semver ne doit pas produire de mise à jour"
    );
}
