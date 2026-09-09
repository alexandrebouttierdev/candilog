# Mission — Optimiser l’import de CV de Candilog avec LFM2.5 1.2B

Ce prompt est le brief complet. N’importe qui peut s’en servir pour continuer à améliorer l’import. Les règles ci-dessous restent toutes valables.

## Point de départ actuel (2026-09-09)

Ne repars pas de zéro, et n’annule pas le code déjà gardé sans un benchmark qui le bat.

Le code gardé est sur la branche d’optimisation d’import. Ne le revert pas sans un benchmark qui le bat.

Le code produit contient déjà :

- extraction PDF dans l’ordre de mise en page (`pdftotext -layout`, repli sur l’extracteur de flux) ; le modèle et le grounding voient le même texte ;
- remplissage d’un e-mail ou d’un téléphone vide seulement si la chaîne est déjà dans le texte tronqué ;
- ajout d’une formation manquante seulement si le diplôme ou l’établissement est unique et exact dans ce texte ;
- `invite_import` inchangé, température 0,7, un seul appel LLM.

Runner qui appelle le vrai `AiService::import_profile`, sans appliquer le profil :

```text
src-tauri/examples/cv_import_baseline.rs
```

Note courte : `src-tauri/examples/cv_import_baseline.md`.

Référence actuelle, sur les 30 PDF de `CANDILOG_CV_DIR`, modèle `LiquidAI/lfm2.5-1.2b-instruct:latest` :

```text
Score moyen : 78,0 / 100
Score médian : 75
Score minimum : 53
Temps moyen : 44,3 s / CV
Temps médian : 39,0 s
P95 : 83,3 s
OK / échecs : 28 / 2
```

Baseline historique, avant ces changements, pour comparaison :

```text
Score moyen : 54,9 / 100
Score médian : 53
Score minimum : 17
Temps moyen : ~45,3 s / CV
P95 : ~100 s
OK / échecs : 27 / 3
```

Objectif principal 75 : atteint. Objectif secondaire 80 : pas encore atteint. Minimum visé 60 : pas encore atteint (53).

Avant de modifier, remesure cet état actuel sur tous les PDF. C’est ta nouvelle référence, pas la baseline à 54,9.

Essais déjà perdus, ne les refais pas tels quels (ils ont baissé le score, ajouté un échec, ou n’ont pas bougé le trou utile) :

- prompt d’expériences rallongé ;
- température 0 ;
- étiquettes d’expériences injectées dans l’entrée du modèle ;
- second appel LLM ;
- préfixer le texte avec des extraits datés ;
- remplacer l’entrée par des tranches datées ;
- récupérer des titres après coup (le score compte les lignes, pas le texte du titre) ;
- compléter les expériences à partir des plages d’années ;
- exemple few-shot fictif (moyenne tombée à 61,5).

Le trou restant est surtout le nombre d’expériences émises par le modèle, face aux plages d’années du texte. Un PDF sans couche texte échoue en validation avant le modèle. Un autre renvoie parfois un JSON illisible. La troncature à 12 000 caractères ne se déclenche pas (maximum observé 6 123).

Tu peux inventer une autre piste. Tu ne dois pas committer sans demande, ni versionner les CV.

---


Tu travailles directement dans le projet **Candilog** présent dans le dossier courant.

L’objectif est d’améliorer de manière intensive, mesurée et itérative la fonctionnalité existante :

**Profil → Importer depuis un CV → extraction des informations → création/remplissage du profil**

Le modèle IA à utiliser pour TOUS les tests est exclusivement :

```text
LiquidAI/lfm2.5-1.2b-instruct:latest
```

via **Ollama local**.

Le runner peut viser un autre modèle Ollama avec `CANDILOG_CV_MODEL`
(`CANDILOG_OLLAMA_URL`, `CANDILOG_CV_TEMPERATURE` en option).
Le score 78,0 / 44,3 s est celui du modèle par défaut. Un autre modèle
a sa propre référence : indique toujours le nom du modèle à côté du score.

Le corpus de test contient environ **30 CV réels au format PDF** et se trouve ici :

```text
CANDILOG_CV_DIR
```

Tu dois utiliser directement les fichiers présents dans ce dossier.

Le but n’est PAS de créer un prototype indépendant.

Le but est d’obtenir **dans Candilog elle-même** une extraction de profil aussi fiable et rapide que possible avec ce petit modèle de 1,2B.

---

# Objectifs principaux

