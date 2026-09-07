# IA native

Toute l'IA vit dans `src-tauri/src/features/ai/`. Le frontend n'envoie que des DTO et
écoute la progression : **aucun prompt dans React**.

## Fournisseurs

Mistral Local, Ollama, Claude, OpenAI, Gemini, Mistral, DeepSeek et un point de terminaison
personnalisé implémentent `LlmGenerator`. Le choix, le modèle, la
température et le mode d'analyse sont persistés dans les paramètres ; la clé API vit dans
le coffre du système (`core::secrets`), jamais dans SQLite ni dans les journaux.

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

Ollama tourne sur la machine : une connexion impossible y renvoie un message qui le nomme
et renvoie aux réglages, pas le « Vérifiez votre réseau » des erreurs HTTP. C'est le
fournisseur par défaut, donc le premier écueil d'une installation neuve où Ollama n'est pas
encore installé. La reprise vit dans l'adaptateur de transport (`infrastructure/provider.rs`),
donc tous les appels en bénéficient, et l'annulation reste immédiate : `ai_cancel` abandonne
le futur qui porte la boucle, attente comprise.

Elle ne remplace pas la reprise de `generate_json`, qui vise un tout autre défaut : une
réponse HTTP valide dont le corps n'est pas le JSON attendu. Les deux se cumulent —
transport d'abord, forme de la réponse ensuite.

## Mistral Local

Mistral Local est un provider distinct d'Ollama. `MistralLocalProvider` adapte le runtime
embarqué llama.cpp à `LlmGenerator` ; aucun autre service n'appelle llama.cpp directement.
Il ne lance ni serveur, ni processus, ni CLI. Le modèle est chargé par `mmap` à la première
requête, réutilisé entre les requêtes, remplacé sous verrou (jamais deux modèles chargés),
libéré à la fermeture de la fenêtre principale et rendu au système après cinq minutes sans
inférence — une surveillance installée au démarrage appelle `release_idle_model` chaque
minute. Sans elle, un import de CV immobiliserait les poids jusqu'à la fermeture.

