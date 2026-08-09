# Données locales

La base `candilog.sqlite` est résolue par `core::config::AppPaths`. Une release utilise le dossier historique `com.candilog.desktop` du répertoire de données de l'OS ; un binaire debug utilise obligatoirement `.candilog-dev/` sous le nouveau projet afin de ne jamais ouvrir la base utilisateur pendant le développement. Les cinq migrations historiques sont embarquées et appliquées dans l'ordre via `PRAGMA user_version`.

Les règles de relation restent : entreprise/candidature en `RESTRICT`, candidature/dépendances en `CASCADE`, contact optionnel en `SET NULL`. Les UUID et dates ISO 8601 sont générés en Rust. Les tests n'ouvrent jamais la base utilisateur.

Les sauvegardes doivent utiliser l'API backup SQLite. Une restauration doit valider l'en-tête, ouvrir la base, exécuter `PRAGMA integrity_check`, vérifier les versions puis remplacer la base avec possibilité de retour arrière.