Nous voulons optimiser simultanément :

```text
1. QUALITÉ DE L'IMPORT
2. FIABILITÉ
3. ABSENCE D'HALLUCINATIONS
4. TEMPS D'IMPORT
```

La baseline historique, avant optimisation, a été mesurée :

```text
Score heuristique moyen : 54,9 / 100
Temps moyen par CV : ~45,3 s
```

La référence à battre aujourd’hui est l’état gardé : 78,0 / 100 en 44,3 s / CV. Voir le point de départ en tête de ce fichier.

Le temps moyen doit toujours être remesuré, jamais repris d’un ancien tableau sans le revérifier.

À partir de ce moment, **score moyen et temps moyen doivent toujours être affichés ensemble**.

Exemple :

```text
Score moyen : 72.4 / 100
Temps moyen : 28.7 s / CV
```

---

# Objectifs de qualité

Utilise les paliers suivants :

```text
< 50     = mauvais
50–60    = baseline / prototype
60–70    = amélioration notable mais insuffisante
70–75    = bon
75–80    = très bon
80–90    = excellent pour un modèle 1.2B
> 90     = exceptionnel
```

## Objectif principal

```text
Score heuristique moyen >= 75 / 100
```

## Objectif secondaire

Si cela reste possible sans détériorer les autres critères :

```text
Score heuristique moyen >= 80 / 100
```

Une fois 75 atteint, continue les itérations tant que des améliorations raisonnables et mesurables restent possibles.

Ne t'arrête pas arbitrairement à 75 si le système peut encore progresser.

---

# Objectif de stabilité

Le score moyen ne doit jamais masquer des CV complètement ratés.

Objectif minimum :

```text
Score minimum par CV >= 60 / 100
```

Objectif préférable :

```text
Score minimum par CV >= 65 / 100
```

Je préfère :

```text
Moyenne : 77
Minimum : 65
Hallucinations : quasi 0
```

à :

```text
Moyenne : 83
Minimum : 31
Plusieurs hallucinations
```

La stabilité du système est prioritaire.

---

# Temps d'import — KPI obligatoire

Le temps d'import est un critère essentiel.

Une optimisation qui augmente le score mais rend l'import beaucoup trop lent ne doit pas automatiquement être considérée comme meilleure.

Pour **chaque CV et chaque itération**, mesure autant que possible :

```text
temps total d'import
temps extraction PDF
temps prétraitement
temps total Ollama
temps de chaque appel Ollama
temps parsing
temps validation
nombre d'appels LLM
tokens d'entrée
tokens de sortie
tokens/s
```

Le temps total doit correspondre au temps réellement perceptible par l'utilisateur :

```text
début de l'import
↓
lecture PDF
↓
prétraitement
↓
appel(s) Ollama
↓
parsing
↓
validation
↓
profil prêt
```

---

# Métriques temporelles obligatoires

Après chaque benchmark complet, calcule :

```text
Temps moyen par CV
Temps médian par CV
Temps minimum
Temps maximum
P95
Temps total du benchmark
```

Le **temps moyen par CV est obligatoire dans chaque rapport d'itération**.

Exemple :

```text
VERSION v4

Score moyen : 76.8 / 100
Score médian : 78.1 / 100
Score minimum : 64.2 / 100

Temps moyen : 31.8 s / CV
Temps médian : 29.7 s / CV
P95 : 48.2 s
Temps maximum : 53.7 s
Temps total benchmark : 15 min 54 s

Appels LLM moyens : 2
```

---

# Rapport qualité / vitesse

Pour chaque modification importante, compare explicitement :

```text
gain de qualité
VS
coût en temps
```

Exemple :

```text
v1

Score moyen : 54
Temps moyen : 24 s
```

Puis :

```text
v2

Score moyen : 67
Temps moyen : 27 s
```

Résultat :

```text
+13 points
+3 secondes

→ excellente amélioration
```

Autre exemple :

```text
v3

Score moyen : 76
Temps moyen : 34 s

→ gain important, surcoût probablement acceptable
```

Autre exemple :

```text
v4

Score moyen : 77
Temps moyen : 95 s
```

Comparé à v3 :

```text
+1 point
+61 secondes

→ mauvais compromis
```

Dans ce cas, préfère normalement v3.

---

# Règle fondamentale — Utiliser Candilog réellement

Tu dois utiliser le véritable pipeline de Candilog :

