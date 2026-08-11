//! Sauvegarde et validation des bases SQLite Candilog.

use crate::shared::db::{SqlitePool, DERNIERE_VERSION};
use crate::shared::error::{AppError, AppResult};
use std::io::Read;
use std::path::{Path, PathBuf};

/// Exporte un instantané cohérent via l'API backup SQLite.
///
/// # Errors
/// Retourne une erreur si la source ou la destination ne peuvent pas être ouvertes.
pub fn export(pool: &SqlitePool, destination: &Path) -> AppResult<()> {
    let source = pool
        .get()
        .map_err(|error| AppError::Database(error.to_string()))?;
    source.execute_batch("PRAGMA wal_checkpoint(PASSIVE);")?;
    let mut target = rusqlite::Connection::open(destination)?;
    let backup = rusqlite::backup::Backup::new(&source, &mut target)?;
    backup
        .run_to_completion(5, std::time::Duration::from_millis(100), None)
        .map_err(AppError::from)
}

/// Vérifie qu'un fichier est une base SQLite Candilog intègre et compatible.
///
/// # Errors
/// Retourne une erreur si l'en-tête, l'intégrité ou les tables indispensables sont invalides.
pub fn validate(path: &Path) -> AppResult<()> {
    // Seize octets suffisent : lire le fichier entier allouerait sa taille en mémoire vive
    // avant même de savoir s'il s'agit d'une base, et `import()` paierait le coût deux fois.
    let mut entete = [0_u8; 16];
    let mut fichier = std::fs::File::open(path)
        .map_err(|error| AppError::Database(format!("Impossible de lire le backup : {error}")))?;
    // Un fichier trop court n'est pas une erreur de lecture mais un fichier non conforme :
    // `UnexpectedEof` rejoint donc le même refus que l'en-tête erroné.
    let trop_court = matches!(
        fichier.read_exact(&mut entete),
        Err(ref error) if error.kind() == std::io::ErrorKind::UnexpectedEof
    );
    if trop_court || &entete != b"SQLite format 3\0" {
        return Err(AppError::Validation(
            "Le fichier sélectionné n'est pas une base SQLite.".into(),
        ));
    }
    let connection =
        rusqlite::Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let integrity: String = connection.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
    if integrity != "ok" {
        return Err(AppError::Validation(format!(
            "Le backup SQLite est corrompu : {integrity}"
        )));
    }
    for table in [
        "candidatures",
        "entreprises",
        "contacts",
        "parametres",
        "profil",
    ] {
        let count: i64 = connection.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
            [table],
            |row| row.get(0),
        )?;
        if count != 1 {
            return Err(AppError::Validation(format!(
                "Le backup ne contient pas la table Candilog {table}."
            )));
        }
    }
    // Un backup issu d'une version plus récente porte un schéma inconnu de celle-ci : les
    // migrations rejouées à l'import n'ont plus rien à appliquer (leur boucle ignore les cibles
    // déjà atteintes) et les lectures se feraient sur un schéma non maîtrisé.
    let version: i64 = connection.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    if version > DERNIERE_VERSION {
        return Err(AppError::Validation(format!(
            "Ce backup a été créé par une version plus récente de Candilog (schéma {version}, \
             cette version gère le schéma {DERNIERE_VERSION}). Installez la mise à jour de \
             Candilog pour pouvoir le restaurer."
        )));
    }
    Ok(())
}

/// Remplace le contenu de la base active par un backup validé, avec retour arrière.
///
/// L'API backup SQLite écrase la cible **en place** : une interruption en cours de route
/// (disque plein, erreur d'E/S, source devenue illisible) laisserait la base à moitié écrasée,
/// sans backup appliqué pour autant. Une copie de l'état courant est donc prise avant toute
/// écriture, et remise en place si la restauration échoue.
///
/// La copie est conservée à côté de la base (`candilog.sqlite.bak`) : elle constitue le dernier
/// recours si le retour arrière lui-même échoue.
///
/// # Errors
/// Retourne une erreur si le backup est invalide, ou si la restauration échoue — auquel cas la
/// base active a été remise dans son état antérieur.
pub fn import(pool: &SqlitePool, db_path: &Path, source_path: &Path) -> AppResult<()> {
    validate(source_path)?;
    let secours = chemin_de_secours(db_path);
    export(pool, &secours)?;
    tracing::info!("copie de secours prise avant restauration");

    let Err(echec) = remplacer(pool, source_path) else {
        tracing::info!("backup restauré");
        return Ok(());
    };
    tracing::error!(erreur = %echec, "restauration échouée, retour arrière");

    match remplacer(pool, &secours) {
        Ok(()) => Err(AppError::Validation(
            "Le backup n'a pas pu être restauré. La base d'origine a été restaurée : vos \
             données sont intactes."
                .into(),
        )),
        Err(perte) => {
            tracing::error!(erreur = %perte, "retour arrière échoué");
            Err(AppError::Database(format!(
                "Le backup n'a pas pu être restauré et la base d'origine n'a pas pu être \
                 remise en place. Une copie intacte de vos données est conservée dans {}.",
                secours.display()
            )))
        }
    }
}

