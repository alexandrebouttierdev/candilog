# Mission — Intégrer Ollama géré automatiquement par Candilog

Tu travailles directement dans le projet **Candilog** présent dans le dossier courant.

Candilog est une application desktop basée sur :

```text
Tauri 2
React
TypeScript strict
Rust
```

L'application possède déjà plusieurs providers IA, notamment :

- Ollama ;
- llama.cpp ;
- fournisseurs IA distants.

L'objectif principal de cette mission est d'ajouter un mode :

```text
OLLAMA GÉRÉ PAR CANDILOG
```

afin que l'utilisateur puisse utiliser une IA locale **sans installer Ollama manuellement**, sans terminal et sans configuration technique.

Cette mission doit également intégrer proprement :

1. le catalogue de modèles locaux recommandés déjà défini pour Candilog ;
2. leur téléchargement et leur gestion ;
3. une recommandation compréhensible pour les utilisateurs débutants ;
4. un bouton **Tester** sur les modèles ;
5. une modal de benchmark utilisant `CV_BENCHMARK.pdf` ;
6. un score de qualité réel basé sur un JSON attendu ;
7. les performances du modèle : temps, tokens/s, tokens, etc. ;
8. une **refonte de la structure, du design et de l'UX des réglages Intelligence artificielle** ;
9. un **sélecteur rapide global Fournisseur + Modèle** disponible sur tous les écrans principaux de Candilog ;
10. la suppression, dans le rail latéral, des informations de monitoring CPU/RAM et du modèle actuellement utilisé, remplacées par ce sélecteur rapide global.


---

# MODE D’EXÉCUTION POUR CURSOR / COMPOSER 2.5 — OBLIGATOIRE

Cette section définit **comment exécuter toute la mission**. Elle ne remplace aucun critère fonctionnel ou technique décrit plus bas : elle impose l’ordre de travail, les vérifications et les garde-fous nécessaires pour éviter une implémentation partielle ou incohérente.

## Principe général

Tu dois traiter cette mission comme une **migration multi-étapes d’une application existante**, et non comme une génération de code à partir de zéro.

Avant toute modification importante :

```text
COMPRENDRE L'EXISTANT
↓
LOCALISER LES POINTS D'INTÉGRATION
↓
ÉTABLIR LE PLAN
↓
IMPLÉMENTER PAR PHASES
↓
TESTER CHAQUE PHASE
↓
AUDITER L'ENSEMBLE
```

Ne saute jamais directement à la création de nouveaux composants/services si une abstraction équivalente existe déjà dans Candilog.

## Règles de priorité en cas de conflit

Si deux exigences semblent entrer en conflit, applique cet ordre de priorité :

```text
1. Sécurité et isolation de l'Ollama géré
2. Intégrité des données utilisateur / aucune régression
3. Réutilisation de l'architecture et des services existants
4. Fonctionnement réel de bout en bout
5. Respect de l'UX et des maquettes fournies
6. Élégance / refactoring secondaire
```

Ne sacrifie jamais une règle de sécurité ou de non-régression pour reproduire plus vite l’interface.

## RÈGLE CRITIQUE — Ne pas s'arrêter après l'analyse

Commence par analyser le projet, mais **l'analyse n'est pas le livrable**.

Après l'analyse, poursuis automatiquement l'implémentation jusqu'aux tests et à l'audit final.

Ne demande pas une validation intermédiaire à l'utilisateur sauf si tu rencontres un **blocage réel impossible à résoudre à partir du dépôt**, par exemple :

- secret/API indispensable absent ;
- fichier explicitement requis réellement introuvable après recherche approfondie ;
- choix fonctionnel contradictoire sans solution compatible ;
- dépendance externe inaccessible empêchant matériellement l'implémentation.

Une préférence mineure de code, de nommage ou d'organisation n'est pas un motif pour interrompre la mission.

## RÈGLE CRITIQUE — Respect du travail existant

Avant toute modification :

```bash
git status
```

Puis inspecte l'état du dépôt et les modifications déjà présentes.

Interdictions :

```text
git reset --hard
git clean -fd
git checkout -- .
restaurer globalement des fichiers modifiés
écraser des changements utilisateur non liés à cette mission
reformatter massivement des fichiers sans nécessité
```

Si un fichier contient déjà des modifications, fusionne ton travail avec précaution.

## Phase 0 — Cartographie obligatoire du projet

Avant d'écrire du code, localise précisément au minimum :

```text
- architecture frontend React/Tauri
- architecture Rust/Tauri
- navigation principale
- navigation de la page Réglages
- page actuelle Intelligence artificielle
- rail latéral actuel
- affichage CPU/RAM/monitoring IA actuel
- sélection actuelle provider/modèle
- stores/state managers associés
- persistance des réglages
- providers IA existants
- provider Ollama existant
- support llama.cpp existant
- fournisseurs distants existants
- DTO/configuration provider
- pipeline réel d'import CV
- extraction PDF
- prompt d'import de profil
- parsing/validation structured output
- persistance du profil
- mécanisme d'annulation IA
- collecte de métriques/tokens si elle existe
- thème Système/Clair/Sombre
- réglage Son de fin de traitement
- composants UI réutilisables
- composants modal/dropdown/select existants
- tests existants liés à l'IA et au profil
- CV_BENCHMARK.pdf
- éventuelle ground truth du benchmark
```

Utilise les outils de recherche du dépôt (`rg`, recherche Cursor, arborescence, imports/références) avant d'inventer une nouvelle architecture.

### Sortie interne attendue de cette phase

Établis une carte concise :

```text
Existant réutilisé
→ ...

Existant à adapter
→ ...

Nouveaux éléments réellement nécessaires
→ ...

Risques de régression identifiés
→ ...
```

Cette carte sert à guider l'implémentation. Tu n'as pas besoin de demander confirmation.

## Phase 0.5 — Établir une baseline avant modification

Avant de modifier les zones critiques, exécute les vérifications disponibles et pertinentes du projet :

```text
- build/typecheck frontend
- cargo check ou équivalent Rust
- tests ciblés existants
- lint si disponible et raisonnablement rapide
```

Le but est de distinguer :

```text
erreurs déjà présentes avant la mission
VS
erreurs introduites par cette mission
```

Ne tente pas de corriger des problèmes totalement hors périmètre sauf s'ils empêchent directement cette mission.

## Gestion de progression pour une mission longue

Cette mission est volumineuse. Maintiens une checklist de progression courte et constamment à jour.

Si le système de TODO intégré à Cursor/Composer est disponible, utilise-le en priorité.

Sinon, tu peux créer temporairement à la racine :

```text
AI_IMPLEMENTATION_PROGRESS.md
```

avec uniquement :

```text
[ ] Phase
[ ] Sous-tâche
[ ] Test attendu
[ ] Statut
```

Mets ce fichier à jour au fur et à mesure afin de pouvoir reprendre correctement après une compression de contexte.

**Supprime ce fichier temporaire avant la fin de la mission**, sauf s'il apporte une vraie documentation utile au projet.

## Ordre d'implémentation obligatoire

Implémente dans l'ordre suivant, sauf si l'analyse du dépôt impose une petite adaptation justifiée.

### Phase 1 — Modèle de domaine et état partagé IA

Commence par stabiliser les concepts communs :

```text
provider actif
modèle actif
configuration provider
catalogue modèles
état runtime local
état installation modèle
métriques benchmark
historique benchmark
```

Objectif : éviter que les composants UI définissent eux-mêmes leur propre logique de sélection.

La sélection rapide globale et la page Réglages doivent utiliser **la même source de vérité**.

### Phase 2 — Managed Ollama côté Rust

Implémente ensuite le runtime privé :

```text
paths
manifest
plateforme/architecture
download
checksum
installation atomique
port privé
process lifecycle
health check
PID/handle fiable
shutdown
recovery
```

À ce stade, ne travaille pas encore sur la finition visuelle.

Valide l'isolation avant de continuer.

### Phase 3 — Catalogue et gestion des modèles

Branche le catalogue local centralisé sur le runtime géré :

```text
list
pull/download
progression
annulation
installed state
use
remove
compatibilité machine
```

Ne duplique pas les identifiants de modèles dans plusieurs composants.

### Phase 4 — Intégration provider commune

Réutilise le provider Ollama existant pour supporter :

```text
Managed Ollama
External Ollama
```

via une configuration commune, notamment :

```text
baseUrl
model
options
```

Puis vérifie que llama.cpp et les fournisseurs distants restent fonctionnels.

### Phase 5 — Pipeline benchmark utilisateur

Seulement après stabilisation des providers, branche :

```text
CV_BENCHMARK.pdf
↓
pipeline réel d'import CV
↓
dry-run
↓
ProfileImportDTO réel
↓
ground truth
↓
scoring déterministe
↓
métriques
```

Le benchmark doit partager le vrai pipeline d'import et **ne doit jamais avoir sa propre extraction simplifiée**.

### Phase 6 — Refonte Réglages → Intelligence artificielle

Ensuite seulement, adapte l'interface à partir des deux maquettes fournies.

RÈGLES ABSOLUES :

```text
- conserver la navigation Réglages existante dans la page ;
- ne pas transformer toute la page Réglages en une page IA isolée ;
- reproduire la hiérarchie visuelle des maquettes sans copier aveuglément des pixels ;
- conserver le design system Candilog ;
- supporter light mode et dark mode ;
- éviter les composants SaaS génériques hors design system.
```

