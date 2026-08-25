//! Cas de test isolé.

use super::*;

/// Sans `Serialize` explicite, Tauri rejetterait la représentation `Debug` de l'énumération :
/// le frontend recevrait `Database("timed out waiting for connection: … /home/alex/…")`,
/// c'est-à-dire ni code exploitable, ni phrase présentable, et le chemin local en clair.
#[test]
fn test_serialisation_ipc_expose_code_et_message_utilisateur() {
    let brute = "unable to open database file: /home/alex/.local/share/candilog.sqlite";
    let json = serde_json::to_value(AppError::Database(brute.into())).unwrap();

    assert_eq!(json["code"], "DATABASE_ERROR");
    assert_eq!(
        json["message"],
        "Le fichier de données de Candilog est illisible ou endommagé."
    );
    assert!(!json.to_string().contains("/home/alex"));
}
