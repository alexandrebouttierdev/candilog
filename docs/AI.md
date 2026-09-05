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

Chaque appel HTTP est **repris jusqu'à trois fois** sur un échec transitoire — délai
dépassé, connexion impossible, `429`, `5xx` — avec une attente de 1 s puis 2 s. Une
génération de CV enchaîne trois appels et dure une à deux minutes : sans reprise, un
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
la prose : le corps reste assemblé par Candilog.

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

Le LLM n'est appelé que pendant la génération pour extraire l'offre, adapter le socle et
classer sémantiquement les candidates. Présence dans le CV, filtrage, score, décisions,
simulation et place disponible sont calculés localement. Une interaction dans l'éditeur ne
déclenche donc aucun nouvel appel DeepSeek.

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
(`build_proposals`, `recalculate`) — jamais repris du LLM. Une proposition non applicable
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
