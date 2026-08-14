use super::super::analyser_reponse;
use semver::Version;

const REPONSE: &str = r#"{
    "tag_name": "v0.2.0",
    "html_url": "https://github.com/alexandrebouttierdev/candilog-releases/releases/tag/v0.2.0",
    "assets": []
}"#;

#[test]
fn une_version_egale_ou_inferieure_est_ignoree() {
    assert_eq!(analyser_reponse(REPONSE, &Version::new(0, 2, 0)), None);
    assert_eq!(analyser_reponse(REPONSE, &Version::new(0, 3, 0)), None);
}
