Prompt de mission, à jour : `PROMPT_IMPORT_CV.md`.

# Baseline d'import CV

Runner local pour mesurer l'import de profil, sans écraser le profil de l'utilisateur.
Il appelle `AiService::import_profile` sur une base SQLite jetable. Il n'appelle pas
`profile_apply_import`. Aucun CV et aucune donnée personnelle ne sont versionnés :
le corpus et le mapping restent hors du dépôt.

## Lancer

Nécessite Poppler (`pdftotext`) et un Ollama local avec
`LiquidAI/lfm2.5-1.2b-instruct:latest` sur `http://localhost:11434`.

```sh
cd src-tauri
cargo run --example cv_import_baseline -- nom-de-passe
```

Le premier argument est le nom du dossier de sortie, sous
`CV_TESTS/.benchmark/` (hors dépôt). Défaut : `run`.
Le mapping attendu est `CV_TESTS/.benchmark/mapping.json`.

## Ce qui est mesuré

30 PDF, modèle ci-dessus, température 0,7, un seul appel par CV.
Score heuristique moyen / médian / minimum, temps moyen / médian / P95,
réussites et échecs. Identifiants anonymes `CV-001`…`CV-030` seulement.

## Résultat gardé

Extraction PDF dans l'ordre de mise en page (`pdftotext -layout`), puis
recadrage, remplissage exact de l'e-mail et du téléphone vides, et complément
d'une formation seulement si le diplôme ou l'établissement est unique et déjà
écrit dans le texte tronqué.

Sur ce modèle : moyenne 78,0, médiane 75, minimum 53. 28 réussites, 2 échecs.
Temps environ 44 s / 39 s / 83 s (P95). Objectif 75 atteint. 80 non atteint
sans dégrader le score. Pas de règle propre à un CV.
