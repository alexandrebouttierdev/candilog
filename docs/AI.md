# IA native

Toute l'IA vit dans `src-tauri/src/features/ai/`. Le frontend n'envoie que des DTO et
écoute la progression : **aucun prompt dans React**.

## Fournisseurs

L'**IA locale Candilog** (`candilog_local`), Ollama, Claude, OpenAI, Gemini,
Mistral, DeepSeek et un point de terminaison personnalisé implémentent `LlmGenerator`. Le
choix, le modèle, la température et le mode d'analyse sont persistés dans les paramètres ;
**chaque fournisseur conserve sa propre configuration** (endpoint, modèle, température,
mode) dans `llm_presets`, et sa clé API dans une entrée distincte du coffre système
(`llm-api-key-openai`, `llm-api-key-mistral`, …) — basculer d'un fournisseur à l'autre
n'écrase jamais les réglages des autres. La clé n'est jamais écrite dans SQLite ni dans
les journaux.

L'IA locale Candilog est le **fournisseur par défaut** sur une installation neuve. Elle
délègue l'inférence à un runtime Ollama privé géré par l'application : binaire officiel
téléchargé et vérifié (SHA-256), processus isolé sur `127.0.0.1` à partir du port 11435,
répertoire de modèles séparé de l'Ollama utilisateur (`:11434`). Le catalogue, les pulls,
l'activation et le benchmark utilisateur passent par `ManagedOllamaService` ; l'adaptateur
HTTP Ollama existant est réutilisé avec l'endpoint local du processus géré. Le badge
**Recommandé** du catalogue met en avant **Ministral 3 3B** (`ministral-3:3b`) dès que la
RAM le permet — bon compromis pour lettres, CV et analyses. Windows n'est
pas encore supporté (archive `.zip` non extraite).

HTTPS obligatoire hors Ollama, adresses privées refusées pour un point de terminaison
distant, réponse plafonnée à 5 Mio, PDF source plafonné à 10 Mio.

Chaque appel HTTP est **repris jusqu'à trois fois** sur un échec transitoire — délai
dépassé, connexion impossible, `429`, `5xx` — avec une attente de 1 s puis 2 s. Une
génération de CV enchaîne quatre appels (offre, socle, relecture, sélection) et peut durer
une à deux minutes : sans reprise, un
incident réseau passager sur le dernier annulait tout le travail et laissait payés les deux
appels déjà aboutis. Une erreur de configuration (`4xx` : clé refusée, modèle inconnu)
n'est **jamais** reprise — la retenter ne ferait que retarder le message que l'utilisateur
doit lire.

Ollama externe tourne sur la machine de l'utilisateur : une connexion impossible y renvoie un
message qui le nomme et renvoie aux réglages, pas le « Vérifiez votre réseau » des erreurs
HTTP. La reprise vit dans l'adaptateur de transport (`infrastructure/provider.rs`),
donc tous les appels en bénéficient, et l'annulation reste immédiate : `ai_cancel` abandonne
le futur qui porte la boucle, attente comprise.

Elle ne remplace pas la reprise de `generate_json`, qui vise un tout autre défaut : une
réponse HTTP valide dont le corps n'est pas le JSON attendu. Les deux se cumulent —
transport d'abord, forme de la réponse ensuite. L'import de profil en ajoute une troisième,
qui porte sur le contenu et non sur la forme (voir « Sorties du modèle »).

## IA locale Candilog (détails)

`candilog_local` est le seul fournisseur dont le modèle actif ne vit pas dans `llm.model` :
il est désigné par `managed_ollama.active_model_id`. L'écran de réglages affiche
`ManagedOllamaPanel` au lieu d'un champ « Modèle ». Les garde-fous de configuration
(`LlmConfig::est_configure`, `manquants` / `iaEstConfiguree`) l'exemptent donc du champ
« modèle » ; l'exiger rendait le fournisseur inutilisable dès que la grille le sélectionnait.

Le catalogue des modèles, les pulls, l'activation et le benchmark utilisateur passent par
`ManagedOllamaService` (`domain/managed_ollama.rs`). Les commandes IPC dédiées sont
`get_managed_ollama_status`, `install_managed_ollama_model`, `cancel_managed_ollama_download`,
`remove_managed_ollama_model` et `activate_managed_ollama_model`. Le téléchargement publie
`managed-ollama://download-progress`, `managed-ollama://download-completed` et
`managed-ollama://download-error`.