/// Écrase la base du pool par le contenu d'un fichier, puis remet le schéma à niveau.
fn remplacer(pool: &SqlitePool, source_path: &Path) -> AppResult<()> {
    {
        let source = rusqlite::Connection::open_with_flags(
            source_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )?;
        let mut target = pool
            .get()
            .map_err(|error| AppError::Database(error.to_string()))?;
        let backup = rusqlite::backup::Backup::new(&source, &mut target)?;
        backup
            .run_to_completion(5, std::time::Duration::from_millis(100), None)
            .map_err(AppError::from)?;
    }
    crate::shared::db::run_local_migrations(pool)
}

/// Emplacement de la copie de secours. Les pools en mémoire (tests) n'ayant pas de fichier,
/// leur copie va dans le dossier temporaire du système.
fn chemin_de_secours(db_path: &Path) -> PathBuf {
    if db_path.as_os_str().is_empty() {
        std::env::temp_dir().join(format!("candilog-secours-{}.sqlite", uuid::Uuid::new_v4()))
    } else {
        db_path.with_extension("sqlite.bak")
    }
}

/// Restaure un backup **au niveau du fichier**, sans base ouverte.
///
/// C'est le chemin de secours de l'écran d'erreur fatale : quand la base active est illisible,
/// aucun pool n'existe, donc ni [`import`] ni [`reset_data`] ne sont utilisables. La copie se
/// fait alors sur le système de fichiers, après la même validation.
///
/// Les journaux WAL de l'ancienne base sont supprimés : conservés, ils seraient rejoués
/// par-dessus le contenu restauré.
///
/// # Errors
/// Retourne une erreur si le backup est invalide ou si la copie échoue.
pub fn restore_file(source: &Path, database: &Path) -> AppResult<()> {
    validate(source)?;
    if database.exists() {
        mettre_de_cote(database)?;
    }
    std::fs::copy(source, database).map_err(|error| {
        AppError::Database(format!(
            "Impossible d'écrire le fichier de données : {error}"
        ))
    })?;
    tracing::info!("backup restauré au niveau du fichier");
    Ok(())
}

/// Met une base illisible de côté pour permettre un redémarrage sur une base neuve.
///
/// La base n'est **jamais supprimée** : elle est renommée avec un horodatage, de sorte qu'une
/// récupération ultérieure reste possible même si l'application ne sait plus l'ouvrir.
///
/// # Errors
/// Retourne une erreur si le fichier ne peut pas être renommé.
pub fn quarantine(database: &Path) -> AppResult<PathBuf> {
    let destination = mettre_de_cote(database)?;
    tracing::warn!(?destination, "base mise de côté");
    Ok(destination)
}

/// Renomme la base et écarte ses journaux WAL. Renvoie le chemin de la copie conservée.
fn mettre_de_cote(database: &Path) -> AppResult<PathBuf> {
    let horodatage = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let destination = database.with_extension(format!("sqlite.{horodatage}.ancienne"));
    std::fs::rename(database, &destination).map_err(|error| {
        AppError::Database(format!(
            "Impossible de mettre l'ancienne base de côté : {error}"
        ))
    })?;
    for suffixe in ["-wal", "-shm"] {
        let mut journal = database.as_os_str().to_os_string();
        journal.push(suffixe);
        // Absent la plupart du temps : son absence n'est pas une erreur.
        let _ = std::fs::remove_file(PathBuf::from(journal));
    }
    Ok(destination)
}

/// Vide toutes les données utilisateur en conservant le schéma et ses migrations.
///
/// # Errors
/// Retourne une erreur si la transaction SQLite échoue.
pub fn reset_data(pool: &SqlitePool) -> AppResult<()> {
    let mut connection = pool
        .get()
        .map_err(|error| AppError::Database(error.to_string()))?;
    let transaction = connection.transaction()?;
    transaction.execute_batch(
        "DELETE FROM relances;
         DELETE FROM entretiens;
         DELETE FROM statut_history;
         DELETE FROM candidatures;
         DELETE FROM contacts;
         DELETE FROM entreprises;
         DELETE FROM cv_versions;
         DELETE FROM profil;
         DELETE FROM parametres;
         DELETE FROM llm_appels;
         DELETE FROM scores_ats;
         DELETE FROM cache_ia;
         DELETE FROM app_kv;",
    )?;
    transaction.commit()?;
    Ok(())
}

#[cfg(test)]
#[path = "tests/backup/mod.rs"]
mod tests;