Les maquettes sont des **références fonctionnelles et visuelles obligatoires** pour cette zone.

### Phase 7 — Nouvel onglet Réglages de la section IA

Déplace proprement dans le nouvel onglet adapté :

```text
Apparence
└── Thème

Notifications et sons
└── Son de fin de traitement
```

Réutilise impérativement la logique/persistance déjà existante.

Ne crée pas un second système de thème ou un second réglage audio.

Supprime uniquement les anciens emplacements devenus redondants après vérification de leur remplacement.

### Phase 8 — Sélecteur rapide global provider + modèle

Implémente le sélecteur rapide dans les écrans principaux.

Il remplace l'ancien affichage latéral de :

```text
monitoring CPU/RAM
activité IA
provider/modèle utilisé
```

Ne supprime pas le rail lui-même : supprime seulement les informations explicitement remplacées par cette nouvelle UX.

Le sélecteur doit toujours refléter la même source de vérité que la page Réglages IA.

### Phase 9 — Modal Tester et UX de benchmark

Branche le bouton Tester global et ceux des modèles sur le benchmark commun.

Vérifie notamment :

```text
loading
progression
time elapsed
annulation réelle
score
temps
tokens/s
détails
retester
provider distant warning
historique dernier test
```

### Phase 10 — Nettoyage, tests et audit final

Une fois la feature fonctionnelle :

```text
chercher les doublons
chercher les anciens sélecteurs provider/modèle
chercher les anciens blocs CPU/RAM à supprimer
chercher les anciennes occurrences Thème / Son de fin de traitement
chercher les TODO temporaires
chercher les mocks
chercher les valeurs hardcodées
chercher les chemins utilisateur hardcodés
chercher les commandes shell dangereuses
chercher les appels directs frontend → binaire Ollama
```

Puis exécute les tests finaux décrits plus bas.

## Gate obligatoire après chaque phase

À la fin de chaque phase :

```text
1. compiler/typecheck la zone concernée ;
2. exécuter les tests ciblés disponibles ;
3. inspecter git diff ;
4. vérifier qu'aucun fichier hors périmètre n'a été modifié inutilement ;
5. corriger les erreurs avant de passer à la phase suivante.
```

Ne cumule pas volontairement des erreurs de compilation en te disant qu'elles seront corrigées plus tard.

## Règle anti-placeholder

La mission finale ne doit contenir aucun faux fonctionnement du type :

```text
TODO important laissé en place
mock présenté comme implémentation finale
score benchmark fictif
timeout arbitraire simulant une progression
provider factice
métrique inventée
modèle marqué installé sans vérification
bouton Annuler qui ferme seulement la modal sans annuler la requête
```

Si une métrique n'est réellement pas disponible :

```text
N/A
```

comme demandé plus bas.

## Règle anti-duplication

Avant de créer :

```text
nouveau store
nouveau provider
nouveau modal system
nouveau theme manager
nouveau persistence service
nouveau HTTP client
nouveau process manager générique
```

cherche si une abstraction existe déjà.

Privilégie :

```text
adapter
extraire
étendre
factoriser
```

plutôt que dupliquer.

## Règle de diff minimal

Évite :

```text
refactorings esthétiques sans rapport
renommages massifs hors périmètre
formatage global du dépôt
réécriture complète d'une feature fonctionnelle sans nécessité
ajout d'une nouvelle librairie pour une fonction déjà couverte
```

Toute dépendance ajoutée doit être justifiée par un besoin réel de cette mission.

## Vérification des données externes critiques

Pour les éléments qui doivent être exacts et ne peuvent pas être inventés, notamment :

```text
URL de distribution officielle Ollama
version supportée
SHA-256
format archive par OS/architecture
nom de l'exécutable
identifiants/tags réels des modèles Ollama
```

utilise une source officielle ou une source déjà validée dans le projet.

Ne fabrique aucune valeur pour faire compiler.

Si l'accès réseau n'est pas disponible pendant le développement, structure le code correctement mais indique clairement dans le rapport final ce qui n'a pas pu être vérifié. Ne présente jamais une valeur supposée comme vérifiée.

## Vérification spécifique des maquettes

Les fichiers visuels fournis avec ce prompt sont :

```text
modeles_locaux.png
autres_fourniseurs.png
```

Analyse-les avant de modifier l'UI.

Après implémentation, compare explicitement :

```text
structure
hiérarchie
placement du sélecteur rapide
onglets
cartes providers
cartes modèles
configuration provider dépliée
états des boutons
badges
espacements généraux
cohérence light/dark
```

Ne modifie jamais les images de référence.

## Vérification spécifique de la navigation Réglages

La navigation générale des Réglages **doit rester présente dans la page**.

La structure conceptuelle attendue est :

```text
Réglages
├── navigation existante conservée
│
└── Intelligence artificielle
    ├── Modèles locaux
    ├── Autres fournisseurs/modèles
    └── Réglages
        ├── Apparence
        │   └── Thème
        └── Notifications et sons
            └── Son de fin de traitement
```

Ne remplace pas la navigation des Réglages par les trois onglets IA. Les trois onglets appartiennent **à la section Intelligence artificielle**, à l'intérieur de la navigation Réglages existante.

## Tests : ne jamais confondre « compilé » et « validé »

Une build réussie ne suffit pas.

Pour chaque feature critique, vérifie le comportement attendu avec le niveau de test réellement disponible :

```text
unit test
integration test
manual app test
ou combinaison des trois
```

Lorsque tu ne peux pas effectuer un test matériellement, marque-le explicitement :

```text
NON TESTÉ — raison
```

Ne marque jamais un scénario comme validé uniquement parce que le code semble correct.

## Matrice d'acceptation finale obligatoire

Avant de conclure, construis une matrice interne couvrant **tous les critères d'acceptation** de ce prompt.

Chaque critère doit avoir un état :

```text
PASS
FAIL
NON TESTÉ
NON APPLICABLE
```

Pour les critères importants, associe une preuve concise :

```text
fichier
fonction
commande de test
résultat de test
```

Tout `FAIL` doit être corrigé avant de déclarer la mission terminée, sauf blocage externe explicitement documenté.

Les `NON TESTÉ` doivent apparaître dans le rapport final avec la raison exacte.

## Definition of Done Composer

Ne considère la mission terminée que lorsque :

```text
✓ architecture existante comprise et réutilisée
✓ build frontend concernée valide
✓ build/check Rust concerné valide
✓ tests ciblés passent ou échecs préexistants documentés
✓ Managed Ollama réellement isolé
✓ aucun processus Ollama utilisateur touché
✓ catalogue modèles centralisé
✓ provider partagé Managed/External Ollama
✓ benchmark branché sur le vrai pipeline
✓ benchmark sans persistance
✓ annulation réelle lorsque supportée
✓ métriques non inventées
✓ navigation Réglages conservée
✓ Thème déplacé dans le nouvel onglet adapté
✓ Son de fin de traitement déplacé dans le nouvel onglet adapté
✓ sélecteur rapide global synchronisé
✓ ancien monitoring latéral remplacé comme demandé
✓ maquettes respectées
✓ light/dark vérifiés
✓ pas de placeholders critiques
✓ pas de chemins utilisateur hardcodés
✓ pas de secrets dans le code ou les logs
✓ git diff final relu
✓ rapport final complet
```

---

# Objectif UX global

L'expérience recherchée est :

```text
Installer Candilog
↓
Ouvrir la page IA
↓
Activer l'IA locale
↓
Candilog installe automatiquement son moteur local
↓
Choisir un modèle
↓
Candilog télécharge le modèle
↓
Tester le modèle
↓
Voir immédiatement qualité + vitesse
↓
Utiliser Candilog
```

L'utilisateur ne doit jamais avoir besoin de faire :

```bash
ollama serve
ollama pull ...
```

Il ne doit pas non plus avoir besoin :

- d'aller sur le site d'Ollama ;
- d'installer Ollama lui-même ;
- d'ouvrir un terminal ;
- de comprendre les ports ;
- de comprendre `OLLAMA_HOST` ;
- de comprendre `OLLAMA_MODELS`.

Pour l'utilisateur débutant, cela doit simplement apparaître comme :

> **IA locale Candilog**

---

# 1 — Runtime Ollama privé

Candilog doit disposer de **sa propre installation privée d'Ollama**.

Architecture cible :

```text
Ordinateur
│
├── Ollama utilisateur éventuel
│   ├── installation indépendante
│   ├── port 11434 éventuel
│   └── modèles personnels
│
└── Candilog
    └── IA locale
        ├── runtime Ollama privé
        ├── dossier modèles privé
        ├── port privé
        └── processus géré par Candilog
```

---

# RÈGLE CRITIQUE — Ne jamais toucher à Ollama utilisateur

Si Ollama est déjà installé :

**NE RIEN MODIFIER.**

Candilog ne doit jamais :

```text
désinstaller Ollama utilisateur
mettre à jour Ollama utilisateur
arrêter Ollama utilisateur
tuer son processus
modifier ~/.ollama
modifier ses modèles
modifier son port
modifier ses variables d'environnement
modifier son PATH
écraser son exécutable
```