Les anciens réglages `mistral_local` (llama.cpp) sont migrés automatiquement vers
`candilog_local` au chargement ; le bloc `local_ai` est ignoré. Les fichiers `.gguf` restent
sur le disque mais ne sont plus gérés.

La commande `system_resource_snapshot` alimente le rail (CPU %, RAM %, VRAM %). Elle
s'appuie sur `sysinfo` sans démarrer le runtime Ollama.

## Sorties du modèle

`AiService` porte le parsing d'offre et de CV, la génération, l'ATS, le grounding et les
lettres. La chaîne est toujours **parse → validate → use** : le JSON brut du modèle
n'est jamais utilisé tel quel, il est réparé si besoin (`jsonrepair-rs`), désérialisé et
borné (`domain/validation.rs` : `MAX_SOURCE_CHARS`, `MAX_CONTEXT_CHARS`,
`MAX_STRUCTURED_CHARS`, `MAX_ITEMS`, `MAX_ITEM_CHARS`). Les documents générés sont ensuite
recadrés sur les faits réels par `ground_generated_resume`, `ground_imported_resume` et
`ground_extracted_listing`.

Lors d'un import de profil, les dates recopiées librement par les petits modèles locaux
(`Juil. 2019`, `12/2023`, `aujourd'hui`) sont normalisées après la validation de taille.
Une date avec une année est ramenée à `AAAA-MM` ou `AAAA`, une fin « en cours » marque le
poste comme actuel et un fragment inexploitable est vidé. Une variation de format ne fait
donc plus perdre toute une analyse de CV ; les autres bornes de sortie restent inchangées.

### Import de CV : Vision par défaut, Texte en repli

L'import de profil (`ai_import_profile`) propose deux méthodes :

| Méthode | Pipeline |
| --- | --- |
| **Vision** (recommandé) | PDF → rendu pages (`pdftoppm`) → modèle multimodal (+ texte Rust complémentaire) → JSON |
| **Texte** | PDF → extraction Rust (`pdftotext -layout` / flux) → modèle texte → JSON |

La capacité Vision est déterminée pour le **modèle réellement sélectionné** (métadonnées
Ollama `/api/show` si disponibles — y compris via un endpoint Custom qui répond à
`/api/show` — sinon catalogue Candilog et familles connues). Ministral 3 du catalogue
local est Vision ; LFM2.5 ne l'est pas. **Gemma 3** : les variantes `4b` et plus sont
Vision ; `gemma3:1b` et `270m` sont texte-only (un envoi d'images provoque un HTTP 400
chez Ollama).

Comportement :

1. préférence `vision` + modèle compatible → Vision, avec **repli automatique Texte** en cas
   d'échec (rendu, provider, JSON, timeout…) ;
2. préférence `vision` + modèle text-only → Texte sans erreur ;
3. préférence `text` → Texte uniquement, Vision jamais appelée.

Le résultat (`ProfileImportAnalysis`) indique `method_used` et `fallback_used`. Les invites
Vision sont distinctes : le document visuel prime, le texte brut n'est qu'un complément.
Le recadrage (`ground_imported_profile`) s'applique dès qu'un texte complémentaire est
disponible. L'UI expose le choix Vision / Texte, désactive Vision si indisponible, et
signale un repli éventuel.

Le gabarit envoyé au modèle **décrit** chaque valeur attendue (« prénom du candidat ») au
lieu de la laisser vide. Un gabarit rempli de `""` était recopié tel quel par les petits
modèles : `llama3.2:1b` renvoyait le squelette intact et l'import échouait faute de données.
Les libellés comptent plusieurs mots exprès — recopiés faute d'information, ils ne figurent
dans aucun CV et le recadrage les écarte.

Une extraction qui ne rapporte ni identité, ni expérience, ni compétence relance **un**
second appel, en disant au modèle ce qui manquait à sa réponse. Ce défaut-là échappe à la
reprise de `generate_json` : le gabarit recopié et `{}` sont du JSON valide. Si le second
essai ne donne rien non plus, l'erreur désigne le modèle et invite à en choisir un plus
grand, au lieu de laisser croire que le CV est en cause.

