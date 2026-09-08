//! Cas de test isolé.

use super::*;
use std::io::Write as _;

/// La rotation a lieu au démarrage : sans plafond, une session longue — ou relancée avec
/// `RUST_LOG=debug` — alimente indéfiniment le même fichier. Passé la limite, le journal
/// doit cesser de grossir tout en disant pourquoi, plutôt que de remplir le disque.
#[test]
fn test_le_journal_cesse_d_ecrire_au_dela_du_plafond() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("candilog.log");
    let file = std::fs::File::create(&path).unwrap();
    let mut writer = BoundedLogWriter::new(file);

    let line = vec![b'x'; 64 * 1024];
    // Deux fois le plafond : la seconde moitié ne doit rien ajouter au fichier.
    for _ in 0..(2 * MAX_LOG_BYTES / line.len() as u64) {
        writer.write_all(&line).unwrap();
    }
    writer.flush().unwrap();

    let size = std::fs::metadata(&path).unwrap().len();
    assert!(
        size <= MAX_LOG_BYTES + LIMIT_NOTICE.len() as u64,
        "journal de {size} octets au-delà du plafond de {MAX_LOG_BYTES}"
    );
    let content = std::fs::read(&path).unwrap();
    assert!(
        content.ends_with(LIMIT_NOTICE),
        "le journal doit se terminer par la mention du plafond"
    );
}

/// Une trace peut exceptionnellement contenir un gros payload. Le plafond doit donc
/// s'appliquer à chaque écriture, pas seulement entre deux appels à `write`.
#[test]
fn test_une_ecriture_unique_ne_peut_pas_depasser_le_plafond() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("candilog.log");
    let file = std::fs::File::create(&path).unwrap();
    let mut writer = BoundedLogWriter::new(file);

    writer
        .write_all(&vec![b'x'; MAX_LOG_BYTES as usize + 64 * 1024])
        .unwrap();
    writer.flush().unwrap();

    let content = std::fs::read(&path).unwrap();
    assert_eq!(
        content.len() as u64,
        MAX_LOG_BYTES + LIMIT_NOTICE.len() as u64
    );
    assert!(content.ends_with(LIMIT_NOTICE));
}

/// Un journal encore court s'écrit normalement : le plafond ne doit pas tronquer une
/// session ordinaire.
#[test]
fn test_une_session_ordinaire_ecrit_sans_perte() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("candilog.log");
    let mut writer = BoundedLogWriter::new(std::fs::File::create(&path).unwrap());

    writer.write_all(b"demarrage de Candilog\n").unwrap();
    writer.flush().unwrap();

    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        "demarrage de Candilog\n"
    );
}

/// La taille déjà présente compte : si la rotation a échoué, le fichier repris ne doit pas
/// repartir d'un compteur à zéro.
#[test]
fn test_le_plafond_tient_compte_du_fichier_deja_ecrit() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("candilog.log");
    std::fs::write(&path, vec![b'y'; MAX_LOG_BYTES as usize]).unwrap();
    let file = std::fs::OpenOptions::new()
        .append(true)
        .open(&path)
        .unwrap();
    let mut writer = BoundedLogWriter::new(file);

    writer.write_all(b"une trace de plus\n").unwrap();
    writer.flush().unwrap();

    let content = std::fs::read(&path).unwrap();
    assert!(content.ends_with(LIMIT_NOTICE));
    assert_eq!(
        content.len() as u64,
        MAX_LOG_BYTES + LIMIT_NOTICE.len() as u64
    );
}
