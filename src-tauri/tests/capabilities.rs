//! Verrou des permissions accordées à la webview (`capabilities/default.json`).
//!
//! Un ensemble large comme `core:default` réapparaît facilement — c'est le gabarit de
//! `tauri init` — et rien d'autre ne signalerait qu'il rouvre menus, barre système, lecture
//! d'images par chemin ou émission d'événements (`docs/CODE_RULES.md` §10). Toute nouvelle
//! permission doit donc être ajoutée ici, délibérément, avec la fonction qui l'exige.

const CAPABILITY: &str = include_str!("../capabilities/default.json");

/// Permissions sans portée réellement utilisées par l'interface.
const ATTENDUES: &[&str] = &[
    // `listen` / `unlisten` des événements du natif (`docs/ARCHITECTURE.md` § Événements).
    "core:event:allow-listen",
    "core:event:allow-unlisten",
    // Barre de titre dessinée par l'application (`data-tauri-drag-region`).
    "core:window:allow-start-dragging",
    "core:window:allow-internal-toggle-maximize",
];

fn permissions() -> Vec<serde_json::Value> {
    let capability: serde_json::Value = serde_json::from_str(CAPABILITY).unwrap();
    capability["permissions"].as_array().unwrap().clone()
}

#[test]
fn seules_les_permissions_utilisees_sont_accordees() {
    let permissions = permissions();
    let noms: Vec<&str> = permissions
        .iter()
        .filter_map(serde_json::Value::as_str)
        .collect();
    assert_eq!(noms, ATTENDUES);
}

#[test]
fn l_ouverture_d_url_reste_bornee_aux_origines_du_projet() {
    let permissions = permissions();
    let scopees: Vec<&serde_json::Value> = permissions.iter().filter(|p| p.is_object()).collect();
    assert_eq!(scopees.len(), 1);
    assert_eq!(scopees[0]["identifier"], "opener:allow-open-url");
    for entree in scopees[0]["allow"].as_array().unwrap() {
        let url = entree["url"].as_str().unwrap();
        assert!(
            url.starts_with("https://github.com/alexandrebouttierdev/candilog/")
                || url.starts_with("https://www.alexandrebouttier.fr"),
            "{url}"
        );
    }
}