Interdiction absolue des commandes globales :

```bash
pkill ollama
killall ollama
```

Candilog ne peut arrêter que **le processus qu'il a lui-même lancé**.

---

# 2 — Isolation complète

Utilise un dossier applicatif obtenu via les API Tauri / système.

Ne hardcode jamais un chemin utilisateur.

Structure conceptuelle :

```text
<Candilog App Data>/
└── ai/
    └── ollama/
        ├── runtime/
        │   └── <version>/
        ├── models/
        ├── downloads/
        ├── state/
        └── logs/
```

Configure le processus Ollama Candilog avec son propre :

```text
OLLAMA_MODELS=<Candilog App Data>/ai/ollama/models
```

Ne jamais utiliser le répertoire de modèles personnel de l'utilisateur.

---

# 3 — Port privé

Ollama utilisateur peut rester sur :

```text
127.0.0.1:11434
```

Candilog doit préférer :

```text
127.0.0.1:11435
```

Algorithme obligatoire :

```text
tester 11435
↓
libre → utiliser 11435

occupé
↓
chercher automatiquement le prochain port local disponible
↓
ne tuer aucun processus
```

Le runtime doit uniquement écouter sur :

```text
127.0.0.1
```

Jamais :

```text
0.0.0.0
```

---

# 4 — Processus géré par Rust

Toute la gestion système doit rester côté Rust.

Le frontend React ne lance jamais directement l'exécutable.

Créer ou adapter un gestionnaire conceptuellement équivalent à :

```text
ManagedOllamaRuntime
├── RuntimeInstaller
├── RuntimeDownloader
├── RuntimeVerifier
├── RuntimeProcessManager
├── RuntimeHealthChecker
├── RuntimeVersionManager
└── ModelManager
```

Ne crée pas artificiellement toutes ces classes si l'architecture existante permet quelque chose de plus simple.

Le backend Rust doit gérer :

- téléchargement ;
- progression ;
- checksum ;
- extraction ;
- permissions ;
- installation atomique ;
- démarrage ;
- arrêt ;
- PID / handle ;
- health check ;
- port ;
- modèles ;
- erreurs ;
- annulation ;
- mise à jour.

---

# 5 — Téléchargement du runtime

Le runtime Ollama n'a pas besoin d'alourdir l'installeur Candilog.

Il doit être téléchargé au premier usage.

Workflow :

```text
Activer IA locale
↓
détection OS
↓
détection architecture
↓
sélection du runtime officiel
↓
téléchargement HTTPS
↓
progression
↓
SHA-256
↓
extraction temporaire
↓
validation
↓
installation atomique
↓
démarrage
↓
health check
↓
READY
```

Utilise uniquement des distributions officielles Ollama.

Ne jamais exécuter automatiquement :

```bash
curl ... | sh
```

ou un installeur système.

Nous voulons une copie privée d'Ollama appartenant à Candilog.

---

# 6 — Version Ollama contrôlée

Ne télécharge pas aveuglément `latest`.

Candilog doit utiliser une version d'Ollama connue et testée.

Centralise un manifest contenant conceptuellement :

```text
version
OS
architecture
URL officielle
SHA-256
format archive
exécutable
```

Les URL et checksums ne doivent pas être dispersés dans le code.

---

# 7 — Cycle de vie

Privilégie un démarrage lazy :

```text
première utilisation IA locale
↓
runtime arrêté
↓
démarrage automatique
↓
health check
↓
requête
```

L'utilisateur ne doit jamais devoir cliquer sur :

> Démarrer Ollama

À la fermeture de Candilog :

```text
arrêter uniquement Ollama Candilog
```

Ne jamais toucher à Ollama utilisateur.

Prévoir également :

- crash Candilog ;
- processus orphelin ;
- PID recyclé ;
- port déjà utilisé ;
- double démarrage ;
- double clic ;
- fermeture pendant installation.

---

# 8 — États du runtime

Utiliser un état typé, par exemple :

```text
NOT_INSTALLED
DOWNLOADING
INSTALLING
STARTING
READY
STOPPING
STOPPED
UPDATING
ERROR
```

Éviter plusieurs booléens contradictoires.

---

# 9 — Catalogue officiel de modèles Candilog

La page IA locale doit proposer en priorité les modèles suivants.

## Modèle 1 — Très léger

```text
LiquidAI/LFM2.5-1.2B-Instruct
```

Nom affiché :

```text
LFM2.5 1.2B
```

Positionnement utilisateur :

> **Très léger**

Description courte :

> Rapide et peu gourmand en mémoire. Recommandé pour les ordinateurs modestes.

---

## Modèle 2 — Léger

```text
Ministral 3 3B
```

Positionnement :

> **Léger**

Description :

> Plus précis tout en restant adapté aux machines disposant de peu de mémoire.

---

## Modèle 3 — Équilibré

```text
Ministral 3 8B
```

Positionnement :

> **Équilibré**

Description :

> Bon compromis entre qualité, vitesse et consommation mémoire.

---

## Modèle 4 — Puissant

```text
Ministral 3 14B
```

Positionnement :

> **Puissant**

Description :

> Meilleure qualité d'analyse pour les machines disposant de davantage de mémoire.

---

## Modèle 5 — Qualité maximale

```text
Mistral Small 3.2 24B
```

Positionnement :

> **Qualité maximale**

Description :

> Modèle local plus exigeant, destiné aux machines puissantes et aux utilisateurs privilégiant la qualité.

---

# Important concernant les IDs des modèles

Ne disperse pas leurs noms ou tags Ollama dans le frontend.

Crée un catalogue central typé.

Conceptuellement :

```ts
{
  id,
  displayName,
  providerModelId,
  category,
  description,
  parameterCount,
  approximateDownloadSize,
  recommendedMemory,
  capabilities
}
```

Les vrais identifiants/tags utilisés par Ollama doivent être vérifiés avec le catalogue réellement supporté lors de l'implémentation.

Ne fabrique pas arbitrairement un tag si le nom officiel diffère.

---

# 10 — Recommandation suivant la machine

Candilog doit aider l'utilisateur à choisir.

Analyse les informations système accessibles :

```text
RAM
architecture
GPU lorsque pertinent
mémoire disponible lorsque possible
```

Et indique par modèle :

```text
Recommandé
Compatible
Peut être lent
Mémoire insuffisante
```

Ne bloque pas automatiquement un utilisateur expérimenté si le modèle peut techniquement fonctionner.

Afficher plutôt un avertissement lorsque pertinent.

Exemple :

```text
Ministral 3 8B
Équilibré

✓ Recommandé pour votre ordinateur

[ Télécharger ]
```

ou :

```text
Mistral Small 3.2 24B
Qualité maximale

⚠ Ce modèle peut être lent avec votre configuration.

[ Télécharger quand même ]
```

---

# 11 — Gestion des modèles

Workflow :

```text
choisir modèle
↓
runtime installé ?

NON
→ installer automatiquement le runtime

OUI
↓
runtime prêt
↓
modèle installé ?

NON
→ télécharger

OUI
→ prêt
```

Lorsque possible, utiliser l'API Ollama locale pour :

- pull ;
- progression ;
- liste ;
- détails ;
- suppression.

Éviter d'exécuter la CLI et parser du texte si l'API officielle permet de faire proprement la même chose.

---

# 12 — Progression modèle

Afficher :

```text
nom
taille totale
octets téléchargés
pourcentage
étape
```

Exemple :

```text
LFM2.5 1.2B

Téléchargement du modèle

██████████████░░ 81 %

824 Mo / 1,02 Go

[ Annuler ]
```

L'annulation ne doit jamais laisser l'application dans un état incohérent.

---

# 13 — Cartes modèles

Chaque modèle installé doit proposer clairement :

```text
[ Utiliser ]
[ Tester ]
[ Supprimer ]
```

Le bouton **Tester** est une fonctionnalité importante.

Il doit permettre à un utilisateur de connaître la qualité réelle du modèle **sur une tâche Candilog représentative**, et pas simplement mesurer des tokens/s avec un prompt artificiel.

---

# 14 — Benchmark utilisateur intégré

Un fichier de référence existe :

```text
CV_BENCHMARK.pdf
```

Ce CV est spécifiquement conçu pour tester l'extraction d'un profil Candilog.

Commence par localiser ce fichier dans le projet.

Ne crée pas un autre CV si celui-ci existe déjà.

Il doit devenir la base du bouton :

```text
Tester
```

---

# 15 — Ground truth de CV_BENCHMARK.pdf

Le benchmark doit disposer d'un JSON de référence contenant **le résultat exact attendu** pour :

```text
CV_BENCHMARK.pdf
```

Localise d'abord une éventuelle ground truth déjà présente.

Par exemple, conceptuellement :

```text
CV_BENCHMARK.expected.json
```

Si elle existe :

**réutilise-la.**

Ne la recrée pas arbitrairement.

Le JSON doit correspondre au schéma réellement utilisé par l'import de profil Candilog.

Exemple conceptuel uniquement :

```json
{
  "firstName": "...",
  "lastName": "...",
  "email": "...",
  "phone": "...",
  "experiences": [],
  "educations": [],
  "skills": [],
  "languages": []
}
```

Ne copie pas cet exemple comme schéma si Candilog utilise un autre DTO.