```text
PDF réel
↓
lecture/extraction du document par Candilog
↓
préparation du contexte
↓
prompt réel de Candilog
↓
Ollama
↓
LiquidAI/lfm2.5-1.2b-instruct:latest
↓
parsing
↓
validation
↓
DTO / modèles Candilog
↓
profil importé
```

Tu dois identifier et utiliser :

- la fonctionnalité existante d'import de CV ;
- les services Rust concernés ;
- les commandes Tauri concernées ;
- les services frontend concernés ;
- le provider Ollama ;
- le prompt réellement utilisé ;
- les DTO ;
- les schémas de validation ;
- le parsing de la réponse ;
- la transformation vers le profil Candilog.

---

# Interdiction de créer un faux benchmark

Ne crée PAS un script indépendant qui fait :

```text
PDF
→ Ollama
→ JSON
```

en dehors de Candilog pour prétendre que l'import fonctionne.

Un outil de benchmark peut être créé si nécessaire, mais il doit appeler **exactement le même code métier que Candilog**.

Idéalement :

```text
importProfileFromResume(...)
```

doit pouvoir être appelé depuis :

```text
Interface Candilog
Benchmark
Tests
```

sans duplication de logique.

La vérité finale reste :

> Est-ce que le CV réel est correctement importé dans Candilog ?

---

# Corpus de test

Les CV sont présents dans :

```text
CANDILOG_CV_DIR
```

Commence par inventorier automatiquement ce dossier.

Détecte :

```text
*.pdf
*.PDF
```

Utilise tous les PDF valides présents.

Ne suppose pas qu’il y en a exactement 30.

---

# IMPORTANT — Utiliser les vraies données sans modification

Les CV doivent être utilisés **strictement tels quels**.

Ne modifie jamais :

- noms ;
- prénoms ;
- emails ;
- téléphones ;
- adresses ;
- entreprises ;
- dates ;
- postes ;
- expériences ;
- formations ;
- compétences ;
- liens ;
- descriptions ;
- mise en page ;
- contenu.

Aucune anonymisation des PDF locaux. Le dépôt public, lui, ne reçoit ni PDF, ni mapping, ni donnée personnelle : seulement des identifiants `CV-001`…

Aucun remplacement.

Aucune simplification artificielle.

Aucune correction manuelle avant l'import.

Le benchmark doit comparer :

```text
CV original réel
VS
résultat réel produit par Candilog
```

Modifier les données fausserait le benchmark.

---

# Utilisation des données réelles dans le benchmark

Tu peux utiliser les vraies données contenues dans les CV pour :

- construire la ground truth ;
- comparer les résultats ;
- vérifier les noms ;
- vérifier les coordonnées ;
- vérifier les entreprises ;
- vérifier les postes ;
- vérifier les dates ;
- vérifier les formations ;
- vérifier les compétences ;
- détecter les hallucinations ;
- analyser précisément les erreurs.

Il n’est pas nécessaire d’anonymiser ces données pour l’analyse locale. Ne les écris pas dans un fichier versionné.

---

# Traitement local

Les CV servent à des tests locaux.

Leur analyse avec le LLM doit rester locale.

Le modèle utilisé doit être :

```text
LiquidAI/lfm2.5-1.2b-instruct:latest
```

via Ollama.

Ne remplace jamais silencieusement ce modèle par :

- Mistral ;
- Qwen ;
- DeepSeek ;
- OpenAI ;
- Claude ;
- Gemini ;
- un autre modèle.

Ne commit pas les PDF, le mapping, les sorties brutes, ni aucune donnée personnelle dans le dépôt public.

Les CV restent sur la machine de la personne qui mesure. Un rapport versionné n’utilise que des identifiants anonymes (`CV-001`…), jamais un nom, un e-mail, un téléphone ou une adresse.

---

# Étape 1 — Comprendre l'existant

Avant toute modification, analyse complètement le pipeline existant.

Identifie précisément :

1. page d'import du profil ;
2. bouton déclenchant l'import ;
3. code frontend ;
4. commande Tauri ;
5. service Rust ;
6. extraction PDF ;
7. provider Ollama ;
8. prompt actuel ;
9. paramètres Ollama ;
10. format demandé ;
11. parser ;
12. DTO ;
13. validations ;
14. transformations ;
15. mapping vers Profile ;
16. gestion des erreurs ;
17. logs ;
18. tests existants.

Documente brièvement le pipeline avant de commencer les modifications.

Ne refactorise pas inutilement tant que tu n'as pas identifié les causes réelles des erreurs.

