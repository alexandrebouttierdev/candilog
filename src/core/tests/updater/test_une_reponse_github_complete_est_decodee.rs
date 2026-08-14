use super::super::{analyser_reponse, extension_installeur};
use semver::Version;

const REPONSE_EXEMPLE: &str = r#"{
    "tag_name": "v0.3.0",
    "body": "Nouvelle version avec corrections.",
    "html_url": "https://github.com/alexandrebouttierdev/candilog-releases/releases/tag/v0.3.0",
    "assets": [
        {
            "name": "candilog-ubuntu-0.3.0.deb",
            "browser_download_url": "https://github.com/.../candilog-ubuntu-0.3.0.deb"
        },
        {
            "name": "candilog-fedora-0.3.0.rpm",
            "browser_download_url": "https://github.com/.../candilog-fedora-0.3.0.rpm"
        },
        {
            "name": "candilog-macos-0.3.0.dmg",
            "browser_download_url": "https://github.com/.../candilog-macos-0.3.0.dmg"
        },
        {
            "name": "candilog-windows-0.3.0.exe",
            "browser_download_url": "https://github.com/.../candilog-windows-0.3.0.exe"
        }
    ]
}"#;

#[test]
fn une_reponse_github_complete_est_decodee() {
    let info = analyser_reponse(REPONSE_EXEMPLE, &Version::new(0, 2, 0))
        .expect("la réponse complète doit être décodée");
    assert_eq!(info.version, Version::new(0, 3, 0));
    assert_eq!(info.notes, "Nouvelle version avec corrections.");
    assert_eq!(
        info.page_url,
        "https://github.com/alexandrebouttierdev/candilog-releases/releases/tag/v0.3.0"
    );
    if let Some(asset) = info.asset {
        let extension = extension_installeur().expect("plateforme reconnue en test");
        assert!(
            asset.name.ends_with(&format!(".{extension}")),
            "l'asset {} doit correspondre à l'extension {extension}",
            asset.name
        );
    }
}