Le profil extrait est ensuite recadré sur le CV par `ground_imported_profile` : tout texte
qui n'apparaît pas dans le passage réellement soumis au modèle est effacé, et les entrées
ainsi vidées de leur libellé sont retirées. Les dates, déjà reformatées, en sont exclues.
Le rapprochement passe par `search_key` et exige des frontières alphanumériques : un modèle
de 350 M renvoyait sinon des fragments de domaine (`.com`, `.fr`, `.org`), des morceaux de
mots (`.franc`) et jusqu'à une certification absente du document, que le seul rejet des
valeurs vides laissait arriver dans l'écran de revue.

Une offre d'emploi ou un PDF importé est de la **donnée**, jamais des instructions : les
contenus non fiables sont encadrés par `bloc_donnees`, dont la balise porte un identifiant
tiré au sort à chaque appel et dont la balise fermante est neutralisée dans le contenu — un
délimiteur fixe pouvait figurer dans l'offre elle-même et refermer le bloc.

Le récapitulatif et les reformulations de `AtsAnalysis` restent du texte libre borné. Les
recommandations de contenu, elles, ne transportent que des identifiants du catalogue du
profil, une justification et une pertinence qualitative. `ground_content_recommendations`
écarte les identifiants inconnus et les doublons avant l'éditeur.

La lettre de motivation est **assemblée**, pas rédigée librement : le modèle ne renvoie
qu'une sélection d'identifiants du catalogue de faits et des mots-clés du brief
(`domain/cover_letter.rs`). Les invites système détaillent la priorité des faits
(experience → summary → skill…) et le plafond selon la longueur demandée, pour guider
les petits modèles locaux. Un identifiant inconnu invalide la réponse ; un mot-clé absent
du brief est simplement écarté, parce qu'une paraphrase du modèle ne justifie pas de faire
échouer toute la rédaction — la lettre reste dans tous les cas limitée aux faits vérifiés.
Si le plan est vide ou trop court, l'assemblage **complète** avec les faits du catalogue
dans le même ordre de priorité, pour éviter une lettre réduite à l'ouverture et la
formule de politesse.

La composition de la lettre produit un français de candidature (ouverture selon le ton,
faits reliés au poste, clôture) : la préposition est **élidée**
devant une voyelle (`core::utils::text::elider`, jumeau de `letterLayout.ts`) — « au poste
d'Administrateur », jamais « au poste de Administrateur » —, et un fait repris du profil est
ramené à une fin de phrase unique, ses retours à la ligne aplatis et sa ponctuation finale
dédoublonnée.

Les itérations de l'écran passent par le champ `instruction` du brief : les consignes
successives sont cumulées et renvoyées ensemble, faute de quoi « plus court » puis « plus
formel » ne vaudraient jamais en même temps. Elles orientent la **sélection de faits**, pas
la prose : le corps reste assemblé par Candilog. Quand une lettre précédente est fournie
(`previous_cover_letter`), le brief d'itération omet le long contexte d'offre, compacte le
catalogue et réutilise un prompt d'ajustement : une simple retouche ne doit plus reconstruire
toute la lettre ni renvoyer des dizaines de milliers de tokens.

Une relecture française termine désormais les générations de CV et de lettre. Elle échange
une liste de champs `{id, text}` plutôt que le document complet : les identifiants inconnus
ou dupliqués sont ignorés, l'ordre vient toujours de la requête et les chiffres, coordonnées,
noms propres à majuscule et technologies à casse distinctive doivent rester présents. Une
réponse vide, beaucoup plus courte ou plus longue, ou qui perd un de ces fragments est
remplacée localement par le texte source. Le même contrat sert au bouton manuel « Corriger
l'orthographe » ; le CV ne transmet que ses champs de prose, et la lettre réinjecte chaque
fragment dans sa mise en forme existante.

Le score ATS affiché est toujours le calcul déterministe Rust (`profile_score` /
`score_resume_imported`, `domain/scoring.rs`), jamais le chiffre renvoyé par le modèle.

L'exigence d'expérience est lue **à côté d'une mention d'année**, et non comme le premier
entier du texte : « Bac+3, 5 ans d'expérience » demande cinq ans, pas trois. Une fourchette
vaut par son minimum — « 2 à 5 ans » n'écarte pas un profil de deux ans.