---

# Étape 2 — Vérifier Ollama

Vérifie qu'Ollama fonctionne.

Vérifie que le modèle est installé :

```bash
ollama list
```

Le modèle obligatoire est :

```text
LiquidAI/lfm2.5-1.2b-instruct:latest
```

Tous les benchmarks doivent réellement utiliser ce modèle.

---

# Étape 3 — Référence obligatoire

Avant toute modification :

**exécute TOUS les CV avec l'état actuel de Candilog.**

Cette passe est la référence à battre. Aujourd’hui, cet état est déjà celui décrit en tête (ordre de mise en page, contacts, formation). Ne reviens pas à la baseline à 54,9 pour « repartir propre », sauf si tu mesures d’abord que le code gardé a régressé.

Je veux obligatoirement :

```text
Nombre de CV

Score moyen
Score médian
Score minimum
Score maximum
Écart-type

Temps moyen par CV
Temps médian par CV
Temps minimum
Temps maximum
P95
Temps total du benchmark

Nombre moyen d'appels LLM
Tokens moyens d'entrée
Tokens moyens de sortie
Tokens/s moyen si disponible

Imports réussis
Imports échoués
JSON invalides
Hallucinations
```

La baseline doit notamment transformer :

```text
Score moyen ≈ 54 / 100
```

en une référence complète telle que :

```text
BASELINE

Score moyen : 54.0 / 100
Temps moyen : 24.8 s / CV
```

Toutes les versions futures devront être comparées à cette baseline.

---

# Étape 4 — Construire la ground truth

Construis pour chaque CV une référence représentant les informations réellement présentes.

La ground truth doit conserver les vraies informations.

Exemple :

```json
{
  "firstName": "valeur réelle",
  "lastName": "valeur réelle",
  "email": "valeur réelle",
  "phone": "valeur réelle",
  "experiences": [
    {
      "company": "entreprise réelle",
      "jobTitle": "poste réel",
      "startDate": "date réelle",
      "endDate": "date réelle"
    }
  ]
}
```

Ne remplace pas les informations par des valeurs fictives.

---

# Ne pas utiliser LFM2.5 comme juge unique

Le modèle testé ne doit pas simplement juger lui-même ses propres réponses.

La qualité doit être évaluée autant que possible par :

- ground truth ;
- contenu réel du PDF ;
- texte extrait ;
- comparaison déterministe ;
- règles métier ;
- validation structurée.

---

# Classification des résultats

Pour chaque valeur :

```text
CORRECT
PARTIEL
MANQUANT
INCORRECT
INVENTÉ
```

Une information inventée est une erreur grave.

---

# Champs à évaluer

## Identité

- prénom ;
- nom ;
- intitulé professionnel ;
- résumé ;
- adresse ;
- ville ;
- code postal ;
- téléphone ;
- email ;
- liens.

Objectifs :

```text
Prénom / nom corrects       >= 98 %
Email correct               >= 98 %
Téléphone correct           >= 95 %
Ville / localisation        >= 90 %
Liens                       >= 90 %
```

---

# Expériences professionnelles

Pour chaque expérience :

- poste ;
- entreprise ;
- localisation ;
- date de début ;
- date de fin ;
- poste actuel ;
- description ;
- missions ;
- technologies ;
- compétences éventuelles.

Vérifie :

- nombre d'expériences ;
- ordre ;
- associations poste / entreprise ;
- associations dates / expérience ;
- emploi actuel ;
- expériences manquantes ;
- expériences fusionnées ;
- expériences inventées.

Objectifs :

```text
Expériences détectées correctement        >= 85 %
Poste correctement associé                >= 90 %
Entreprise correctement associée          >= 90 %
Dates correctement associées              >= 85 %
Expérience actuelle correctement détectée >= 90 %
```

Les erreurs suivantes sont critiques :

```text
deux expériences fusionnées
mauvaise entreprise
mauvais poste
date rattachée au mauvais emploi
poste inventé
entreprise inventée
expérience inventée
```

---

# Formations

Vérifie :

- diplôme ;
- établissement ;
- localisation ;
- dates ;
- description.

Objectifs :

```text
Formations détectées correctement >= 85 %
Diplôme correct                   >= 90 %
Établissement correct             >= 90 %
Dates correctement associées      >= 85 %
```

---

# Compétences

Vérifie :

- compétences techniques ;
- outils ;
- technologies ;
- frameworks ;
- compétences métier.

