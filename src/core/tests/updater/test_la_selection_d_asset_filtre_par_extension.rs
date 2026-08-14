use super::super::{asset_pour_extension, AssetInfo};

fn asset(nom: &str) -> AssetInfo {
    AssetInfo {
        name: nom.to_string(),
        url: format!("https://example.test/{nom}"),
    }
}

#[test]
fn la_selection_d_asset_filtre_par_extension() {
    let assets = vec![
        asset("candilog-ubuntu-latest.deb"),
        asset("candilog-fedora-latest.rpm"),
        asset("candilog-macos-latest.dmg"),
        asset("candilog-windows-latest.exe"),
    ];
    assert_eq!(
        asset_pour_extension(&assets, "deb").map(|a| a.name),
        Some("candilog-ubuntu-latest.deb".to_string())
    );
    assert_eq!(
        asset_pour_extension(&assets, "rpm").map(|a| a.name),
        Some("candilog-fedora-latest.rpm".to_string())
    );
    assert_eq!(
        asset_pour_extension(&assets, "dmg").map(|a| a.name),
        Some("candilog-macos-latest.dmg".to_string())
    );
    assert_eq!(
        asset_pour_extension(&assets, "exe").map(|a| a.name),
        Some("candilog-windows-latest.exe".to_string())
    );
    assert_eq!(asset_pour_extension(&assets, "AppImage"), None);
}
