# Mission — Import CV Candilog avec LFM2.5 1.2B

État au 2026-09-09. Ce document remplace le prompt de départ. Il décrit ce qui est déjà fait, ce qui est gardé, et ce qu'il ne faut pas refaire.

Le but reste le même : dans Candilog elle-même, un import de profil fiable et rapide depuis un vrai PDF, avec uniquement

```text
LiquidAI/lfm2.5-1.2b-instruct:latest
```

via Ollama local (`http://localhost:11434`).

Le corpus est hors dépôt :

```text
/home/alex/Documents/CV_TESTS
```

30 PDF. Identifiants anonymes `CV-001`…`CV-030` seulement. Ne jamais copier les CV dans git, ni écrire de noms, e-mails, téléphones ou adresses dans un rapport.

---

# État gardé

Branche locale `OPTIMISATION_IMPORT_CV`, commit `e8b81d5`. Pas de push.

Pipeline réel, un seul appel :

```text
PDF
→ pdftotext -layout (repli : extracteur de flux)
→ texte tronqué à 12 000 caractères
→ invite_import de HEAD, température 0,7
→ parsing / réparation
→ grounding (recopie exacte seulement)
→ e-mail et téléphone vides remplis seulement si la chaîne est déjà dans le texte
→ formation manquante ajoutée seulement si le diplôme ou l'établissement est unique et exact
→ nettoyage
```

Le prompt produit n'a pas été rallongé. Un exemple fictif dans le message a fait chuter la moyenne à 61,5 et a été annulé.

Runner, même code métier, sans appliquer le profil :

```text
src-tauri/examples/cv_import_baseline.rs
```

Note courte : `src-tauri/examples/cv_import_baseline.md`.

---

# Scores

Modèle : `LiquidAI/lfm2.5-1.2b-instruct:latest`. Température 0,7. Un appel par CV.

| Version | Score moyen | Médiane | Min | Temps moyen | Médiane | P95 | OK / échecs | Décision |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- | --- |
| Baseline | 54,9 | 53 | 17 | ~45,3 s | — | ~100 s | 27 / 3 | référence |
| Contact e-mail / téléphone | 69,7 | 72 | 37 | 40,1 s | 37,8 s | 64,7 s | 28 / 2 | gardé, puis dépassé |
| + formation unique | 71,9 | 73,5 | 43 | 43,1 s | 39,5 s | 71,5 s | 28 / 2 | gardé, puis dépassé |
| + ordre de mise en page | 78,0 | 75 | 53 | 44,3 s | 39,0 s | 83,3 s | 28 / 2 | gardé |

```text
SCORE HEURISTIQUE MOYEN : 78,0 / 100
TEMPS MOYEN PAR CV : 44,3 secondes
```

Objectif principal 75 : atteint. Objectif 80 : non atteint sur ce modèle, sans rejouer un essai déjà perdu. Le minimum 53 est sous le plancher visé de 60.

Deux échecs restants : un PDF sans couche texte (validation avant le modèle), et un JSON illisible après réparation. La troncature à 12 000 caractères ne se déclenche pas (maximum observé 6123).

---

# Ce qui a été annulé

Ne pas refaire ces essais tels quels. Ils ont baissé le score, ajouté un échec, ou n'ont pas bougé le trou utile.

- Prompt d'expériences rallongé.
- Température 0.
- Étiquettes d'expériences injectées dans l'entrée du modèle.
- Second appel LLM, même seulement pour les expériences.
- Préfixer le texte avec des extraits datés.
- Remplacer l'entrée par des tranches datées.
- Récupérer des titres après coup : le score compte les lignes, pas le texte du titre.
- Compléter les expériences à partir des plages d'années, même de façon stricte.
- Exemple few-shot fictif : moyenne 61,5, expériences effondrées.

Le trou restant est le nombre d'expériences émises par le modèle, face aux plages d'années du texte. Créer des lignes à sa place a déjà perdu.

---

# Règles qui restent valables

1. Ne jamais inventer. Une info absente vaut mieux qu'une info inventée.
2. Ne jamais rattacher une donnée au mauvais poste ou au mauvais diplôme.
3. Robustesse : 0 crash, 0 panic, 0 import bloqué, 0 JSON invalide final.
4. Expériences, puis formations, puis identité, puis compétences. Une compétence manquante est moins grave qu'une compétence inventée.
5. Afficher toujours le score avec le temps (moyen, médiane, P95).
6. Garder un changement seulement s'il monte le score sans hausse disproportionnée du temps, sans hallucination en plus, et sans échec critique en plus. Sinon, revert.
7. Pas de règle propre à un CV (`si fichier == …`).
8. Tout candidat sérieux se mesure sur les 30 PDF, via `AiService::import_profile`. Ne pas écrire le profil de l'utilisateur.
9. Ne pas committer sans demande. Messages en français. Ne pas committer `CV_TESTS`.

---

# Si on reprend

Un plateau a déjà été déclaré pour 80 sur ce modèle. Ne relance pas la boucle avec une variante des essais annulés.

Le seul levier restant nommé : un autre modèle local, sur demande explicite. Si ce modèle change, refaire la baseline sur les 30 PDF avant de comparer.

Question toujours ouverte, pour un CV inconnu :

> Est-ce qu'un utilisateur peut importer un vrai CV dans Candilog et obtenir un profil fiable avec ce petit modèle, en environ 45 s ?

Pour la version gardée, la réponse mesurée est oui sur le score moyen (78), non sur le minimum (53) et non sur 80.