Objectif :

```text
Compétences pertinentes détectées >= 85 %
```

Mais :

```text
compétence manquante > compétence inventée
```

---

# Langues

Vérifie :

- langue ;
- niveau lorsqu'il est réellement présent.

Objectif :

```text
Langues correctement détectées >= 90 %
```

Ne jamais inventer un niveau.

---

# Autres champs

Analyse également tous les autres champs réellement supportés par le schéma `Profile` de Candilog.

Le benchmark doit suivre le schéma réel de l'application.

---

# Score heuristique

Base recommandée :

```text
Identité                      /10
Expériences                   /30
Formations                    /20
Compétences                   /15
Langues                       /10
Autres champs                 /5
Respect du format JSON        /5
Absence d'hallucinations      /5

TOTAL                         /100
```

Adapte les pondérations au schéma réel si nécessaire.

Ne modifie cependant pas le système de scoring entre deux versions sans raison explicite, sinon les scores deviennent incomparables.

---

# Étape 5 — Vérifier l'extraction PDF

Avant d'accuser le modèle, vérifie le contenu réellement donné au LLM.

Compare :

```text
PDF original
↓
texte extrait par Candilog
↓
texte fourni au modèle
```

Cherche :

- colonnes dans le mauvais ordre ;
- caractères perdus ;
- mots concaténés ;
- sections mélangées ;
- dates séparées ;
- sidebar insérée au mauvais endroit ;
- informations perdues.

Si le texte fourni est mauvais, améliorer uniquement le prompt ne suffira pas.

Tu peux améliorer le pipeline général d'extraction PDF si cela améliore tous les CV concernés.

Interdit :

```text
if fichier == "cv_de_x.pdf":
    correction spéciale
```

Autorisé :

```text
améliorer génériquement la reconstruction des CV en deux colonnes
```

---

# Étape 6 — Analyser les erreurs

Après chaque passe, catégorise les erreurs.

Exemples :

```text
DATE_PARSING
EXPERIENCE_MERGED
EXPERIENCE_MISSING
EDUCATION_MISSING
SKILL_HALLUCINATION
IDENTITY_WRONG
PHONE_WRONG
LOCATION_WRONG
JSON_INVALID
FIELD_FORMAT_INVALID
SECTION_CONFUSION
TWO_COLUMN_READING
EMPTY_FIELD_INVENTED
CURRENT_JOB_WRONG
COMPANY_JOB_CONFUSION
DATES_ASSIGNED_TO_WRONG_JOB
DUPLICATED_EXPERIENCE
DUPLICATED_SKILL
```

Compte :

```text
nombre de CV affectés
impact sur le score
impact sur les champs critiques
```

Travaille d'abord sur les erreurs les plus fréquentes et les plus coûteuses.

---

# Étape 7 — Optimiser le vrai prompt Candilog

Le principal objectif est d'améliorer le prompt réellement utilisé dans l'application.

Pour un modèle 1,2B, privilégie :

- instructions courtes ;
- règles explicites ;
- vocabulaire simple ;
- format strict ;
- JSON minimal ;
- valeurs `null` lorsque l'information manque ;
- interdiction explicite d'inventer ;
- tâches simples ;
- exemples courts uniquement si utiles.

Évite les prompts inutilement gigantesques.

Avec un petit modèle :

```text
plus d'instructions != forcément plus de qualité
```

Teste réellement.

---

# Étape 8 — Tester plusieurs architectures

Tu as l'autorisation d'expérimenter.

## Stratégie A — Appel unique

```text
CV
↓
1 appel
↓
Profile complet
```

Mesurer :

```text
score
temps
tokens
```

---

## Stratégie B — Appels spécialisés

Exemple :

```text
Appel 1 → identité
Appel 2 → expériences
Appel 3 → formations
Appel 4 → compétences + langues
```

Puis fusion déterministe.

Ne suppose pas que cela est meilleur.

Benchmarke.

---

## Stratégie C — Deux appels

Exemple :

```text
Appel 1 → identité + expériences

Appel 2 → formations + compétences + langues
```

Cette stratégie peut éventuellement offrir un meilleur compromis vitesse / qualité.

---

## Stratégie D — Prétraitement déterministe

Extraire sans LLM ce que du code classique peut reconnaître correctement :

```text
email
téléphone
LinkedIn
GitHub
URL
```

Puis utiliser le LLM pour le contenu sémantique.

---

## Stratégie E — Correction ciblée