Utilise le vrai DTO d'import de profil.

---

# 16 — Le benchmark doit utiliser le vrai pipeline Candilog

C'est une règle essentielle.

Le bouton Tester ne doit PAS faire :

```text
CV_BENCHMARK.pdf
↓
petit script spécial
↓
LLM
↓
score
```

Il doit faire :

```text
CV_BENCHMARK.pdf
↓
EXACTEMENT le pipeline réel d'import de profil
↓
extraction PDF réelle
↓
prétraitement réel
↓
prompt réel
↓
provider réel
↓
modèle sélectionné
↓
parsing réel
↓
validation réelle
↓
ProfileImportDTO
↓
comparaison avec ground truth
```

Le benchmark doit donc mesurer ce qu'un utilisateur obtiendrait réellement en important un CV.

---

# 17 — Mode dry-run obligatoire

Le benchmark ne doit modifier aucune donnée utilisateur.

Il doit utiliser le pipeline réel en mode :

```text
DRY RUN
```

ou une abstraction équivalente.

Interdiction pendant un test :

```text
modifier le profil
écraser le profil
ajouter des expériences en base
ajouter des formations
modifier les compétences utilisateur
créer une candidature
modifier SQLite
```

Le résultat doit être analysé puis jeté après le benchmark.

Architecture souhaitée :

```text
importProfileFromResume()
```

ou service commun similaire :

```text
vrai import
→ pipeline
→ persistance uniquement après validation utilisateur

benchmark
→ même pipeline
→ aucune persistance
→ scoring
```

Ne duplique pas la logique d'extraction.

---

# 18 — Providers supportés par le bouton Tester

Le benchmark `CV_BENCHMARK.pdf` ne doit pas être limité à Ollama géré.

Il doit pouvoir tester, lorsque le provider supporte l'import CV :

```text
Ollama géré par Candilog
Ollama externe
llama.cpp
autres providers IA déjà supportés par Candilog
```

Le test doit utiliser :

```text
provider actuellement sélectionné
+
modèle actuellement sélectionné
```

Le même benchmark permettra donc de comparer objectivement les modèles et providers.

---

# 19 — Ne pas confondre avec le benchmark interne multi-CV

Candilog possède ou utilise également un benchmark de développement plus poussé basé sur plusieurs CV.

Les deux systèmes ont des objectifs différents.

## Benchmark utilisateur

```text
CV_BENCHMARK.pdf
```

Objectif :

```text
test rapide
compréhensible
lancé depuis l'interface
un seul CV
résultat immédiatement visible
```

## Benchmark interne développement

Exemple actuel :

```text
/home/alex/Documents/CV_TESTS
```

Objectif :

```text
plusieurs dizaines de vrais CV
analyse approfondie
régressions
optimisation prompts
comparaison des modèles
```

**Ne mélange pas les deux systèmes.**

Le test `CV_BENCHMARK.pdf` ne remplace jamais le benchmark multi-CV.

Il est destiné aux utilisateurs de Candilog et aux tests rapides.

---

# 20 — Score de benchmark

Après l'import de `CV_BENCHMARK.pdf`, compare le résultat au JSON attendu.

Le scoring doit être **déterministe autant que possible**.

Ne demande pas au même modèle de noter sa propre réponse.

Évaluer au minimum :

```text
Identité
Coordonnées
Expériences
Formations
Compétences
Langues
Autres champs Profile
Structure
Absence d'hallucinations
```

Produire un score global :

```text
XX / 100
```

---

# 21 — Score clair pour l'utilisateur

La modal doit mettre le score fortement en avant.

Exemple :

```text
Test terminé

86 / 100
Très bon
```

Créer des catégories cohérentes, par exemple :

```text
0–49    Faible
50–64   Moyen
65–74   Correct
75–84   Bon
85–94   Très bon
95–100  Excellent
```

Centralise cette classification.

Ne disperse pas les seuils dans plusieurs composants.

---

# 22 — Pénalités importantes

Une hallucination doit être davantage pénalisée qu'un champ simplement absent.

Principe :

```text
valeur correcte
> valeur partielle
> valeur manquante
> valeur incorrecte
> valeur inventée
```

En particulier, pénalise fortement :

```text
expérience inventée
entreprise inventée
diplôme inventé
date inventée
mauvais email
mauvais téléphone
mauvaise association poste / entreprise
mauvaise association dates / expérience
```

---

# 23 — Mesures de performances

Le bouton Tester doit mesurer obligatoirement :

```text
temps total
temps extraction PDF
temps prétraitement
temps LLM
temps parsing / validation
nombre d'appels LLM
tokens input
tokens output
tokens/s
```

Lorsque certaines métriques ne sont pas disponibles avec un provider :

```text
N/A
```

Ne jamais inventer une valeur.

---

# 24 — Temps moyen / répétitions

Pour éviter qu'un seul run donne une mesure aberrante, prévois une architecture permettant plusieurs répétitions.

Cependant, pour ne pas rendre le bouton trop long par défaut :

```text
Test rapide utilisateur
→ 1 exécution
```

Et éventuellement dans les options avancées :

```text
Benchmark précis
→ 3 exécutions
```

Dans ce cas afficher :

```text
score moyen
temps moyen
temps minimum
temps maximum
tokens/s moyen
```

Ne force pas trois exécutions à chaque simple clic sur Tester si cela rend l'expérience trop lente.

---

# 25 — Modal de résultat

Lorsque l'utilisateur clique sur :

```text
Tester
```

ouvrir une modal.

## Pendant le test

Exemple conceptuel :

```text
Tester LFM2.5 1.2B

Analyse du CV de référence...

Extraction du PDF         ✓
Analyse avec le modèle    ⟳
Validation                En attente
Calcul du score           En attente

Temps écoulé : 18,4 s

[ Arrêter ]
```

Le bouton Arrêter doit réellement annuler la génération lorsque le provider le permet.

---

# 26 — Modal après résultat

Exemple conceptuel :

```text
LFM2.5 1.2B
Test terminé

          78 / 100
             Bon

Qualité d'extraction
████████████████░░░░

Performances

Temps total       44,3 s
Temps IA          39,7 s
Vitesse           22,6 tokens/s
Tokens entrée     2 841
Tokens sortie     1 126

Détails

Identité          10 / 10
Expériences       24 / 30
Formations        17 / 20
Compétences       12 / 15
Langues            9 / 10
Autres             ...
Hallucinations     0

[ Fermer ]          [ Retester ]
```

Adapte les catégories au vrai scoring Candilog.

---

# 27 — Mettre en avant qualité + vitesse

Les deux informations principales sont :

```text
SCORE
TEMPS
```

Ne montre pas uniquement :

```text
78 / 100
```

Je veux immédiatement pouvoir voir :

```text
78 / 100
44,3 s
```

Le but est de comparer le compromis :

```text
qualité / vitesse
```

---

# 28 — Historique local des tests

Si cohérent avec l'architecture actuelle, conserve localement le dernier résultat de benchmark pour chaque combinaison :

```text
provider
model
runtime
model version / identifier
benchmark version
```

Exemple :

```text
LFM2.5 1.2B

Dernier test
78 / 100 · 44,3 s
```

Cela permet d'afficher directement le résultat sur la carte du modèle.

---

# 29 — Invalidation du benchmark

Un score précédent ne doit pas être présenté comme encore comparable si une partie importante du benchmark change.

Prévoir un :

```text
benchmarkVersion
```

à incrémenter lorsque changent :

```text
CV_BENCHMARK.pdf
ground truth
algorithme de scoring
pipeline benchmark incompatible
```

Un ancien résultat doit alors être marqué obsolète ou supprimé.

---

# 30 — Comparaison des modèles

La page IA doit pouvoir afficher simplement :

| Modèle | Niveau | Score test | Temps |
|---|---|---:|---:|
| LFM2.5 1.2B | Très léger | 78/100 | 44,3 s |
| Ministral 3 3B | Léger | Non testé | — |
| Ministral 3 8B | Équilibré | Non testé | — |
| Ministral 3 14B | Puissant | Non testé | — |
| Mistral Small 3.2 24B | Qualité maximale | Non testé | — |

Ne hardcode évidemment pas les scores.

Ils doivent venir des benchmarks réellement exécutés sur la machine de l'utilisateur.

---

# 31 — Le score dépend de la machine et du provider

La qualité devrait principalement dépendre du modèle et du pipeline, mais les performances dépendent fortement de la machine.

Le résultat doit donc être présenté comme :

> **Résultat sur cet ordinateur**

Ne présente jamais `44,3 s` comme une performance universelle du modèle.

---

# 32 — Test avant téléchargement

Le bouton Tester doit être disponible uniquement lorsque :

```text
runtime prêt
+
modèle installé
+
provider configuré
```

Sinon proposer l'action appropriée :

```text
[ Télécharger ]
```

ou :

```text
[ Configurer ]
```

---

# 33 — Test après installation

Après le téléchargement d'un modèle, proposer clairement :

```text
✓ Modèle installé

[ Utiliser ]
[ Tester les performances ]
```

Le test n'est pas obligatoire pour utiliser le modèle.

---

# 34 — Benchmark et appels distants

Pour un provider distant déjà configuré dans Candilog, le benchmark peut fonctionner si cela correspond au comportement actuel de l'application.

