# Audit de migration — Candilog Desktop vers Rust + Iced

## Références et état initial

| Référence | Chemin | Révision observée | État initial | Politique |
|---|---|---|---|---|
| Candilog Desktop | `/home/alex/Documents/Mes projets/candilog-desktop` | `45abfef6ee6dc308230c3f3af95634d8095e8622`, branche `dev` | modification préexistante de `__pycache__/migrate.cpython-314.pyc` | lecture seule absolue |
| AfterBudget | `/home/alex/Documents/Mes projets/afterbudget` | `af2735c666d1485762e92c7946e8aa00d1af77e1`, branche `master` | propre | lecture seule absolue |

Cet état est la référence du contrôle final. Aucun build ni migration n'est lancé dans ces deux dossiers.

## Produit réellement livré dans l'ancienne application

### Navigation et pages

1. Tableau de bord (`/`) : statistiques de synthèse, candidatures récentes, prochains entretiens.
2. Candidatures (`/candidatures`) : Kanban/liste, recherche, six catégories de filtres, CRUD, détail, statut, export CSV, entretiens et relances associés.
3. Mes CV (`/cv`) : liste, chargement, suppression et versions de CV.
4. Entreprises (`/entreprises`) : liste, recherche, CRUD, fiche et relations.
5. Réseau (`/reseau`) : liste, recherche, CRUD contact et fiche.
6. Calendrier (`/calendrier`) : événements issus des entretiens et relances, navigation et formulaires associés.
7. Statistiques (`/statistiques`) : entonnoir candidature, statistiques CV/ATS, IA, providers, modèles et latences.
8. CV Generator (`/cv-generator`) : analyse d'offre, score, génération, recommandations, édition, aperçu, versions et export.
9. Lettre de motivation (`/lettre-motivation`) : analyse de demande, génération progressive, annulation et édition.
10. Analysez un CV (`/cv-import`) : import PDF, offre, analyse ATS en lecture seule.
11. Mon Profil (`/profil`) : sept sections structurées et import intelligent de CV par suggestions.
12. Paramètres (`/parametres`) : provider IA, modèle, endpoint, clé, thème, sauvegarde/restauration/reset et updater.

### Composants et interactions inventoriés

- Shell : `AppShell`, `Titlebar`, `Sidebar`, `PageHeader`, `AppVersion`, `RuntimeStatus`, gestion de thème et vérification de mise à jour.
- Retours visuels : `StatCard`, `ScoreGauge`, `BarreProportion`, skeletons, loader, chronomètre, progression IA, notifications et frontière d'erreur.
- Overlays : `Drawer`, `OverlayPortal`, en-tête de formulaire, modales candidatures/entreprises/contacts/entretiens/relances, détail candidature/entreprise/contact, import profil.
- Formulaires : champs date/date-heure, collage presse-papiers, inputs, labels, boutons, cartes et onglets.
- Candidatures : `KanbanBoard`, `CandidatureListe`, `FiltersPanel`, recherche, filtre statut/contrat/entreprise/ville/poste/période et export borné par les filtres.
- CV : éditeur structuré, aperçu, template, export, versions, acceptation/refus des recommandations, code couleur score.
- Profil : informations personnelles, expériences, compétences, formations, langues, projets et certifications ; détection de doublons et fusion/remplacement/conservation/ignorance.
- Statistiques : entonnoir, agrégats ATS par tranche/origine, appels IA par opération/provider/modèle, pagination.

### Modules Rust réutilisables

| Module | Modèles | Service | Repository SQLite | Particularités |
|---|---|---|---|---|
| candidatures | oui | oui | oui | validation, changement statut, jointure entreprise, CSV dans l'ancienne commande |
| entreprises | oui | oui | oui | suppression restreinte si candidature liée |
| contacts | oui | oui | oui | suppression applicative refusée si contact référencé |
| entretiens | oui | oui | oui | analyse IA persistée, contact facultatif |
| relances | oui | oui | oui | cascade avec candidature |
| cv | oui | oui | oui | contenu JSON opaque et résumés |
| profil | types partagés | oui | oui | singleton JSON, validation e-mail et expériences |
| settings | oui | oui | oui | singleton JSON, clé API dans coffre natif |
| metriques | oui | sans service | oui | appels LLM, scores ATS, agrégats et pagination |
| ia | oui | `CvEngine` | cache SQLite | providers Ollama, Claude, OpenAI, Gemini, Mistral, Nvidia et custom |