```text
Extraction initiale
↓
validation
↓
détection d'une section problématique
↓
nouvel appel uniquement sur cette section
↓
fusion
```

Ne relance pas automatiquement tout le CV lorsqu'une seule section pose problème.

---

# Mesurer le coût des appels LLM

Pour chaque architecture :

```text
nombre d'appels
temps total
temps de chaque appel
tokens input
tokens output
score final
```

Exemple de comparaison :

```text
STRATÉGIE A
1 appel
Score : 68
Temps : 21 s

STRATÉGIE B
2 appels
Score : 77
Temps : 29 s

STRATÉGIE C
4 appels
Score : 79
Temps : 57 s
```

Dans cet exemple, la stratégie B peut être préférable.

Le nombre d'appels n'est pas un objectif en soi.

Le critère est :

```text
meilleur rapport qualité / fiabilité / vitesse
```

---

# Étape 9 — Paramètres Ollama

Teste lorsque pertinent :

- `temperature` ;
- `top_p` ;
- `top_k` ;
- `repeat_penalty` ;
- `seed` ;
- `num_predict` ;
- fenêtre de contexte ;
- JSON mode ;
- structured output / JSON Schema si supporté.

Teste notamment :

```text
temperature = 0
```

pour l'extraction déterministe.

Utilise une seed fixe lorsqu'elle est disponible afin d'améliorer la reproductibilité des comparaisons.

---

# Étape 10 — Validation déterministe

Ne demande pas au LLM de faire ce que le code peut vérifier.

Ajoute ou améliore si utile :

- validation email ;
- validation téléphone ;
- validation URL ;
- normalisation dates ;
- trim ;
- suppression valeurs vides ;
- déduplication ;
- validation enums ;
- cohérence début / fin ;
- normalisation tableaux ;
- validation Rust / Zod ;
- détection de réponses incohérentes.

Le modèle extrait.

Le code sécurise et normalise.

---

# Hallucinations

Les hallucinations sont des erreurs critiques.

Objectif :

```text
quasi 0 hallucination
```

Et idéalement :

```text
0 hallucination sur les champs critiques
```

Règle :

```text
information manquante > information inventée
```

Une augmentation du score qui augmente sensiblement les hallucinations doit normalement être rejetée.

---

# Robustesse technique obligatoire

Objectifs :

```text
0 crash
0 panic Rust
0 import bloqué
0 JSON final invalide
0 profil impossible à parser
0 erreur de validation non gérée
```

---

# Étape 11 — Boucle d'optimisation

Fonctionne continuellement ainsi :

```text
1. Benchmark
2. Mesure score + temps
3. Analyse des erreurs
4. Hypothèse
5. Modification
6. Nouveau benchmark
7. Mesure score + temps
8. Recherche des régressions
9. Comparaison avec la meilleure version
10. Conservation ou annulation
11. Nouvelle itération
```

---

# Tests rapides et validation complète

Pendant les expérimentations, tu peux utiliser un sous-ensemble représentatif :

```text
CV simple
CV complexe
CV deux colonnes
CV avec nombreuses expériences
CV junior
CV très dense
```

Mais toute version candidate sérieuse doit obligatoirement être testée sur :

```text
TOUS les PDF du dossier `CANDILOG_CV_DIR`
```

---

# Ne pas suradapter les CV de test

Interdit :

```text
si candidat == X
si entreprise == Y
si fichier == Z
```

Le prompt final et le pipeline doivent rester généralisables à un nouveau CV totalement inconnu.

---

# Régressions

Pour chaque nouvelle version, classe les CV :

```text
AMÉLIORÉ
STABLE
DÉGRADÉ
```

Et concernant la vitesse :

```text
PLUS RAPIDE
ÉQUIVALENT
PLUS LENT
```

Pour chaque CV fortement dégradé, analyse pourquoi.

Une augmentation de la moyenne ne suffit pas à valider une version.

---

# Tableau comparatif obligatoire

Maintiens pendant toute la mission un tableau similaire :

```text
Version | Score moyen | Médiane | Score min | Temps moyen | P95 | Appels LLM | Hallucinations | Échecs
v1      | 54.0        | ...     | ...       | 24.8 s      | ... | 1          | ...            | ...
v2      | 63.2        | ...     | ...       | 25.5 s      | ... | 1          | ...            | ...
v3      | 72.7        | ...     | ...       | 29.1 s      | ... | 2          | ...            | ...
v4      | 77.4        | ...     | ...       | 31.6 s      | ... | 2          | ...            | ...
```