Mais afficher clairement que :

```text
le CV de benchmark sera envoyé au provider configuré
```

Pour les runtimes locaux :

```text
Ollama géré
Ollama externe
llama.cpp
```

le test reste local.

`CV_BENCHMARK.pdf` étant un CV de benchmark prévu à cet effet, cela ne doit pas être confondu avec les vrais CV utilisateurs.

---

# 35 — Intégration au provider Ollama existant

Ne duplique pas le provider Ollama.

Le provider doit pouvoir recevoir :

```text
baseUrl
model
options
```

Pour Ollama géré :

```text
baseUrl=http://127.0.0.1:<managed-port>
```

Pour Ollama externe :

```text
baseUrl=<configuration utilisateur>
```

Le pipeline commun doit conserver :

```text
prompt
structured output
streaming
tokens
tokens/s
annulation
erreurs
```

---

# 36 — Non-régression du pipeline CV

Cette mission ne doit pas dégrader l'optimisation déjà effectuée sur l'import.

Référence actuelle obtenue avec :

```text
LiquidAI/lfm2.5-1.2b-instruct:latest
```

sur le benchmark multi-CV :

```text
Score heuristique moyen : 78,0 / 100
Temps moyen : 44,3 secondes / CV
```

Ne modifie pas silencieusement :

```text
prompt
pipeline
validation
structured output
scoring interne
```

uniquement pour intégrer Ollama géré.

Si une modification commune est nécessaire :

```text
relancer les tests de non-régression
```

---

# 37 — Important : benchmark utilisateur ≠ score interne multi-CV

Ne compare pas directement :

```text
78/100 benchmark multi-CV
```

avec :

```text
XX/100 CV_BENCHMARK.pdf
```

comme s'il s'agissait nécessairement du même indicateur.

Ce sont deux benchmarks différents.

Le score utilisateur doit rester stable et reproductible avec son propre :

```text
benchmarkVersion
```

---

# 38 — Refonte UX des réglages Intelligence artificielle

Cette mission inclut une **refonte structurante de la page Réglages → Intelligence artificielle**.

Ne traite pas cette partie comme un simple ajout fonctionnel au-dessus de l'UI existante.

L'objectif est de rendre la configuration IA plus claire, plus cohérente et plus rapide à utiliser, en séparant clairement :

```text
1. l'IA locale gérée par Candilog ;
2. les autres fournisseurs / runtimes ;
3. le choix courant Fournisseur + Modèle ;
4. les actions de benchmark ;
5. la configuration détaillée des providers.
```

Respecte strictement le design system existant de Candilog.

L'interface doit rester **desktop native**, cohérente avec :

- light mode ;
- dark mode ;
- composants existants ;
- loaders ;
- modales ;
- boutons ;
- typographie ;
- rayons ;
- bordures ;
- ombres ;
- espacements ;
- états hover / active / disabled / loading ;
- densité visuelle actuelle de Candilog.

Ne crée pas une UI générique de dashboard SaaS.

---

## 38.1 — Maquettes visuelles obligatoires

Deux images de référence sont fournies avec ce prompt :

```text
modeles_locaux.png
autres_fourniseurs.png
```

Commence par **ouvrir et analyser réellement ces deux images avant de modifier l'interface**.

Elles constituent la référence visuelle principale pour :

```text
structure de page
hiérarchie visuelle
positionnement du sélecteur rapide
organisation des onglets
cartes modèles
cartes fournisseurs
panneau de configuration déplié
espacements
proportions
états actifs
badges
boutons
```

Ne les copie pas pixel par pixel si une contrainte réelle du projet l'empêche, mais conserve leur intention UX et leur structure générale.

Les données visibles dans les maquettes sont des exemples d'affichage :

```text
scores
temps
tailles
états de connexion
modèles disponibles
```

Elles ne doivent jamais être hardcodées à partir des images.

---

## 38.2 — Structure cible de la page Intelligence artificielle

### Règle critique — conserver la navigation Réglages existante

La refonte des écrans IA **ne doit surtout pas supprimer, remplacer ni masquer la navigation interne actuelle de la page Réglages**.

Avant toute modification, localise le composant / layout qui fournit actuellement la navigation des réglages et réutilise-le.

Les maquettes fournies servent de référence pour le **contenu de la section Intelligence artificielle**, mais elles ne doivent pas être interprétées comme une demande de supprimer la navigation Réglages déjà présente dans Candilog.

La navigation Réglages doit rester :

```text
visible
accessible
cohérente avec les autres pages de réglages
au même emplacement structurel qu'actuellement
compatible light / dark mode
```

Ne recrée pas une deuxième navigation concurrente si le projet possède déjà un composant commun pour les réglages.

La page doit conserver une hiérarchie claire proche des maquettes :

```text
NAVIGATION RÉGLAGES EXISTANTE — À CONSERVER

RÉGLAGES
Intelligence artificielle
Description courte

                          [ Sélecteur Fournisseur + Modèle ] [ Tester ] [ Réglages ]

Carte de synthèse du provider / modèle actif

[ Modèles locaux ] [ Autres fournisseurs/modèles ] [ Réglages ]

Contenu de l'onglet sélectionné
```

Le haut de page doit donc regrouper les actions globales IA au même endroit, **sans sacrifier la navigation générale des réglages**.

Évite de disperser dans plusieurs zones :

```text
provider actif
modèle actif
bouton Tester
configuration IA
```

---

## 38.3 — Carte de synthèse IA active

Sous le titre principal, afficher une carte de synthèse correspondant au fournisseur IA actuellement actif.

Pour `IA locale Candilog`, la maquette montre conceptuellement :

```text
IA locale Candilog      ✓ Prêt
Modèles gérés et optimisés pour votre ordinateur.

Modèle actif       Dernier test            Espace modèles
LFM2.5 1.2B        78 / 100                2,1 Go
Très léger         44,3 s sur cet ordinateur
```

Les valeurs doivent venir de l'état réel de l'application.

La carte doit notamment pouvoir refléter :

```text
provider actif
modèle actif
état provider/runtime
catégorie du modèle
résultat du dernier benchmark
temps du dernier benchmark
stockage local utilisé lorsque pertinent
```

Pour un provider distant, adapte intelligemment la carte : ne montre pas de métrique locale non pertinente simplement pour remplir l'espace.

---

# 39 — Onglets et organisation des providers

La page Intelligence artificielle doit être structurée autour de **trois onglets principaux** :

```text
Modèles locaux
Autres fournisseurs/modèles
Réglages
```

Le changement d'onglet ne doit pas modifier le provider actif par lui-même.

L'onglet `Réglages` sert à regrouper les préférences transversales actuellement mélangées au contenu IA. Il ne remplace pas la navigation générale de la section Réglages : **les deux niveaux doivent coexister proprement**.

---

## 39.1 — Onglet « Modèles locaux »

Cet onglet reprend la structure générale de `modeles_locaux.png`.

Afficher un bloc d'information simple du type :

> **Modèles optimisés pour votre ordinateur**  
> Nous recommandons automatiquement les modèles compatibles avec votre configuration.

Puis afficher les modèles Candilog sous forme de cartes comparables visuellement.

Chaque carte doit pouvoir afficher :

```text
logo / identité du fournisseur ou du moteur
catégorie utilisateur
nom du modèle
description courte
taille approximative
RAM recommandée
compatibilité machine
recommandation éventuelle
état téléchargement / installation
dernier benchmark si disponible
```

Actions selon l'état :

```text
non installé
→ [ Télécharger ]

installé mais non actif
→ [ Utiliser ] [ Tester ] [ ... ]

installé et actif
→ état sélectionné visible
→ [ Utiliser ] si nécessaire ou état actif non ambigu
→ [ Tester ] [ Supprimer ]
```

Le bouton `Tester` doit utiliser le benchmark utilisateur défini dans ce prompt.

Le menu `...` peut regrouper les actions secondaires lorsque cela améliore la lisibilité, mais les actions principales doivent rester évidentes.

Les cinq niveaux doivent être immédiatement compréhensibles :

```text
Très léger
Léger
Équilibré
Puissant
Qualité maximale
```

---

## 39.2 — Onglet « Autres fournisseurs/modèles »

Cet onglet reprend la structure générale de `autres_fourniseurs.png`.

Afficher les providers déjà réellement supportés par Candilog sous forme de cartes homogènes.

Exemples visibles dans la maquette :

```text
Claude
OpenAI
Gemini
Mistral
DeepSeek
API personnalisée
```

N'ajoute pas artificiellement un provider uniquement parce qu'il apparaît dans la maquette si Candilog ne le supporte pas réellement.

Inversement, si Candilog possède déjà un provider pertinent absent de la maquette, conserve-le et adapte-le au même système visuel.

Chaque carte provider doit afficher au minimum :

```text
logo officiel / identité visuelle du provider
nom
sous-label fournisseur
état : Configuré / Connecté / Non configuré / Erreur
bouton Utiliser lorsque disponible
bouton Configurer
```

Utilise les vrais logos déjà présents dans le projet ou des assets officiels compatibles avec les règles du projet.

Ne remplace pas tous les providers par une icône générique identique.

---

## 39.2 bis — Nouvel onglet « Réglages »

Créer un troisième onglet :

```text
Réglages
```

