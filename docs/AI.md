# IA native

Les providers Ollama, Claude, OpenAI, Gemini, Mistral, Nvidia et custom implémentent `LlmProvider`. `AiService` porte le parsing d'offre/CV, la génération, l'ATS, le grounding et les lettres. Le score ATS affiché est toujours le calcul déterministe Rust (`profile_score` / `score_resume_imported`), jamais le chiffre renvoyé par le modèle. La table `ai_cache` existe dans le schéma mais n'est pas encore alimentée ; le bouton « Vider le cache » reste un no-op tant que le cache n'est pas implémenté.

Les opérations longues s'exécutent côté Rust (`spawn_blocking` / Tokio). Le streaming remonte par événements Tauri (`ia-progression`) ; chaque génération possède un `CancellationToken`. L'abandon du futur métier arrête d'attendre le résultat ; la requête HTTP sous-jacente n'est pas encore avortée. La télémétrie reste best-effort et ne masque jamais le résultat métier.