**Score moyen et temps moyen doivent toujours être visibles ensemble.**

---

# Progression attendue

Référence actuelle :

```text
78,0 / 100
```

La baseline historique était 54,9 / 100. Cherche maintenant :

```text
78
↓
80 si raisonnablement possible
↓
minimum 60, puis 65 si possible
```

Ne considère pas 75 comme un arrêt. 75 est déjà atteint. 80 et le minimum 60 ne le sont pas.

À chaque palier, vérifie également l'évolution du temps.

Exemple :

```text
54 → 65
Temps : 25 s → 26 s
→ excellent
```

```text
65 → 75
Temps : 26 s → 32 s
→ probablement intéressant
```

```text
75 → 76
Temps : 32 s → 85 s
→ probablement mauvais compromis
```

---

# Critère de validation finale

Une version est considérée comme bonne si elle atteint idéalement :

```text
Score moyen >= 75
Score médian >= 75
Score minimum >= 60

0 crash
0 JSON invalide
quasi 0 hallucination
aucun CV complètement raté

temps moyen compatible avec une utilisation réelle dans Candilog
```

Une version peut être considérée comme excellente pour LFM2.5 1.2B si elle atteint :

```text
Score moyen >= 80
Score médian >= 80
Score minimum >= 65

0 crash
0 JSON invalide
quasi 0 hallucination

avec un temps moyen raisonnable
```

---

# Règle concernant le temps

Ne fixe pas arbitrairement une limite de secondes avant d'avoir mesuré la baseline réelle.

Utilise d'abord les performances actuelles comme référence.

Ensuite :

```text
à qualité équivalente → la version la plus rapide gagne

à vitesse équivalente → la version la plus fiable gagne

petit gain de qualité + énorme augmentation du temps → généralement rejeter

gros gain de qualité + augmentation raisonnable du temps → peut être accepté
```

Le meilleur système n'est donc pas forcément celui qui obtient le score maximal.

Le but est de trouver le meilleur **compromis qualité / fiabilité / vitesse**.

---

# Plateau

Si plusieurs itérations sérieuses successives n'améliorent plus significativement le système :

ne modifie pas quelques mots au hasard.

Teste plutôt :

- réduction du prompt ;
- extraction par sections ;
- deux passes ;
- validation déterministe ;
- structured output ;
- amélioration extraction PDF ;
- prétraitement ;
- correction ciblée ;
- simplification du JSON.

Un plateau ne doit être déclaré qu'après plusieurs expérimentations sérieuses.

---

# Validation dans l'application réelle

Le benchmark automatisé sert à accélérer les tests.

Mais tu dois également tester plusieurs CV directement dans l'interface réelle :

```text
Profil
↓
Importer depuis un CV
↓
sélection PDF
↓
loading
↓
extraction
↓
Ollama
↓
parsing
↓
aperçu
↓
validation
↓
profil Candilog
```

Vérifie également que le temps mesuré par le benchmark correspond raisonnablement à l'expérience réelle utilisateur.

---

# Tests automatisés

Lorsque tu corriges un problème générique, ajoute des tests automatisés lorsque pertinent.

Exemples :

- parsing JSON ;
- dates ;
- déduplication ;
- expériences multiples ;
- emploi actuel ;
- champs optionnels ;
- erreurs Ollama ;
- réponse invalide ;
- mapping `Profile` ;
- validations ;
- normalisation.

Les tests permanents peuvent utiliser des données synthétiques.

Mais ils ne remplacent jamais le benchmark principal avec les vrais CV.

---

# Architecture

Respecte l'architecture existante de Candilog.

Le code doit rester :

- clair ;
- modulaire ;
- maintenable ;
- typé ;
- testé ;
- cohérent avec l'existant.

Tous les noms techniques :

```text
anglais
```

Cela concerne :

- fichiers ;
- dossiers ;
- fonctions ;
- variables ;
- structs ;
- enums ;
- DTO ;
- services ;
- repositories ;
- types.

Les commentaires éventuels peuvent rester :

```text
français
```

Ne fais aucun refactor massif sans rapport avec la mission.

---

# Git

Avant de commencer :

```bash
git status
```

Ne détruis aucun changement existant. Ne revert pas le code gardé sans un benchmark complet qui le bat.

Avant les modifications importantes :

```bash
git diff
```

Ne commit pas accidentellement les CV présents dans :