Cet onglet doit accueillir les préférences générales qui ne concernent ni le catalogue des modèles locaux ni la configuration d'un fournisseur.

Déplacer dans cet onglet les réglages existants suivants :

```text
Apparence
└── Thème

Son de fin de traitement
```

### Important

Il s'agit d'un **déplacement UX**, pas d'une réécriture fonctionnelle inutile.

Commence par localiser les composants, hooks, stores et mécanismes de persistance déjà utilisés pour :

```text
le thème
le mode système / clair / sombre lorsqu'ils existent
le son de fin de traitement
son activation / désactivation
ses éventuelles options existantes
```

Réutilise la logique actuelle et déplace uniquement leur présentation lorsque c'est possible.

Ne crée pas une seconde source de vérité pour ces préférences.

Ne laisse pas les anciens contrôles en double dans leur emplacement précédent une fois le nouvel onglet fonctionnel.

### Structure UX attendue

L'onglet doit rester cohérent avec le design system Candilog et utiliser des sections de réglages desktop simples, par exemple :

```text
Réglages

Apparence
Personnalisez l'affichage de Candilog.

Thème
[ Système ] [ Clair ] [ Sombre ]

────────────────────────────────

Notifications et sons
Choisissez les retours de l'application à la fin d'un traitement.

Son de fin de traitement
[ activé / désactivé ]
```

Cet exemple est conceptuel : **reprends les contrôles réellement supportés par Candilog** et ne fabrique pas d'options qui n'existent pas.

Le design doit :

```text
rester compact et desktop
reprendre les composants existants
conserver la persistance actuelle
appliquer le thème immédiatement si c'est le comportement actuel
respecter le thème système si cette option existe
ne pas déclencher de son lors d'un simple changement de réglage sauf si un aperçu existe déjà
fonctionner en light et dark mode
```

### Navigation

L'utilisateur doit pouvoir passer simplement entre :

```text
Modèles locaux
Autres fournisseurs/modèles
Réglages
```

sans perdre :

```text
le provider actif
le modèle actif
un téléchargement en cours
un benchmark en cours
la configuration non sauvegardée d'un provider lorsque l'architecture actuelle sait déjà la préserver
```

La navigation Réglages générale de Candilog doit rester visible autour de ce contenu.

---

## 39.3 — Configuration provider dépliée dans la page

Quand l'utilisateur clique sur :

```text
Configurer
```

sur une carte provider, **ne pas ouvrir une modal générique si la structure de la maquette peut être respectée**.

La vue de configuration doit se **déplier directement sous les cartes providers**, comme dans `autres_fourniseurs.png`.

Le panneau doit être visuellement rattaché à la carte sélectionnée.

Il doit pouvoir contenir selon le provider :

```text
provider
modèle(s)
endpoint
clé API
options spécifiques
bouton Tester la connexion
état de connexion
informations de stockage sécurisé de la clé
```

Exemple conceptuel :

```text
Claude                                      ✓ Configuré
Anthropic — Configurez votre accès à l'API Anthropic.

Modèles              Endpoint                    Clé API
[ Claude ... ]        [ https://... ]             [ ••••••••••• ] [ œil ]

[ Tester la connexion ]   ✓ Connexion réussie
```

Le panneau doit :

```text
s'ouvrir avec une transition légère ;
se mettre à jour si un autre provider est configuré ;
pouvoir être fermé ;
ne pas perdre les données saisies accidentellement ;
gérer loading / succès / erreur ;
respecter light et dark mode.
```

Évite d'avoir plusieurs panneaux de configuration ouverts simultanément.

---

## 39.4 — Sélecteur rapide global Fournisseur + Modèle

Créer un composant réutilisable conceptuellement équivalent à :

```text
AiQuickSelector
```

ou utiliser un composant existant si l'architecture en possède déjà un adapté.

Ce sélecteur devient **le point d'accès rapide global au provider et au modèle actifs**.

Il doit apparaître dans le shell principal de Candilog, sur **tous les écrans principaux de l'application**, et non uniquement dans les réglages IA.

Il doit être intégré au niveau du layout / header global afin d'éviter de recopier le composant dans chaque feature.

Les écrans d'authentification, de splash ou d'onboarding qui n'utilisent pas le shell principal ne sont pas obligés de l'afficher.

Affichage fermé, proche de la maquette :

```text
[ icône provider ]  IA locale · LFM2.5 1.2B   ●   v
```

ou pour un provider distant :

```text
[ logo ] Claude · Sonnet ...   ●   v
```

Le libellé doit rester compact et lisible.

Il doit afficher au minimum :

```text
provider actif
modèle actif
logo du provider
état global sous forme d'indicateur discret
chevron d'ouverture
```

Ne surcharge pas ce bouton avec CPU, RAM, tokens/s ou autres métriques permanentes.

---

## 39.5 — Dropdown du sélecteur rapide

Au clic, afficher un dropdown à **deux colonnes**, comme dans les maquettes.

### Colonne gauche — Fournisseurs

Afficher les providers avec :

```text
logo
nom
sous-label / société
état discret
sélection active
```

Exemple conceptuel :

```text
Fournisseur

[logo] IA locale Candilog     ✓
       Modèles gérés localement

[logo] Claude                 ●
       Anthropic

[logo] OpenAI                 ●
       GPT

...
```

### Colonne droite — Modèles

Afficher uniquement les modèles du provider actuellement survolé/sélectionné dans le dropdown.

Chaque ligne modèle peut afficher :

```text
logo / icône
nom du modèle
catégorie ou badge lorsque pertinent
état sélectionné
```

Pour l'IA locale Candilog, reprendre les catégories :

```text
LFM2.5 1.2B              Très léger
Ministral 3 3B           Léger
Ministral 3 8B           Équilibré
Ministral 3 14B          Puissant
Mistral Small 3.2 24B    Qualité maximale
```

Un modèle non installé ne doit pas être présenté comme immédiatement utilisable.

Selon le cas, afficher une action claire :

```text
Télécharger
Configurer
Indisponible
```

plutôt qu'un changement silencieux qui échouera ensuite.

---

## 39.6 — Comportement de sélection globale

La sélection rapide doit piloter **la même source de vérité** que les réglages IA.

Interdiction d'avoir :

```text
un provider dans le sélecteur rapide
+
un autre provider dans les settings
+
un troisième état dans une feature
```

Le couple actif doit être centralisé :

```text
activeProviderId
activeModelId
```

ou l'équivalent déjà présent dans l'architecture.

Quand l'utilisateur change de modèle depuis n'importe quel écran :

```text
sélecteur rapide
↓
état global mis à jour
↓
settings IA reflètent immédiatement le changement
↓
prochaine opération IA utilise ce provider + ce modèle
```

Quand l'utilisateur change de modèle depuis la page settings :

```text
settings IA
↓
état global mis à jour
↓
sélecteur rapide reflète immédiatement le changement
```

Si une génération IA est déjà en cours, ne change pas silencieusement le provider de cette requête en plein milieu.

La requête en cours conserve son contexte initial ; le nouveau choix s'applique aux opérations suivantes.

---

## 39.7 — Providers non configurés dans le sélecteur rapide

Un provider non configuré doit rester visible si cela aide à la découverte, mais son état doit être clair.

Au clic sur un provider non configuré :

```text
ne pas lancer une requête
ne pas afficher un faux état connecté
```

Proposer l'action appropriée :

```text
Configurer
```

et ouvrir / naviguer vers :

```text
Réglages → Intelligence artificielle → Autres fournisseurs/modèles
```

avec le bon panneau provider déjà déplié lorsque possible.

---

## 39.8 — Actions à côté du sélecteur rapide

Dans le header global des écrans principaux, reprendre l'intention de la maquette :

```text
[ Sélecteur rapide ] [ Tester ] [ Réglages ]
```

Le bouton `Tester` doit lancer le benchmark du **provider + modèle actuellement actifs** lorsqu'il est disponible.

États :

```text
provider/model prêt
→ Tester actif

modèle local non téléchargé
→ Tester désactivé ou action explicite vers Télécharger

provider distant non configuré
→ Tester désactivé ou action explicite vers Configurer

provider ne supportant pas le benchmark CV
→ état indisponible expliqué
```

Le bouton Réglages doit ouvrir directement la page Intelligence artificielle.

Si l'espace horizontal d'un écran est contraint, adapte proprement les actions secondaires sans masquer le sélecteur principal.

---

## 39.9 — Suppression du monitoring IA dans le rail latéral

Le rail latéral doit rester un **outil de navigation**, pas un tableau de monitoring IA.

Supprime du rail latéral existant les éléments permanents du type :

```text
CPU
RAM / mémoire
charge système
activité modèle
tokens/s en direct
runtime local
modèle actuellement utilisé
nom du provider actuellement utilisé
bloc de monitoring IA
```

si ces éléments y sont actuellement présents.

Ne supprime pas les données techniques du backend si elles sont utiles au diagnostic ou aux benchmarks.

La règle concerne leur **présence permanente dans le rail de navigation**.

Le remplacement UX est :

```text
ancien bloc rail : monitoring CPU/RAM + modèle utilisé
↓
NOUVEAU : sélecteur rapide global Fournisseur + Modèle dans le header du shell
```

Le rail doit donc retrouver une structure plus simple proche des maquettes :