### Commandes publiques historiques (48)

- Paramètres : `get_settings`, `update_settings`, `export_backup`, `import_backup`, `reset_database`.
- Profil : `get_profil`, `update_profil`.
- IA : `test_llm_connection`, `list_llm_models`, `analyze_offer`, `generate_cv`, `analyze_imported_cv`, `extract_cv_profile`, `analyser_entretien`, `analyser_demande_lettre`, `generer_lettre_motivation`, `annuler_generation`, `reset_cache_ia`.
- CV : `save_cv_version`, `list_cv_versions`, `load_cv_version`, `delete_cv_version`.
- Candidatures : `create_candidature`, `list_candidatures`, `update_candidature`, `update_statut_candidature`, `delete_candidature`, `export_candidatures_csv`.
- Entreprises : `list_entreprises`, `create_entreprise`, `update_entreprise`, `delete_entreprise`.
- Contacts : `list_contacts`, `create_contact`, `update_contact`, `delete_contact`.
- Relances : `list_relances`, `create_relance`, `update_relance`, `delete_relance`.
- Entretiens : `list_entretiens`, `create_entretien`, `update_entretien`, `delete_entretien`.
- Métriques : `list_llm_appels`, `list_scores_ats`, `reset_llm_appels`, `reset_scores_ats`.

Dans la nouvelle application, ces points d'entrée deviennent des messages Iced, tâches Tokio et appels directs aux services. Aucun pont IPC n'est nécessaire.

## Données et migrations

Les cinq migrations historiques sont copiées sans modification :

1. `001_tables_locales.sql` : `llm_appels`, `scores_ats`, `cache_ia`, `app_kv`.
2. `002_purge_score_offre.sql` : suppression de la télémétrie retirée.
3. `003_drop_offres.sql` : retrait du moteur de flux d'offres.
4. `004_schema_metier.sql` : `entreprises`, `contacts`, `candidatures`, `statut_history`, `relances`, `entretiens`, `cv_versions`, `parametres`, `profil` et index.
5. `005_contraintes_enum.sql` : contraintes candidatures et entretiens.

Contraintes conservées : UUID texte, dates ISO 8601 côté Rust, singleton `CHECK (id = 1)`, `RESTRICT` entreprise→candidature, `CASCADE` candidature→historique/relance/entretien et `SET NULL` pour les contacts. `PRAGMA foreign_keys=ON`, WAL et timeout 5 secondes sont appliqués à chaque connexion du pool.

Le chemin actif reste `candilog.sqlite` sous le dossier de données applicatif multiplateforme. Les tests utilisent uniquement une base mémoire ou un dossier temporaire.

## IA, PDF et système

- Trait `LlmProvider` et factory multi-provider conservés.
- `CvEngine`, scoring déterministe, grounding, cache, modes Small/Standard/Advanced, retry et parsing JSON conservés.
- PDF : extraction locale conservée ; export remplacé par un document PDF natif issu de `CvLayout` commun.
- Secrets : crate `keyring`, jamais de clé API SQLite.
- Annulation : `CancellationToken`, abandon du futur HTTP.
- Notifications : `notify-rust` retenu.
- Updater : manifeste GitLab public, comparaison SemVer, téléchargement, vérification minisign, installation adaptée à la plateforme et redémarrage à recréer sans plugin.

## Tests recensés

- Ancienne application : 299 annotations de tests Rust et 52 fichiers de tests TypeScript/TSX.
- Nouvelle application : tous les tests Rust réutilisables sont conservés ; les calculs autrefois TypeScript doivent avoir leurs équivalents Rust ciblés.

## Conventions reprises d'AfterBudget

- `app/{state,message,update,view}` pour la boucle Iced.
- `core/` pour chemins, base, erreurs, updater et système.
- `modules/<domaine>/{model,dto,repository,service,view}`.
- `navigation::Route` typé.
- `ui/{theme,components,layouts}` sans SQL.
- Flux : vue → message → update → service → repository → SQLite.
- DTO/validation indépendants des vues et tests sur base temporaire.

