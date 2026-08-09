# IA native

Les providers Ollama, Claude, OpenAI, Gemini, Mistral, Nvidia et custom implémentent `LlmProvider`. `CvEngine` porte le parsing d'offre/CV, la génération, l'ATS, le grounding et les lettres. Les appels déterministes utilisent le cache SQLite ; la génération créative n'est jamais mise en cache.

Les opérations Iced longues sont des `Task`. Le streaming utilise un canal Tokio et une `Subscription`; chaque génération possède un `CancellationToken`. L'abandon du futur réseau doit fermer la requête. La télémétrie reste best-effort et ne masque jamais le résultat métier.

