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
contenus non fiables sont encadrés par `bloc_donnees`, dont la balise porte un identifiant
tiré au sort à chaque appel et dont la balise fermante est neutralisée dans le contenu — un
délimiteur fixe pouvait figurer dans l'offre elle-même et refermer le bloc.

Le récapitulatif et les suggestions de `AtsAnalysis` restent du texte libre du modèle :
bornés, mais non recadrés sur les faits. L'interface les présente comme un commentaire, à
côté du score, qui est calculé par Candilog.

La lettre de motivation est **assemblée**, pas rédigée : le modèle ne renvoie qu'une
sélection d'identifiants du catalogue de faits et des mots-clés du brief
(`domain/cover_letter.rs`). Un identifiant inconnu invalide la réponse ; un mot-clé absent
du brief est simplement écarté, parce qu'une paraphrase du modèle ne justifie pas de faire
échouer toute la rédaction — la lettre reste dans tous les cas limitée aux faits vérifiés.

Le score ATS affiché est toujours le calcul déterministe Rust (`profile_score` /
`score_resume_imported`, `domain/scoring.rs`), jamais le chiffre renvoyé par le modèle.

## Progression et annulation

Les traitements sont asynchrones côté Rust. La progression remonte par événements Tauri :
`ia-progression` pour la génération et l'analyse, `profile_import_progress` pour l'import
de profil. La télémétrie est best-effort et ne masque jamais le résultat métier.

La fin d'un traitement est aussi annoncée par un signal sonore, émis une seule fois dans
`features/ai/services/aiService.ts` pour que plus aucun écran ne puisse l'oublier. La
préférence « Son de fin de traitement » (Réglages → IA) est locale à la machine
(`shared/lib/completion-sound.ts`, `localStorage`) et active par défaut.

Chaque génération possède un `CancellationToken`, indexé par `generation_id` ;
`ai_cancel` le déclenche. L'annulation abandonne le futur en cours — la requête HTTP est
portée par ce futur, elle s'interrompt donc avec lui. Relancer une génération avec un
identifiant déjà actif annule la précédente.

## Cache

Il n'y a pas de cache de réponses IA. La table `ai_cache`, la commande
`settings_clear_ai_cache` et le bouton « Vider le cache IA » ont été retirés : rien
n'alimentait la table, et l'écran des réglages annonçait à l'utilisateur un effet qui ne se
produisait jamais. Une base de développement antérieure conserve la table, désormais
orpheline et sans usage.