Une compétence de l'offre est **couverte dès qu'une compétence du candidat la contient comme
mot entier** : « VMware vSphere 7/8 » couvre « VMware », « Windows Server 2016/2019/2022 »
couvre « Windows ». L'égalité stricte des clés normalisées exigeait le libellé exact de
l'offre : un profil réel, qui nomme ses technologies précisément, affichait zéro compétence
couverte, et l'éditeur lui proposait d'ajouter une compétence déjà présente sous son nom
complet. La frontière de mot reste celle de `contains_search_term` — « Java » ne couvre
toujours pas « JavaScript ».

## Assistance éditoriale du CV

Le profil est une bibliothèque ; le CV n'en est pas une copie. Le socle initial contient
l'identité, les coordonnées, les expériences et les formations. Compétences, projets,
certifications et langues commencent hors du document et restent disponibles dans
**Suggestions**.

La troisième étape de génération reçoit l'offre, le socle courant et le catalogue optionnel.
Elle renvoie au plus huit candidates pertinentes, jamais le catalogue complet. Rust vérifie
leurs identifiants, les trie par pertinence qualitative puis en retient au plus quatre après
simulation cumulée dans le moteur PDF. Un ajout qui déborde n'est pas recommandé. Si le CV
est plein, un remplacement plus pertinent peut être proposé, sans modifier le document.

Accepter, ignorer, retirer ou réajouter est local à l'éditeur. Ces intentions sont conservées
dans `ResumeEditorialDecisions` : un élément ignoré ou retiré ne revient pas au recalcul, mais
reste disponible dans Suggestions. Ajouter manuellement retire l'élément des Suggestions ;
le retirer l'y remet immédiatement.

Le LLM est appelé pendant la génération pour extraire l'offre, adapter le socle, classer
sémantiquement les candidates et effectuer la relecture finale. Dans l'éditeur, seul un clic
explicite sur « Corriger l'orthographe » déclenche un autre appel. Présence dans le CV,
filtrage, score, impact ATS de chaque action, décisions, simulation et place disponible sont
calculés localement ; les autres interactions ne déclenchent donc aucun appel au modèle.

Chaque recommandation du modèle (`AtsRecommendation`) cible une section **fermée** :

| Section | Cible |
| --- | --- |
| `profile` | texte du profil (`item_index` absent) |
| `experience` | description d'une expérience (`item_index` = indice 0-based) |

`validate_ai_output` rejette une recommandation mal ciblée (profil avec indice, expérience
sans indice, indice hors limites). Le champ `impact` n'existe plus : le modèle ne déclare
aucun gain de score.

Dans l'éditeur, chaque recommandation applicable devient une `ResumeProposal`. Son **gain**
(`proposal.gain`) est simulé localement par `simulate_gain` sur une copie du document
(`build_proposals`, `recalculate`) — jamais repris du LLM. Les recommandations de contenu
exposent de même un `score_delta`, obtenu en recalculant le score sur une copie du document
avec l'ajout ou le remplacement simulé. Une proposition non applicable
(texte modifié depuis la génération) reste visible avec son statut mais sans action possible.

Les compétences manquantes de l'offre (`MatchScore.missing`) qui n'existent pas dans le
profil sont affichées comme **écarts à vérifier**, jamais comme compétences possédées ni
comme actions d'ajout au CV.

Les **expériences et les formations** ne sont pas une sélection : la consigne de génération
est de toutes les conserver, le modèle n'en choisit que l'ordre et la mise en avant. Le
recadrage sur les faits écarte pourtant toute entrée que le modèle n'a pas recopiée à
l'identique — reformuler « BTS SIO » en « BTS Services informatiques aux organisations »
suffisait à faire disparaître le diplôme du CV. `prepare_workspace` **complète** donc la
liste générée par les entrées du profil qu'elle a laissées de côté, à la suite et dans
l'ordre du profil. Les compétences, elles, restent une sélection : c'est leur rôle
vis-à-vis de l'offre ; elles restent désormais dans la bibliothèque tant qu'aucun choix
utilisateur ne les ajoute.

## Progression et annulation

Les traitements sont asynchrones côté Rust. La progression remonte par événements Tauri :
`ia-progression` pour la génération et l'analyse, `profile_import_progress` pour l'import
de profil. La télémétrie est best-effort et ne masque jamais le résultat métier.

Aucun de ces événements ne porte de pourcentage : la durée dépend du fournisseur et du
modèle, et un chiffre calculé à partir du numéro d'étape n'était qu'une constante déguisée
en mesure. Les événements transportent l'étape en cours, les tokens cumulés déjà connus et,
pour la lettre, les fragments de texte ; l'interface affiche une barre indéterminée avec le
temps écoulé.