Mistral Local est le seul fournisseur dont le modèle ne vient pas de `llm.model` : son
artefact est désigné par `local_ai.active_model_id`, et l'écran de réglages affiche
`MistralLocalPanel` au lieu d'un champ « Modèle ». Les deux garde-fous de configuration
l'exemptent donc du champ « modèle » : `LlmConfig::est_configure` côté Rust et
`manquants` (`src/features/settings/model/etatIa.ts`) côté interface, ce dernier pilotant
`AiRequiredModal` via `useAiRequiredStore`. L'exiger rendait le fournisseur inutilisable dès
que la grille le sélectionnait, puisqu'elle vide ce champ. Les deux gardes doivent rester
d'accord : n'en corriger qu'un déplace le blocage d'une couche à l'autre. Un modèle absent
est signalé plus précisément en aval, par `LocalAiService::provider` (« Installez l'IA
locale avant de l'utiliser »).

`domain/local_ai.rs::ModelRegistry` est l'unique source des artefacts. Quatre entrées sont
verrouillées sur un commit et un SHA-256 : trois GGUF officiels Mistral, plus un Qwen3 1.7B
Q4_K_M (le dépôt officiel `Qwen/Qwen3-1.7B-GGUF` ne publie que du Q8_0 ; l'artefact Q4_K_M
retenu vient de `unsloth/Qwen3-1.7B-GGUF`).

| Profil UI | Famille | Dépôt / fichier Q4_K_M | Révision | Octets | SHA-256 |
| --- | --- | --- | --- | ---: | --- |
| Ultra léger | Qwen | `unsloth/Qwen3-1.7B-GGUF` / `Qwen3-1.7B-Q4_K_M.gguf` | `d7f544eead698dbd1f15126ef60b45a1e1933222` | 1 107 409 472 | `b139949c5bd74937ad8ed8c8cf3d9ffb1e99c866c823204dc42c0d91fa181897` |
| Léger | Mistral | `mistralai/Ministral-3-3B-Instruct-2512-GGUF` / `Ministral-3-3B-Instruct-2512-Q4_K_M.gguf` | `eb599d408350ea2bb60452cb86be7c7b2fc28227` | 2 147 023 008 | `9ed150d4367e68df0ac8e1540f6ddc65b42d0ee26378329d1ecbca60f93fc5f8` |
| Équilibré | Mistral | `mistralai/Ministral-3-8B-Instruct-2512-GGUF` / `Ministral-3-8B-Instruct-2512-Q4_K_M.gguf` | `0102285ad796bd99af90f58de616092e5630e970` | 5 198 911 904 | `33e7a72cf5e6e2cfc2f2847075acc013d68bba023e35310cef86b5cf8fdca761` |
| Qualité | Mistral | `mistralai/Ministral-3-14B-Instruct-2512-GGUF` / `Ministral-3-14B-Instruct-2512-Q4_K_M.gguf` | `74fac473c43357d7fb2671713608183cc72496d0` | 8 239 593 024 | `824e0f3373e69b84f2cae46fdcb9bd1ebc6ab3bfc7acc125d818b7b8178cc613` |

La sélection teste Qualité, Équilibré, Léger puis Ultra léger. L'écran de réglages affiche pour chaque profil le poids disque, la RAM et la VRAM recommandées, et le nombre de cœurs (`recommended_*` du registre). Les seuils nominaux sont
24/16/8/4 Gio de RAM ou de mémoire unifiée et 12/8/4/2,5 Gio de VRAM. Elle exige en plus
que l'artefact et ses buffers tiennent dans 68 % de la RAM (Apple compris) ou 80 % de la
VRAM, que la mémoire actuellement disponible suffise et, en CPU, que 8/6/4/2 cœurs
physiques soient présents. Un modèle seulement chargeable est `NotRecommended` et n'est
jamais choisi automatiquement. Sur Apple Silicon, RAM et VRAM ne sont jamais additionnées.

Le downloader n'accepte comme source initiale que `https://huggingface.co`, limite les
redirections aux domaines de distribution Hugging Face, vérifie l'espace disque avec une
réserve de 512 Mio, écrit le flux dans `.part`, reprend avec HTTP Range et ne publie le
`.gguf` par renommage qu'après contrôle de la taille et du SHA-256. Une empreinte est aussi
recontrôlée avant le premier chargement de chaque session.

Le contexte normal est 8 192 tokens (16 384 réservé techniquement) ; le lot
d'inférence (`n_batch`) reste à 512 pour limiter la RAM allouée lors d'un import de CV.
L'invite est donc **décodée par lots** de cette taille (`lots_de_decodage`), les logits
n'étant demandés que sur l'ultime jeton. Ce découpage n'est pas une optimisation :
`llama_context::decode` impose `n_tokens_all <= n_batch` et abandonne le processus par
`GGML_ASSERT` au-delà. Décoder une invite d'un seul bloc faisait donc planter Candilog pour
tout texte dépassant 512 jetons — c'est-à-dire n'importe quel CV, le plafond de 12 000
caractères en produisant plusieurs milliers. Le contrôle de longueur en tête d'inférence
borne le **contexte** (cache KV), jamais la taille d'un appel à `decode` : les deux limites
sont distinctes et doivent être tenues séparément.

L'échantillonnage ne doit **jamais** accepter le jeton lui-même : `llama_sampler_sample`
appelle déjà `llama_sampler_accept`. Un second appel faisait avancer la grammaire JSON de
deux pas par jeton ; ses piles d'analyse se vidaient, et l'appel suivant abandonnait le
processus sur `GGML_ASSERT(!stacks.empty())`. Le défaut ne touchait que les générations
sous grammaire (`json: true`) — donc l'import de CV, jamais le benchmark.

Ces deux pièges ont la même signature : un `abort()` de llama.cpp, que Rust ne peut ni
intercepter, ni convertir en `Err`, ni journaliser. Seule une inférence réelle les révèle,
d'où le scénario `tests/e2e_local_ai.rs` (`CANDILOG_E2E_LOCAL_AI=1`), qui couvre la
génération sous grammaire, le texte libre et l'annulation.

### Annulation, plafond et progression

Une génération locale s'exécute dans un `spawn_blocking`, que Tokio **ne sait pas
interrompre** : abandonner le futur rend la main à l'interface pendant que les cœurs
continuent de calculer. `MistralLocalRuntime` expose donc un jeton d'annulation, consulté
entre chaque lot de préremplissage et à chaque jeton produit ; `AiService::cancel` le
déclenche en plus du jeton du futur.

Le plafond de sortie dépend de la nature de la réponse : `MAX_OUTPUT_TOKENS` (4 096) en
texte libre, `MAX_JSON_OUTPUT_TOKENS` (3 072) pour une sortie structurée. Ce second chiffre
est mesuré, pas estimé : tokenisés avec le modèle, les profils de `tests/fixtures/profiles/`
pèsent 336 à 1 928 jetons, et le cas volontairement trop long pour une page A4 en pèse
2 634. Une assertion `const` interdit d'abaisser le plafond sous cette valeur — le tronquer
produirait un JSON invalide, soit un échec là où l'on avait un résultat lent.

Pendant l'analyse d'un CV, `cancel_avec_progression` réveille l'appelant chaque seconde et
publie l'étape « Analyse du CV… N jetons · X jeton/s · T s ». À 2,6 jetons/s sur un portable
quadricœur, une analyse dure une douzaine de minutes : sans ce battement, rien ne distingue
une génération lente d'un blocage. Le message reste vide pour ne pas gonfler le journal
d'import ; seul `step` est remplacé. Un fournisseur distant ne publie aucun avancement, et
le battement reste alors muet.

### Garde-fou mémoire

Sous Linux, une allocation excessive n'échoue pas : le noyau l'accorde puis tue le processus
(OOM killer). Ce `SIGKILL` n'est ni interceptable, ni journalisable — filtrer `out of memory`
dans les erreurs de llama.cpp ne protège donc de rien. La seule défense est de refuser
l'inférence **avant** de réserver la mémoire.

`domain/local_ai.rs::local_ai_memory_shortfall_mb` compare la RAM réellement disponible au
besoin du modèle augmenté de `LOCAL_AI_SYSTEM_MARGIN_MB` (768 Mio laissés au système, à la
fenêtre WebKit et au reste de Candilog). `MistralLocalRuntime::generate` l'évalue à chaque
inférence et remonte une erreur chiffrée (« il manque environ N Mo ») plutôt que de laisser
le noyau trancher. Un modèle déjà chargé occupe déjà la RAM mesurée : ses poids ne sont pas
recomptés, sinon le garde-fou refuserait toute inférence après le premier chargement.

Cette vérification est distincte de la compatibilité affichée à l'installation
(`ensure_supported`), qui n'est évaluée qu'une fois : la mémoire disponible, elle, varie
pendant la session.

Chaque inférence est encadrée de deux lignes de journal portant l'empreinte résidente du
processus et la RAM disponible. Après un arrêt brutal, cet encadrement est la seule trace
montrant qu'une inférence était en cours. En complément, `core/logging.rs` dépose un
marqueur `candilog.session` au démarrage et le retire en sortie normale : un marqueur
survivant au lancement suivant signale une session tuée sans un mot dans le journal.

Après installation, un prompt synthétique sans donnée utilisateur mesure
chargement, tokens/s et mémoire du processus. Les paliers sont excellent (> 20), bon (10–20), acceptable (5–10) et trop lent
(< 5). Dans ce dernier cas, l'interface propose le profil inférieur mais ne le télécharge
qu'après confirmation. Un ancien modèle n'est proposé à la suppression qu'après validation
du nouveau.

Les paquets macOS ARM64 utilisent Metal ; macOS Intel retombe sur le CPU. Les paquets Linux
et Windows du workflow de release activent Vulkan, avec fallback CPU. Le feature flag
`local-ai-cuda` permet une variante CUDA bâtie sur un runner équipé du toolkit ; la release
publique courante ne produit pas encore cet artefact NVIDIA séparé.

Les commandes `detect_local_ai_hardware`, `get_local_ai_recommendation`,
`get_local_ai_status`, `install_local_ai_model`, `cancel_local_ai_download`,
`remove_local_ai_model`, `benchmark_local_ai_model` et `test_local_ai_model` restent minces.
Le téléchargement publie `local-ai://download-progress`, `local-ai://download-completed` et
`local-ai://download-error` ; aucun polling n'est utilisé.

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

Le récapitulatif et les reformulations de `AtsAnalysis` restent du texte libre borné. Les
recommandations de contenu, elles, ne transportent que des identifiants du catalogue du
profil, une justification et une pertinence qualitative. `ground_content_recommendations`
écarte les identifiants inconnus et les doublons avant l'éditeur.

La lettre de motivation est **assemblée**, pas rédigée : le modèle ne renvoie qu'une
sélection d'identifiants du catalogue de faits et des mots-clés du brief
(`domain/cover_letter.rs`). Un identifiant inconnu invalide la réponse ; un mot-clé absent
du brief est simplement écarté, parce qu'une paraphrase du modèle ne justifie pas de faire
échouer toute la rédaction — la lettre reste dans tous les cas limitée aux faits vérifiés.

La composition de la lettre est du français, pas du gabarit : la préposition est **élidée**
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

## Cache

Il n'y a pas de cache de réponses IA. La table `ai_cache`, la commande
`settings_clear_ai_cache` et le bouton « Vider le cache IA » ont été retirés : rien
n'alimentait la table, et l'écran des réglages annonçait à l'utilisateur un effet qui ne se
produisait jamais. Une base de développement antérieure conserve la table, désormais
orpheline et sans usage.