```text
CANDILOG_CV_DIR
```

Si tu réalises des commits, les messages de commit doivent être **en français**.

---

# Journal expérimental obligatoire

Maintiens un journal des expérimentations.

Exemple :

```markdown
## Iteration 01 — Baseline

Model:
LiquidAI/lfm2.5-1.2b-instruct:latest

Score moyen:
54.0

Temps moyen:
24.8 s

Temps médian:
22.9 s

P95:
39.2 s

Appels LLM:
1

Principales erreurs:
- expériences fusionnées
- formations manquantes
- dates mal associées

---

## Iteration 02

Hypothèse:
Le modèle reçoit trop d'informations simultanément.

Modification:
Prompt expériences simplifié.

Score moyen:
61.7

Temps moyen:
25.2 s

Gain score:
+7.7

Surcoût:
+0.4 s

Décision:
CONSERVER
```

---

# Rapport final obligatoire

À la fin, produis impérativement :

```text
CORPUS

Nombre de PDF :
PDF exploitables :
PDF en erreur :

==================================================
BASELINE
==================================================

Score moyen :
Score médian :
Score minimum :
Score maximum :
Écart-type :

Temps moyen par CV :
Temps médian :
Temps minimum :
Temps maximum :
P95 :
Temps total du benchmark :

Appels LLM moyens :
Tokens input moyens :
Tokens output moyens :
Tokens/s :

Imports échoués :
JSON invalides :
Hallucinations :

==================================================
VERSION FINALE
==================================================

Score moyen :
Score médian :
Score minimum :
Score maximum :
Écart-type :

Temps moyen par CV :
Temps médian :
Temps minimum :
Temps maximum :
P95 :
Temps total du benchmark :

Appels LLM moyens :
Tokens input moyens :
Tokens output moyens :
Tokens/s :

Imports échoués :
JSON invalides :
Hallucinations :

==================================================
DIFFÉRENCE
==================================================

Gain score absolu :
Gain score relatif :

Différence temps moyen :
Différence temps moyen en % :

Différence P95 :
Différence nombre d'appels :
Différence hallucinations :
Différence échecs :

==================================================
MEILLEUR COMPROMIS QUALITÉ / VITESSE
==================================================

Version retenue :
Score :
Temps moyen :
Pourquoi cette version est retenue :

==================================================
MODIFICATIONS
==================================================

Prompt :
...

Pipeline :
...

Extraction PDF :
...

Validation :
...

==================================================
LIMITES RESTANTES
==================================================

...

==================================================
FICHIERS MODIFIÉS
==================================================

...

==================================================
TESTS AJOUTÉS
==================================================

...
```

---

# Priorités finales

Respecte cet ordre :

```text
1. Ne pas inventer
2. Ne pas associer une information au mauvais poste / diplôme
3. Robustesse technique
4. Expériences
5. Formations
6. Identité
7. Compétences
8. Langues et informations secondaires
9. Score global
10. Temps d'exécution
```

Cependant, le temps d'exécution doit être **mesuré à chaque itération**.

Ne sacrifie pas énormément de vitesse pour quelques dixièmes de point.

---

# Règle finale

Tu as l'autorisation de modifier autant de fois que nécessaire :

- le prompt ;
- les paramètres Ollama ;
- le parsing ;
- les validations ;
- le prétraitement ;
- le pipeline d'import ;

si les benchmarks démontrent objectivement que cela améliore le résultat.

Ne modifie jamais artificiellement les CV pour augmenter les scores.

Utilise les fichiers originaux présents dans :

```text
CANDILOG_CV_DIR
```

Procède continuellement ainsi :

```text
observer
↓
mesurer
↓
comprendre
↓
formuler une hypothèse
↓
modifier
↓
benchmark
↓
comparer score + temps
↓
chercher les régressions
↓
conserver ou annuler
↓
recommencer
```

Ne considère pas le travail terminé après seulement quelques itérations.

Continue tant qu'il reste des améliorations raisonnables et mesurables.

La question finale est :

> **Est-ce qu'un utilisateur peut importer un vrai CV dans Candilog et obtenir rapidement un profil fiable avec `LiquidAI/lfm2.5-1.2b-instruct:latest` ?**

Et les deux KPI principaux à me communiquer pour chaque version sont obligatoirement :

```text
SCORE HEURISTIQUE MOYEN : XX / 100
TEMPS MOYEN PAR CV : XX.X secondes
```