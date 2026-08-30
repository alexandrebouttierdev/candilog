# IA native

Toute l'IA vit dans `src-tauri/src/features/ai/`. Le frontend n'envoie que des DTO et
écoute la progression : **aucun prompt dans React**.

## Fournisseurs

Ollama, Claude, OpenAI, Gemini, Mistral, Nvidia et un point de terminaison personnalisé
implémentent `LlmProvider` (`infrastructure/provider.rs`). Le choix, le modèle, la
température et le mode d'analyse sont persistés dans les paramètres ; la clé API vit dans
le coffre du système (`core::secrets`), jamais dans SQLite ni dans les journaux.

HTTPS obligatoire hors Ollama, adresses privées refusées pour un point de terminaison
distant, réponse plafonnée à 5 Mio, PDF source plafonné à 10 Mio.

## Sorties du modèle

`AiService` porte le parsing d'offre et de CV, la génération, l'ATS, le grounding et les
lettres. La chaîne est toujours **parse → validate → grounding** : le JSON brut du modèle
n'est jamais utilisé tel quel, il est réparé si besoin (`jsonrepair-rs`), désérialisé,
borné (`domain/validation.rs` : `MAX_SOURCE_CHARS`, `MAX_CONTEXT_CHARS`,
`MAX_STRUCTURED_CHARS`, `MAX_ITEMS`, `MAX_ITEM_CHARS`) puis recadré sur les faits réels
par `ground_generated_resume`, `ground_imported_resume` et `ground_extracted_listing`.

Une offre d'emploi ou un PDF importé est de la **donnée**, jamais des instructions : les
contenus non fiables sont encadrés par `bloc_donnees`.

Le score ATS affiché est toujours le calcul déterministe Rust (`profile_score` /
`score_resume_imported`, `domain/scoring.rs`), jamais le chiffre renvoyé par le modèle.

## Progression et annulation

Les traitements sont asynchrones côté Rust. La progression remonte par événements Tauri :
`ia-progression` pour la génération et l'analyse, `profile_import_progress` pour l'import
de profil. La télémétrie est best-effort et ne masque jamais le résultat métier.

Chaque génération possède un `CancellationToken`, indexé par `generation_id` ;
`ai_cancel` le déclenche. L'annulation abandonne le futur en cours — la requête HTTP est
portée par ce futur, elle s'interrompt donc avec lui. Relancer une génération avec un
identifiant déjà actif annule la précédente.

## Cache

La table `ai_cache` existe dans le schéma mais rien ne l'alimente aujourd'hui. La commande
`settings_clear_ai_cache` la vide réellement (`DELETE FROM ai_cache`) ; tant que le cache
n'est pas implémenté, l'opération n'a aucun effet observable.