```text
logo Candilog
navigation principale
...
accès aux réglages
```

**Ne transforme pas le rail en navigation détaillée des préférences.** Le réglage `Thème` doit désormais être présenté dans l'onglet `Réglages` de la page Intelligence artificielle, avec `Apparence` et `Son de fin de traitement`, tout en conservant la navigation interne générale des réglages déjà présente dans la page.

Ne conserve pas une deuxième indication redondante du modèle actif dans le rail après l'ajout du sélecteur rapide.

---

## 39.10 — Responsive desktop et robustesse du dropdown

Même si Candilog est une application desktop, le sélecteur doit rester robuste aux différentes tailles de fenêtre.

Prévoir :

```text
largeur minimum de fenêtre
troncature propre des noms de modèles longs
positionnement du dropdown sans sortir de l'écran
hauteur maximale + scroll si beaucoup de modèles
navigation clavier
Escape pour fermer
clic extérieur pour fermer
focus visible
ARIA approprié
```

Le dropdown ne doit pas être coupé par un parent avec `overflow: hidden`.

Il doit rester au-dessus des cartes, tableaux et modales non bloquantes avec une gestion correcte du z-index / portal déjà utilisé par le projet.

---

## 39.11 — Éviter les sélecteurs concurrents dans les features

Analyse les écrans qui possèdent déjà un choix local de provider ou de modèle.

Quand ce choix correspond simplement au provider global de Candilog :

```text
remplace-le par la source de vérité du sélecteur rapide global
```

Ne conserve pas des menus redondants dans chaque page sans raison.

Exception : une feature peut conserver un choix spécifique si son besoin fonctionnel exige réellement un provider/modèle différent pour une opération donnée.

Dans ce cas :

```text
le comportement doit être explicite
le choix local ne doit pas modifier silencieusement le choix global
```

Documente cette exception dans le rapport final.

---

## 39.12 — Architecture frontend attendue pour la sélection IA

La refonte doit éviter le couplage entre le shell, la page settings et les providers.

Réutilise les abstractions existantes ou crée une architecture équivalente à :

```text
ai/
├── components/
│   ├── AiQuickSelector
│   ├── AiProviderList
│   ├── AiModelList
│   └── AiBenchmarkButton
├── hooks/
├── state/
├── providers/
└── models/
```

Ceci est conceptuel.

N'impose pas ces dossiers si Candilog possède déjà une architecture feature-first plus cohérente.

Il faut en revanche avoir :

```text
une source de vérité globale
un composant réutilisable
aucune duplication du catalogue
aucune duplication des états provider/model
aucun hardcode des listes dans plusieurs écrans
```

---

## 39.13 — États UX obligatoires

Les maquettes montrent surtout l'état nominal, mais l'implémentation doit couvrir :

```text
chargement initial des providers
chargement initial des modèles
provider configuré
provider non configuré
provider indisponible
provider en erreur
runtime local absent
runtime local en installation
runtime local prêt
modèle local non téléchargé
modèle local en téléchargement
modèle local installé
modèle actif
benchmark en cours
benchmark terminé
benchmark échoué
aucune métrique disponible
```

Aucun de ces états ne doit provoquer un saut de layout majeur ou un écran ambigu.

Utilise des skeletons / loaders cohérents avec Candilog.

---

## 39.14 — Exemple fonctionnel de l'onglet local

Conceptuellement :

```text
Intelligence artificielle
Utilisez l'IA pour générer, analyser et optimiser vos documents.

[ IA locale · LFM2.5 1.2B ● v ] [ Tester ] [ Réglages ]

IA locale Candilog                         ✓ Prêt
Modèles gérés et optimisés pour votre ordinateur.

Modèle actif       Dernier test            Espace modèles
LFM2.5 1.2B        78 / 100 · 44,3 s       2,1 Go

[ Modèles locaux ] [ Autres fournisseurs/modèles ]

Modèles optimisés pour votre ordinateur

Très léger          Léger              Équilibré          Puissant          Qualité maximale
LFM2.5 1.2B         Ministral 3 3B     Ministral 3 8B     Ministral 3 14B   Mistral Small 3.2 24B
...
```

Les valeurs sont illustratives uniquement.

---

## 39.15 — Règles de cohérence visuelle

Obligatoire :

```text
un seul langage visuel pour local et distant
mêmes hauteurs de boutons comparables
mêmes règles de badges
logos alignés
états actifs immédiatement visibles
texte secondaire moins dominant
pas de bordures excessives
pas de cartes imbriquées inutilement
pas de labels techniques exposés sans nécessité
```

Les termes utilisateur doivent rester simples.

Préférer :

```text
IA locale Candilog
Très léger
Équilibré
Configurer
Tester
Utiliser
Télécharger
```

Éviter dans l'interface principale :

```text
OLLAMA_HOST
baseUrl
quantization interne
PID
port runtime
context window
backend process
```

Ces informations peuvent rester dans le diagnostic avancé lorsque pertinentes.

---

# 40 — Installation / téléchargement UI

Pendant l'installation du moteur :

```text
Installation de l'IA locale

Téléchargement du moteur
████████████░░ 72 %

108 Mo / 150 Mo

[ Annuler ]
```

Puis :

```text
Configuration...
```

Puis :

```text
✓ IA locale prête
```

---

# 41 — Fonctionnement hors connexion

Une fois :

```text
runtime installé
+
modèle installé
```

le fonctionnement local doit rester disponible sans Internet.

Test obligatoire :

```text
installer
↓
télécharger modèle
↓
fermer Candilog
↓
désactiver Internet
↓
relancer
↓
tester CV_BENCHMARK.pdf
↓
import CV utilisateur
↓
succès
```

---

# 42 — Mise à jour runtime

Ne mets pas Ollama à jour silencieusement à chaque lancement.

Conserve une version validée.

Une mise à jour doit suivre :

```text
téléchargement nouvelle version
↓
checksum
↓
validation
↓
arrêt runtime Candilog
↓
switch
↓
health check
```

En cas d'échec :

```text
rollback
```

Les modèles ne doivent pas être retéléchargés.

---

# 43 — Suppression

Permettre :

```text
Supprimer le moteur local
```

et éventuellement :

```text
Supprimer le moteur et les modèles
```

Afficher l'espace disque libéré.

Ne jamais supprimer quoi que ce soit de l'Ollama utilisateur.

---

# 44 — Sécurité

Obligatoire :

```text
HTTPS
SHA-256 avant exécution
URLs officielles
localhost uniquement
chemins contrôlés par Rust
pas d'input utilisateur injecté dans shell
pas de sudo
pas de PATH global
pas de service système
pas de kill global
```

Ne jamais faire :

```rust
Command::new("sh")
    .arg("-c")
    .arg(user_controlled_string)
```

---

# 45 — Gestion des erreurs

Traiter proprement :

```text
pas de connexion
checksum invalide
archive invalide
espace disque insuffisant
permissions
port occupé
runtime crash
timeout
modèle impossible à télécharger
téléchargement annulé
API locale indisponible
benchmark interrompu
JSON invalide
provider non configuré
```

L'UI doit afficher un message humain.

Les détails techniques vont dans le diagnostic.

---

# 46 — Diagnostic

Prévoir notamment :

```text
OS
architecture
runtime
version runtime
runtime path
models path
port
état
dernière erreur
GPU si disponible
```

Option utile :

```text
[ Copier le diagnostic ]
```

Ne jamais inclure les données d'un vrai CV utilisateur.

---

# 47 — Tests unitaires

Tester au minimum :

```text
détection plateforme
manifest runtime
checksum
résolution paths
sélection port
runtime state
model state
scoring CV_BENCHMARK
comparaison ground truth
calcul catégories
calcul score global
calcul temps
parsing métriques provider
benchmarkVersion
```

Le scoring doit être testé très sérieusement.

Pour une même entrée :

```text
même résultat
→ même score
```

---

# 48 — Tests d'intégration runtime

Tester :

```text
runtime absent → installation
runtime présent → pas de téléchargement
11435 occupé → port suivant
Ollama utilisateur 11434 actif → aucun impact
arrêt Candilog → seul runtime Candilog arrêté
offline → fonctionnement
checksum invalide → jamais exécuté
download interrompu → état récupérable
```

---

# 49 — Tests du benchmark utilisateur

Tester au minimum :

## Test normal

```text
CV_BENCHMARK.pdf
↓
pipeline réel
↓
résultat
↓
ground truth
↓
score
↓
modal
```

## Aucun effet sur profil

Avant :

```text
Profile A
```

Lancer benchmark.

Après :

```text
Profile A strictement inchangé
```

## Annulation

```text
benchmark en cours
↓
Arrêter
↓
requête annulée
↓
modal propre
↓
aucune donnée modifiée
```

## JSON invalide

Le benchmark doit retourner une erreur propre et ne jamais crasher.

---

# 50 — Tests avec plusieurs providers

Tester autant que possible :

```text
Managed Ollama
External Ollama
llama.cpp
```

avec :

```text
même CV
même prompt
même modèle lorsque réellement disponible
mêmes paramètres
```

Comparer :

```text
score
temps
tokens/s
```

Cela permettra notamment de vérifier objectivement les différences entre runtimes.

---

# 51 — Architecture code

Respecte l'architecture feature-first existante.

Frontend conceptuel :