Chaque commande IA réussie retourne son résultat dans `AiExecution<T>`, avec la durée
native `elapsed_ms` et le total `tokens_used` communiqué par le fournisseur. L'interface
conserve ces métriques après la progression et les affiche ensemble, par exemple
« Généré en 18,4 s · 1 024 tokens ». Si un endpoint compatible ne fournit pas sa
consommation, Candilog affiche « tokens non communiqués » au lieu d'inventer un zéro. Un
traitement composé de plusieurs appels, reprise JSON comprise, cumule leurs tokens.

La fin d'un traitement est aussi annoncée par un signal sonore, émis une seule fois dans
`features/ai/services/aiService.ts` pour que plus aucun écran ne puisse l'oublier. La
préférence « Son de fin de traitement » (Réglages → IA) est locale à la machine
(`shared/lib/completion-sound.ts`, `localStorage`) et active par défaut.
Une opération annulée reste muette même si sa promesse frontend se résout tardivement.

Chaque génération possède un `CancellationToken`, indexé par `generation_id` ;
`ai_cancel` le déclenche. L'annulation abandonne le futur en cours — la requête HTTP est
portée par ce futur, elle s'interrompt donc avec lui. Relancer une génération avec un
identifiant déjà actif annule la précédente.

Le frontend enregistre l'unique traitement actif dans la feature IA. Tous les écrans
utilisent le même cycle de vie : l'arrêt invalide d'abord l'identifiant pour ignorer toute
réponse tardive, désabonne la progression, arrête le chronomètre, puis appelle `ai_cancel`.
La coque bloque une navigation interne tant que ce traitement est actif et ne la poursuit
qu'après confirmation et transmission de l'arrêt au backend.

## Benchmark utilisateur (`CV_BENCHMARK.pdf`)

Le bouton **Tester** (en-tête global, héros des réglages IA, carte de chaque modèle local
installé) lance `run_user_cv_benchmark`. Le PDF de référence et sa ground truth
(`src-tauri/resources/CV_BENCHMARK.pdf`, `CV_BENCHMARK.expected.json`) sont **compilés dans
le binaire** (`include_bytes!` / `include_str!`) : le PDF est ensuite écrit dans un fichier
temporaire pour les extracteurs. Ils ne figurent pas dans le bundle Tauri
(`tauri.conf.json` → `bundle.resources`) et ne doivent pas dépendre de `CARGO_MANIFEST_DIR`
à l'exécution — ce chemin n'existe que sur la machine de build. Le pipeline suit le même
orchestrateur que l'import de profil (`method` Vision ou Texte, repli éventuel) :
prétraitement PDF, invite, post-traitements, scoring déterministe contre la ground truth.
Aucune donnée utilisateur n'est persistée ; le profil extrait est jeté après calcul du
score.

`UserBenchmarkResult` expose `method_used` et `fallback_used` pour comparer Texte, Vision et
Vision hybride (images + texte complémentaire) sur un même modèle compatible.

Le benchmark fonctionne avec **tout fournisseur configuré** (IA locale Candilog, Ollama
externe, cloud). Les providers distants affichent un avertissement : le CV de
référence sera envoyé au service configuré. Le score (0–100), la qualité qualitative, les
métriques de durée et le détail par catégorie sont renvoyés dans `UserBenchmarkResult`.
L'annulation réutilise `ai_cancel` et le `generation_id` de la session.

Ce test ne remplace pas le benchmark multi-CV de développement (`examples/cv_import_baseline.rs`
et fixtures associées).

## Interface IA

L'écran Paramètres → Intelligence artificielle comporte deux onglets : **IA locale**
(catalogue Ollama géré) et **IA online/personnalisé** (grille distante). Thème et son
vivent dans **Paramètres → Customisation**. Le sélecteur rapide global (`AiQuickSelector` dans la barre supérieure)
synchronise le fournisseur actif avec les paramètres persistés.

## Cache

Il n'y a pas de cache de réponses IA. La table `ai_cache`, la commande
`settings_clear_ai_cache` et le bouton « Vider le cache IA » ont été retirés : rien
n'alimentait la table, et l'écran des réglages annonçait à l'utilisateur un effet qui ne se
produisait jamais. Une base de développement antérieure conserve la table, désormais
orpheline et sans usage.