```text
local-ai/
├── api/
├── components/
├── hooks/
├── models/
└── views/
```

Rust conceptuel :

```text
local_ai/
├── commands/
├── runtime/
├── services/
├── benchmark/
├── models/
└── dto/
```

N'impose pas ces dossiers si l'architecture existante possède déjà une meilleure organisation.

Analyse avant de créer.

---

# 52 — Conventions Candilog

Tous les noms techniques doivent être en anglais :

```text
files
folders
modules
structs
enums
traits
functions
variables
DTO
services
tests
```

L'interface utilisateur est en français.

Les commentaires explicatifs dans le code peuvent être en français.

---

# 53 — Git

Avant de commencer :

```bash
git status
```

Respecte le travail existant.

Consulte régulièrement :

```bash
git diff
```

Ne commit jamais :

```text
runtime Ollama téléchargé
modèles
blobs
archives
logs locaux
fichiers temporaires
résultats machine du benchmark
```

Les messages de commit doivent être en français.

---

# 54 — Critères d'acceptation runtime

La mission n'est pas terminée tant que :

```text
✓ Candilog fonctionne sans Ollama système
✓ installation IA locale en un clic
✓ aucun terminal
✓ runtime privé
✓ modèles privés
✓ localhost uniquement
✓ port 11435 préféré
✓ fallback port automatique
✓ Ollama utilisateur intact
✓ téléchargements avec progression
✓ checksum
✓ health check
✓ démarrage automatique
✓ arrêt propre
✓ offline après installation
```

---

# 55 — Critères d'acceptation modèles

Doivent apparaître correctement dans l'expérience Candilog :

```text
✓ LFM2.5 1.2B
✓ Ministral 3 3B
✓ Ministral 3 8B
✓ Ministral 3 14B
✓ Mistral Small 3.2 24B
```

avec :

```text
✓ catégorie simple
✓ description
✓ compatibilité machine
✓ taille lorsque connue
✓ téléchargement
✓ progression
✓ installation
✓ suppression
✓ sélection
✓ bouton Tester
```

---

# 56 — Critères d'acceptation benchmark

Obligatoire :

```text
✓ CV_BENCHMARK.pdf réellement utilisé
✓ ground truth JSON réellement utilisée
✓ pipeline réel Candilog
✓ aucune persistance
✓ score /100
✓ détail du score
✓ temps total
✓ temps LLM
✓ tokens input
✓ tokens output
✓ tokens/s
✓ nombre d'appels
✓ bouton Arrêter
✓ bouton Retester
✓ résultat compréhensible
✓ historique du dernier test
✓ benchmarkVersion
✓ fonctionne avec Managed Ollama
✓ fonctionne avec Ollama externe lorsque supporté
✓ fonctionne avec llama.cpp lorsque supporté
✓ architecture extensible aux autres providers
```

---

# 56 bis — Critères d'acceptation UX / sélection globale

La mission n'est pas terminée tant que la refonte UI suivante n'est pas fonctionnelle :

```text
✓ les deux maquettes fournies ont été réellement utilisées comme référence
✓ page Réglages → Intelligence artificielle restructurée
✓ navigation interne existante de la section Réglages conservée et visible
✓ aucun remplacement de la navigation Réglages par la seule navigation des maquettes
✓ onglet Modèles locaux conforme à l'intention de modeles_locaux.png
✓ onglet Autres fournisseurs/modèles conforme à l'intention de autres_fourniseurs.png
✓ nouvel onglet Réglages ajouté à côté des deux onglets IA
✓ section Apparence déplacée dans l'onglet Réglages
✓ réglage Thème déplacé sans régression ni duplication
✓ Son de fin de traitement déplacé sans régression ni duplication
✓ logique et persistance existantes de ces préférences réutilisées
✓ configuration provider dépliée sous les cartes
✓ logos providers correctement affichés
✓ états Configuré / Non configuré / Erreur visibles
✓ sélecteur rapide Fournisseur + Modèle dans le shell global
✓ sélecteur visible sur tous les écrans principaux
✓ dropdown deux colonnes Fournisseur / Modèles
✓ état global provider + modèle partagé avec les settings
✓ changement depuis le sélecteur reflété dans les settings
✓ changement depuis les settings reflété dans le sélecteur
✓ bouton Tester global utilise le provider + modèle actifs
✓ bouton Réglages ouvre directement les réglages IA
✓ provider non configuré redirige proprement vers sa configuration
✓ modèle local non téléchargé n'est pas présenté comme prêt
✓ monitoring CPU/RAM retiré du rail latéral
✓ modèle/provider actif retiré du rail latéral
✓ aucune information IA redondante conservée dans le rail
✓ rail latéral recentré sur la navigation
✓ light mode vérifié
✓ dark mode vérifié
✓ navigation clavier du sélecteur vérifiée
✓ fermeture Escape / clic extérieur vérifiée
✓ noms longs et petites largeurs vérifiés
✓ aucune duplication du catalogue providers/modèles dans le frontend
```

---

# 57 — Scénario critique Ollama existant

Ce scénario doit obligatoirement fonctionner :

```text
Ollama utilisateur actif
127.0.0.1:11434
plusieurs modèles personnels
↓
Candilog démarre
↓
activation IA locale
↓
Candilog NE TOUCHE PAS à Ollama utilisateur
↓
runtime privé téléchargé
↓
modèles privés
↓
runtime Candilog sur 11435
↓
les deux fonctionnent
↓
Candilog fermé
↓
runtime Candilog arrêté
↓
Ollama utilisateur continue de fonctionner
```

---

# 58 — Scénario critique benchmark

Ce scénario doit également fonctionner :

```text
LFM2.5 1.2B installé
↓
utilisateur clique Tester
↓
modal s'ouvre
↓
CV_BENCHMARK.pdf chargé automatiquement
↓
pipeline réel d'import
↓
modèle sélectionné
↓
résultat temporaire
↓
comparaison ground truth
↓
score calculé
↓
métriques calculées
↓

78 / 100
44,3 s
XX tokens/s

↓
profil utilisateur inchangé
```

Les valeurs ci-dessus sont uniquement un exemple d'affichage.

Ne hardcode jamais un résultat.

---

# 59 — Rapport final

À la fin, fournis :

```text
ARCHITECTURE RETENUE

...

MANAGED OLLAMA

Version :
Stockage :
Port :
Lifecycle :
Isolation :

CATALOGUE MODÈLES

LFM2.5 1.2B :
Ministral 3 3B :
Ministral 3 8B :
Ministral 3 14B :
Mistral Small 3.2 24B :

BENCHMARK UTILISATEUR

CV utilisé :
Ground truth :
Version benchmark :
Pipeline :
Scoring :
Métriques :

REFONTE UX IA

Maquettes analysées :
Structure settings :
Sélecteur rapide global :
Source de vérité provider/modèle :
Dropdown providers/modèles :
Configuration dépliée :
Modifications du rail latéral :
Écrans adaptés :
Exceptions de sélection locale conservées :

TESTS EFFECTUÉS

Managed Ollama :
Ollama externe :
llama.cpp :
Offline :
Cohabitation Ollama :
Benchmark :
Annulation :
Non-régression :

IMPORT CV MULTI-CV

Référence avant :
78,0 / 100
44,3 s

Résultat après :
...

FICHIERS CRÉÉS

...

FICHIERS MODIFIÉS

...

LIMITES

...
```

---

# Règle finale

Le résultat doit donner l'impression que **l'IA locale est une fonctionnalité native de Candilog**.

L'utilisateur doit simplement avoir :

```text
Installer l'IA locale
↓
Choisir son modèle
↓
Tester
↓
Utiliser
```

Les modèles proposés doivent être :

```text
Très léger
→ LFM2.5 1.2B

Léger
→ Ministral 3 3B

Équilibré
→ Ministral 3 8B

Puissant
→ Ministral 3 14B

Qualité maximale
→ Mistral Small 3.2 24B
```

Et le test d'un modèle doit réellement répondre à deux questions :

```text
1. Est-ce que ce modèle comprend correctement un CV dans Candilog ?

2. Est-ce qu'il est suffisamment rapide sur CET ordinateur ?
```

La modal doit donc toujours mettre en avant :

```text
SCORE : XX / 100
TEMPS : XX.X s
TOKENS/S : XX.X
```

La navigation IA doit également donner l'impression d'une fonctionnalité native et toujours accessible :

```text
n'importe quel écran principal
↓
sélecteur rapide Fournisseur + Modèle
↓
changer de provider ou modèle en quelques clics
↓
les réglages IA et toutes les prochaines opérations restent synchronisés
```

Dans la page Réglages, **conserve impérativement la navigation Réglages existante**. À l'intérieur de `Intelligence artificielle`, les sous-onglets sont désormais :

```text
Modèles locaux
Autres fournisseurs/modèles
Réglages
```

Le troisième onglet regroupe notamment :

```text
Apparence → Thème
Son de fin de traitement
```

Le rail latéral ne doit plus servir à afficher le monitoring CPU/RAM ni le modèle actif.

Le test `CV_BENCHMARK.pdf` est un **benchmark utilisateur rapide**.

Il ne remplace jamais le benchmark interne multi-CV utilisé pour optimiser les prompts et mesurer sérieusement la qualité générale des modèles.