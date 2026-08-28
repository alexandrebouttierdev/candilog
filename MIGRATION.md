# Mission

Je souhaite recréer complètement mon application desktop actuellement développée en **Rust + Iced** vers une nouvelle application basée sur :

- **Tauri 2**
- **Rust** pour tout le backend local
- **React**
- **TypeScript**
- **Vite**
- **Tailwind CSS**

L'application Rust/Iced actuelle doit être utilisée comme **référence fonctionnelle et source du code métier Rust existant**.

En revanche :

> **Ne migre pas l'interface Iced.**

L'interface doit être recréée entièrement en React à partir des spécifications présentes dans :

```text
SPECDESIGN/
```

Ce dossier contient notamment :

- le design system ;
- les spécifications UI/UX ;
- les templates HTML ;
- les maquettes ;
- les composants ;
- les styles ;
- les différents écrans et états.

---

# 1. Objectif principal

Créer une nouvelle version propre, moderne, maintenable et évolutive de l'application en :

```text
Tauri 2
    +
Rust backend
    +
React / TypeScript frontend
```

Je veux conserver toutes les fonctionnalités pertinentes de l'application Iced, mais repartir sur une architecture propre.

La migration ne doit surtout pas consister à traduire mécaniquement le projet actuel fichier par fichier.

Tu dois :

1. analyser le projet Rust/Iced existant ;
2. identifier le code Rust réellement réutilisable ;
3. séparer le métier du code spécifique à Iced ;
4. réutiliser et adapter le code métier existant ;
5. restructurer ce code selon la nouvelle architecture Rust ;
6. supprimer toute dépendance entre le métier et Iced ;
7. créer la couche Tauri ;
8. recréer complètement le frontend avec React ;
9. intégrer fidèlement le design contenu dans `SPECDESIGN` ;
10. reconnecter tous les écrans React aux données réelles du backend Rust.

---

# 2. Règle fondamentale concernant l'ancien projet Iced

L'ancien projet est une **source de vérité fonctionnelle et métier**, mais pas une architecture à reproduire.

Tu dois reprendre lorsque cela est pertinent :

- modèles ;
- entités ;
- enums ;
- logique métier ;
- règles de validation ;
- accès SQLite ;
- requêtes SQL ;
- services ;
- configuration ;
- import/export ;
- génération de fichiers ;
- génération PDF ;
- logique IA ;
- appels HTTP ;
- parsing ;
- sérialisation ;
- utilitaires ;
- gestion des données ;
- traitements métier.

Tu ne dois PAS reprendre :

- widgets Iced ;
- composants Iced ;
- layout Iced ;
- navigation Iced ;
- état spécifique à Iced ;
- subscriptions Iced ;
- messages Iced ;
- commandes Iced ;
- thèmes Iced ;
- styles Iced ;
- logique uniquement destinée au rendu Iced.

Tout code qui mélange actuellement :

```text
UI Iced + métier
```

doit être refactorisé afin d'extraire proprement le métier.

---

# 3. Principe de séparation générale

Je veux cette séparation :

```text
┌──────────────────────────────────────┐
│              React                   │
│                                      │
│ View                                 │
│   ↓                                  │
│ ViewModel                            │
│   ↓                                  │
│ Frontend Service                     │
└─────────────────┬────────────────────┘
                  │
             Tauri IPC
                  │
┌─────────────────▼────────────────────┐
│               Rust                   │
│                                      │
│ Command                              │
│   ↓                                  │
│ Application Service                  │
│   ↓                                  │
│ Domain                               │
│   ↓                                  │
│ Repository                           │
│   ↓                                  │
│ Infrastructure                       │
│   ↓                                  │
│ SQLite / Files / HTTP / IA / etc.    │
└──────────────────────────────────────┘
```

Le frontend ne doit jamais accéder directement à SQLite ou aux éléments sensibles du système.

---

# 4. Stack frontend obligatoire

Le frontend doit utiliser obligatoirement :

```text
React
TypeScript
Vite
Tailwind CSS
React Router
TanStack Query
Zustand
React Hook Form
Zod
```

Ne remplace pas ces technologies sans nécessité technique majeure.

---

# 5. Architecture frontend

Je veux une architecture :

> **Feature-first + MVVM**

Ne crée surtout pas une architecture globale du genre :

```text
components/
pages/
hooks/
services/
models/
```

où toutes les fonctionnalités seraient mélangées.

Chaque feature doit être autonome.

Structure cible :

```text
src/
├── app/
│   ├── router/
│   ├── providers/
│   ├── layout/
│   └── App.tsx
│
├── features/
│   ├── candidatures/
│   │   ├── model/
│   │   │   ├── entities/
│   │   │   ├── dto/
│   │   │   ├── schemas/
│   │   │   └── mappers/
│   │   │
│   │   ├── view/
│   │   │   ├── pages/
│   │   │   └── components/
│   │   │
│   │   ├── viewmodel/
│   │   │   ├── useCandidaturesViewModel.ts
│   │   │   ├── useCandidatureDetailViewModel.ts
│   │   │   └── useCandidatureFormViewModel.ts
│   │   │
│   │   ├── services/
│   │   │   └── candidature.service.ts
│   │   │
│   │   └── index.ts
│   │
│   ├── entreprises/
│   ├── cv/
│   ├── profil/
│   ├── entretiens/
│   └── parametres/
│
├── shared/
│   ├── ui/
│   ├── hooks/
│   ├── lib/
│   ├── services/
│   ├── types/
│   └── utils/
│
└── main.tsx
```

Adapte évidemment les features à celles réellement présentes dans l'application.

---

# 6. Responsabilités MVVM

## Model

Le `model` React contient :

- types frontend ;
- DTO ;
- schemas ;
- enums ;
- mappers ;
- structures utilisées par l'UI.

Il ne doit pas contenir la logique métier centrale de l'application.

---

## View

La View contient :

- JSX ;
- composants ;
- présentation ;
- affichage ;
- interactions utilisateur ;
- classes Tailwind ;
- rendu des erreurs de formulaire.

Elle doit rester aussi déclarative que possible.

Une View ne doit pas :

- appeler directement `invoke()` ;
- contenir du SQL ;
- connaître SQLite ;
- appeler directement une API IA ;
- implémenter de grosses règles métier.

---

## ViewModel

Le ViewModel gère principalement :

- orchestration UI ;
- chargement ;
- erreurs ;
- formulaires ;
- filtres ;
- tri ;
- pagination ;
- navigation ;
- modales ;
- sélection ;
- état temporaire ;
- appels aux services frontend.

Utilise des hooks React pour les ViewModels.

Exemple :

```text
View
 ↓
useCandidaturesViewModel
 ↓
candidatureService
 ↓
Tauri IPC
```

---

# 7. Services frontend

Tous les appels Tauri doivent être centralisés.

Je ne veux PAS voir :

```ts
invoke("...");
```

directement dans les composants ou dans les pages.

Créer par exemple :

```text
features/candidatures/services/candidature.service.ts
```

qui sera la seule couche frontend connaissant les commandes Tauri associées à cette feature.

Le flux standard doit être :

```text
View
 ↓
ViewModel
 ↓
Frontend Service
 ↓
Tauri IPC
```

---

# 8. Tailwind CSS obligatoire

Toute l'interface React doit être développée avec **Tailwind CSS**.

Le dossier `SPECDESIGN` reste la source de vérité graphique.

Tu dois analyser les templates HTML et le design system présents dans `SPECDESIGN`, puis les convertir proprement en composants React utilisant Tailwind CSS.

Je veux éviter :

- CSS inline ;
- fichiers CSS spécifiques dispersés ;
- duplication massive de styles ;
- valeurs arbitraires répétées ;
- mélange incohérent entre plusieurs systèmes de styling.

Configurer Tailwind à partir du design system fourni.

Les éléments suivants doivent être représentés autant que possible par des tokens cohérents :

- couleurs ;
- backgrounds ;
- couleurs de texte ;
- bordures ;
- espacements ;
- tailles ;
- radius ;
- ombres ;
- typographie ;
- transitions ;
- breakpoints ;
- états hover ;
- états focus ;
- états disabled.

Exemple :

```tsx
<button className="bg-primary text-primary-foreground">
```

plutôt que :

```tsx
<button className="bg-[#7367F0]">
```

répété partout.

Des valeurs spécifiques restent acceptables lorsqu'elles sont réellement propres à une maquette, mais elles ne doivent pas devenir la norme.

---

# 9. React Hook Form obligatoire

Tous les véritables formulaires métier doivent utiliser :

```text
React Hook Form
```

Je ne veux pas de gestion manuelle répétitive :

```tsx
const [name, setName] = useState("");
const [email, setEmail] = useState("");
const [city, setCity] = useState("");
```

pour les formulaires.

Utilise :

```tsx
useForm()
```

et les primitives adaptées de React Hook Form.

Le ViewModel peut orchestrer l'enregistrement et les actions associées au formulaire, mais la gestion technique du formulaire doit être basée sur React Hook Form.

---

# 10. Zod obligatoire pour chaque formulaire

Chaque formulaire métier doit obligatoirement posséder son propre schéma **Zod**.

Exemple :

```text
features/
└── candidatures/
    └── model/
        └── schemas/
            └── candidature-form.schema.ts
```

Exemple :

```ts
import { z } from "zod";

export const candidatureFormSchema = z.object({
  titre: z
    .string()
    .min(1, "Le titre est obligatoire"),

  entrepriseId: z
    .string()
    .min(1, "L'entreprise est obligatoire"),

  ville: z
    .string()
    .optional(),
});
```

Le type TypeScript du formulaire doit être dérivé du schéma :

```ts
export type CandidatureFormValues =
  z.infer<typeof candidatureFormSchema>;
```

Évite de maintenir manuellement un type TypeScript différent représentant exactement les mêmes données.

---

# 11. React Hook Form + Zod

Chaque formulaire doit utiliser :

```text
React Hook Form
+
Zod
+
zodResolver
```

Exemple :

```tsx
const form = useForm<CandidatureFormValues>({
  resolver: zodResolver(candidatureFormSchema),
  defaultValues: {
    titre: "",
    entrepriseId: "",
    ville: "",
  },
});
```

La validation frontend doit être centralisée dans Zod.

Je ne veux pas de validations manuelles dispersées du type :

```tsx
if (!titre) {
  setError(...);
}
```

lorsque Zod peut gérer proprement cette règle.

---

# 12. Un schéma Zod par formulaire

Règle obligatoire :

> Chaque formulaire métier possède un schéma Zod dédié.

Par exemple :

```text
features/candidatures/model/schemas/
├── create-candidature.schema.ts
├── edit-candidature.schema.ts
└── candidature-filter.schema.ts
```

Si création et modification utilisent exactement les mêmes règles, un schéma partagé peut être utilisé :

```text
candidature-form.schema.ts
```

Ne duplique pas deux schémas identiques sans raison.

---

# 13. Validation conditionnelle

Les règles conditionnelles doivent également être exprimées avec Zod lorsque cela est pertinent.

Exemple :

```text
type candidature = OFFRE
```

alors :

```text
lien de l'offre obligatoire
```

Cette règle doit exister dans le schéma.

Exemple conceptuel :

```ts
export const candidatureSchema = z
  .object({
    type: z.enum(["OFFRE", "SPONTANEE"]),
    lienOffre: z.string().optional(),
  })
  .superRefine((data, ctx) => {
    if (data.type === "OFFRE" && !data.lienOffre) {
      ctx.addIssue({
        code: "custom",
        path: ["lienOffre"],
        message: "Le lien de l'offre est obligatoire.",
      });
    }
  });
```

---

# 14. Validation frontend ≠ validation métier

Zod sert à :

- validation UX ;
- champs obligatoires ;
- email ;
- URL ;
- longueurs ;
- nombres ;
- dates ;
- cohérence simple du formulaire ;
- validations conditionnelles liées au formulaire.

Mais Rust doit également vérifier toutes les règles métier importantes.

Le flux devient :

```text
Utilisateur
    ↓
React Hook Form
    ↓
Zod
    ↓
ViewModel
    ↓
Frontend Service
    ↓
Tauri
    ↓
Rust
    ↓
Validation métier
    ↓
Repository
```

---

# 15. TanStack Query

Utiliser **TanStack Query** pour :

- chargement de données depuis Rust ;
- cache ;
- queries ;
- mutations ;
- invalidation ;
- refetch ;
- loading states ;
- error states.

Évite de recopier dans Zustand des données déjà gérées correctement par TanStack Query.

---

# 16. Zustand

Utiliser Zustand uniquement pour les états globaux frontend réellement nécessaires, par exemple :

- état global de l'interface ;
- préférences visuelles ;
- certains états de navigation ;
- état partagé non serveur.

Ne pas utiliser Zustand pour :

- remplacer React Hook Form ;
- dupliquer TanStack Query ;
- stocker toute la base de données ;
- recréer une architecture Redux inutilement complexe.

---

# 17. Architecture Rust

Je veux côté Rust une :

> **architecture hexagonale pragmatique + feature-first**

Structure cible :

```text
src-tauri/
├── Cargo.toml
├── tauri.conf.json
├── capabilities/
│
└── src/
    ├── main.rs
    ├── lib.rs
    │
    ├── app/
    │   ├── mod.rs
    │   ├── bootstrap.rs
    │   └── state.rs
    │
    ├── core/
    │   ├── mod.rs
    │   │
    │   ├── config/
    │   │   ├── mod.rs
    │   │   └── app_config.rs
    │   │
    │   ├── database/
    │   │   ├── mod.rs
    │   │   ├── connection.rs
    │   │   └── migrations.rs
    │   │
    │   ├── errors/
    │   │   ├── mod.rs
    │   │   └── app_error.rs
    │   │
    │   ├── events/
    │   │   └── mod.rs
    │   │
    │   └── utils/
    │       └── mod.rs
    │
    ├── features/
    │   │
    │   ├── candidatures/
    │   │   ├── mod.rs
    │   │   │
    │   │   ├── domain/
    │   │   │   ├── mod.rs
    │   │   │   ├── candidature.rs
    │   │   │   ├── statut.rs
    │   │   │   └── repository.rs
    │   │   │
    │   │   ├── application/
    │   │   │   ├── mod.rs
    │   │   │   ├── dto/
    │   │   │   ├── mapper.rs
    │   │   │   └── service.rs
    │   │   │
    │   │   ├── infrastructure/
    │   │   │   ├── mod.rs
    │   │   │   └── sqlite_repository.rs
    │   │   │
    │   │   └── presentation/
    │   │       ├── mod.rs
    │   │       └── commands.rs
    │   │
    │   ├── entreprises/
    │   ├── cv/
    │   ├── profil/
    │   ├── entretiens/
    │   └── parametres/
    │
    └── infrastructure/
        ├── mod.rs
        ├── ai/
        ├── filesystem/
        ├── pdf/
        ├── http/
        └── secure_storage/
```

Cette structure est une référence.

Adapte les noms des features au domaine réel de l'application.

---

# 18. Domain Rust

Le domaine doit être le plus indépendant possible.

Le `domain/` ne doit pas dépendre directement de :

- Tauri ;
- React ;
- Iced ;
- SQLx ;
- SQLite ;
- API OpenAI ;
- filesystem ;
- rendu graphique.

Le domaine contient principalement :

- entities ;
- value objects ;
- enums ;
- règles métier ;
- interfaces/traits métier.

---

# 19. Application Rust

La couche `application/` contient :

- cas d'utilisation ;
- orchestration métier ;
- services applicatifs ;
- DTO d'entrée/sortie ;
- mappers.

Exemple :

```text
Créer candidature
       ↓
charger entreprise
       ↓
appliquer règles métier
       ↓
créer candidature
       ↓
repository
       ↓
retourner DTO
```

---

# 20. Repository Pattern

Les repositories doivent être définis via des traits lorsque cela apporte une réelle séparation.

Exemple :

```rust
pub trait CandidatureRepository: Send + Sync {
    // ...
}
```

Puis :

```text
domain/repository.rs
        ↑
        │ impl
        │
infrastructure/sqlite_repository.rs
```

Les requêtes SQL doivent rester dans la couche infrastructure/repository.

Je ne veux aucun SQL dans :

- les Commands Tauri ;
- les Views React ;
- les ViewModels React.

---

# 21. Infrastructure Rust

La couche infrastructure implémente les détails techniques :

- SQLite ;
- SQL ;
- HTTP ;
- filesystem ;
- génération PDF ;
- stockage sécurisé ;
- APIs externes ;
- IA ;
- imports ;
- exports.

---

# 22. Commands Tauri

Les Commands doivent être fines.

Leur rôle est principalement :

```text
désérialiser la requête
        ↓
appeler le service
        ↓
transformer l'erreur si nécessaire
        ↓
retourner le DTO
```

Ne mets pas de logique métier importante dans les Commands.

Je ne veux pas de Commands de plusieurs centaines de lignes.

---

# 23. AppState et injection des dépendances

Centraliser les dépendances principales dans un `AppState`.

Exemple :

```rust
pub struct AppState {
    pub candidatures: Arc<CandidatureService>,
    pub entreprises: Arc<EntrepriseService>,
    pub cv: Arc<CvService>,
}
```

Construire les repositories et services au bootstrap puis les injecter dans cet état.

Éviter de recréer :

- connexions ;
- repositories ;
- clients HTTP ;
- providers IA ;

à chaque commande Tauri.

---

# 24. Base de données

Analyse précisément la manière dont l'application Iced utilise actuellement sa base de données.

Je veux préserver autant que possible :

- le schéma existant ;
- les données existantes ;
- les règles métier ;
- les requêtes pertinentes.

Si l'application actuelle utilise SQLite, conserve SQLite sauf raison technique très forte.

Pour Rust, utiliser de préférence :

```text
SQLx
```

La connexion doit être centralisée et correctement gérée.

---

# 25. DTO et entities

Ne pas exposer directement toutes les entities du domaine au frontend.

Séparer lorsque nécessaire :

```text
Domain Entity
     ↓
Mapper
     ↓
DTO
     ↓
Tauri
     ↓
React
```

Les structures envoyées à React doivent être pensées comme des contrats IPC.

Utiliser correctement :

```rust
Serialize
Deserialize
```

et une convention cohérente :

```rust
#[serde(rename_all = "camelCase")]
```

---

# 26. Types Rust ↔ TypeScript

Évite autant que possible de maintenir manuellement les mêmes contrats dans Rust et TypeScript.

Étudie l'utilisation d'une solution de génération de bindings TypeScript compatible avec Tauri 2 si elle est adaptée et correctement maintenue.

L'objectif est d'éviter :

```text
DTO Rust modifié
mais
type TypeScript oublié
```

Ne rajoute toutefois pas une dépendance non maintenue uniquement pour respecter cette consigne.

---

# 27. Gestion des erreurs Rust

Mettre en place une gestion d'erreur propre côté Rust.

Utiliser notamment :

```text
thiserror
```

si approprié.

Éviter :

- `unwrap()` non justifiés ;
- `expect()` dans le métier ;
- erreurs sous forme de chaînes dispersées ;
- erreurs SQL affichées directement à l'utilisateur.

Le frontend doit recevoir des erreurs structurées.

Exemple :

```json
{
  "code": "CANDIDATURE_NOT_FOUND",
  "message": "La candidature est introuvable."
}
```

---

# 28. Async

Utiliser correctement :

```text
Tokio
async/await
```

pour :

- base de données ;
- HTTP ;
- IA ;
- fichiers lorsque pertinent ;
- opérations longues.

Ne jamais bloquer inutilement le thread principal Tauri.

---

# 29. IA et providers externes

Si l'application actuelle gère plusieurs providers IA, créer une abstraction claire.

Exemple :

```text
AiProvider
   ├── OpenAiProvider
   ├── AnthropicProvider
   ├── MistralProvider
   ├── GeminiProvider
   └── OllamaProvider
```

Le métier ne doit pas être fortement couplé à un fournisseur.

---

# 30. Événements, Channels et progression

Pour les traitements longs ou progressifs, utiliser correctement les mécanismes Tauri adaptés.

Par exemple :

- progression import ;
- progression export ;
- génération ;
- analyse IA ;
- streaming IA ;
- téléchargement.

Ne pas créer un polling React inutile lorsqu'une communication événementielle est plus adaptée.

---

# 31. SPECDESIGN obligatoire

Le dossier :

```text
SPECDESIGN/
```

est la **source de vérité pour le design et l'UX**.

Avant de coder le frontend :

1. analyser tous les fichiers ;
2. identifier les pages ;
3. identifier les composants réutilisables ;
4. identifier les design tokens ;
5. identifier la typographie ;
6. couleurs ;
7. espacements ;
8. bordures ;
9. rayons ;
10. ombres ;
11. états hover/focus/disabled ;
12. tableaux ;
13. formulaires ;
14. modales ;
15. navigation ;
16. sidebar ;
17. icônes ;
18. états loading ;
19. états empty ;
20. états error.

---

# 32. Templates HTML

Les templates HTML présents dans `SPECDESIGN` servent de référence visuelle.

Ils ne doivent PAS être copiés tels quels dans un énorme composant React.

Tu dois les transformer proprement en :

```text
pages React
components React
layouts
design tokens
composants réutilisables
```

Exemple :

```text
HTML SPECDESIGN
        ↓
analyse
        ↓
AppShell
Sidebar
TopBar
Button
Input
Select
Card
Dialog
Table
Tabs
Badge
etc.
```

---

# 33. Design system

Implémente réellement le design system dans Tailwind.

Je ne veux pas :

```text
padding: 13px
padding: 14px
padding: 15px
```

répété arbitrairement.

Créer des tokens cohérents pour :

- couleurs ;
- backgrounds ;
- texte ;
- borders ;
- spacing ;
- radius ;
- shadows ;
- typography ;
- transitions ;
- z-index.

Le résultat React doit être visuellement fidèle aux maquettes.

---

# 34. Fidélité du design

Ne réinterprète pas arbitrairement les maquettes.

`SPECDESIGN` est prioritaire sur l'ancien rendu Iced pour l'apparence.

La règle est :

```text
Fonctionnalités et métier
        ↓
ancienne app Rust/Iced

Design et UX
        ↓
SPECDESIGN
```

En cas de différence visuelle :

> suivre `SPECDESIGN`.

En cas de fonctionnalité présente dans l'application mais absente des templates :

> l'intégrer de manière cohérente avec le design system existant.

---

# 35. Composants partagés

Créer dans :

```text
shared/ui/
```

uniquement les éléments réellement génériques :

```text
Button
Input
Select
Textarea
Checkbox
Dialog
Drawer
Tooltip
Badge
Card
Table primitives
Dropdown
Tabs
EmptyState
Loader
etc.
```

Un composant métier doit rester dans sa feature.

Exemple :

```text
CandidatureCard
```

reste dans :

```text
features/candidatures/view/components/
```

et non dans `shared/ui`.

---

# 36. Ne pas recréer le backend en TypeScript

Point extrêmement important :

React n'est pas le backend.

Je ne veux pas déplacer la logique Rust vers TypeScript simplement parce que React est introduit.

Le frontend doit principalement gérer :

```text
UI
état UI
navigation
interaction
formulaires
présentation
```

Le Rust doit rester responsable de :

```text
métier
SQLite
fichiers
imports
exports
PDF
IA
HTTP métier
sécurité
traitements lourds
```

---

# 37. Tests frontend

Tester en priorité :

- ViewModels importants ;
- comportements complexes ;
- formulaires ;
- schémas Zod ;
- composants métier importants ;
- interactions critiques.

Tester notamment que les formulaires :

- affichent correctement les erreurs ;
- empêchent les soumissions invalides ;
- respectent les validations conditionnelles ;
- appellent correctement le ViewModel/service une fois valides.

---

# 38. Tests Rust

Tester en priorité :

- services métier ;
- règles métier ;
- repositories ;
- mappers ;
- traitements ;
- parsing.

Les services doivent pouvoir être testés avec des repositories de test, mocks ou implémentations in-memory lorsque cela est pertinent.

---

# 39. Qualité du code

Je veux :

- TypeScript strict ;
- Rust idiomatique ;
- code lisible ;
- fonctions courtes ;
- responsabilités claires ;
- pas de duplication inutile ;
- pas de fichiers géants ;
- pas de `any` sans justification ;
- pas de warnings ignorés ;
- pas de dead code conservé inutilement ;
- pas de hacks temporaires laissés dans le code ;
- pas de surarchitecture inutile.

---

# 40. Commentaires et documentation

Documente ce qui mérite réellement de l'être.

Les commentaires doivent expliquer principalement :

- pourquoi une décision existe ;
- une règle métier non évidente ;
- une contrainte technique ;
- une particularité de migration.

Évite les commentaires triviaux.

---

# 41. Migration incrémentale

Ne tente pas de réécrire toute l'application aveuglément en une seule étape.

Commence par faire un inventaire.

Je veux d'abord que tu identifies :

```text
ancien code
    ↓
réutilisable tel quel
à adapter
à réécrire
à supprimer
spécifique Iced
```

Puis organise la migration feature par feature.

Pour chaque feature :

```text
1. comprendre l'implémentation Iced actuelle
2. identifier le métier Rust
3. extraire/refactoriser le métier
4. l'intégrer dans la nouvelle architecture Rust
5. exposer les Commands Tauri
6. créer les services frontend
7. créer les schemas Zod
8. créer les ViewModels
9. créer les formulaires React Hook Form
10. intégrer les Views React
11. reproduire SPECDESIGN avec Tailwind
12. connecter les vraies données
13. tester
```

---

# 42. Vérification des fonctionnalités

Avant de considérer une feature migrée, comparer son comportement avec l'application existante.

Checklist :

```text
[ ] fonctionnalité présente
[ ] règles métier conservées
[ ] données réelles correctement chargées
[ ] création fonctionnelle
[ ] modification fonctionnelle
[ ] suppression fonctionnelle
[ ] erreurs gérées
[ ] design conforme
[ ] Tailwind correctement utilisé
[ ] React Hook Form utilisé pour les formulaires
[ ] schéma Zod présent
[ ] validation Zod fonctionnelle
[ ] loading state
[ ] empty state
[ ] error state
[ ] tests utiles
```

---

# 43. Ne pas modifier inutilement l'ancien projet

Le projet Iced doit rester disponible comme référence tant que la migration n'est pas terminée.

Ne le détruis pas.

Ne commence pas par supprimer des fichiers dont tu pourrais encore avoir besoin pour comprendre le fonctionnement existant.

La nouvelle implémentation doit être clairement isolée.

---

# 44. Sécurité Tauri

Configurer proprement les capabilities Tauri 2.

Appliquer le principe :

> autoriser uniquement ce qui est réellement nécessaire.

Ne donne pas au frontend des permissions filesystem ou système larges simplement par facilité.

Les opérations sensibles doivent passer par des Commands Rust contrôlées lorsque cela est pertinent.

---

# 45. Dépendances

Avant d'ajouter une dépendance :

1. vérifier qu'elle est nécessaire ;
2. vérifier qu'elle est maintenue ;
3. vérifier sa compatibilité avec les versions actuelles ;
4. préférer une dépendance reconnue à un package obscur.

Évite l'accumulation de bibliothèques.

---

# 46. Stack Rust privilégiée

Lorsque adaptée :

```text
Tauri 2
Tokio
Serde
thiserror
tracing
SQLx
SQLite
reqwest
uuid
chrono
```

Ajouter d'autres bibliothèques uniquement selon les besoins réels.

---

# 47. Correspondance frontend/backend

Je veux que l'organisation soit facile à comprendre.

Exemple :

```text
React
features/candidatures/

            ↕

Rust
features/candidatures/
```

React :

```text
features/candidatures/
├── model/
├── view/
├── viewmodel/
└── services/
```

Rust :

```text
features/candidatures/
├── presentation/
├── application/
├── domain/
└── infrastructure/
```

Cela doit permettre de retrouver rapidement tout le code concernant une fonctionnalité.

---

# 48. Flux final attendu

Le flux standard doit être :

```text
React View
     ↓
React Hook Form
     ↓
Zod
     ↓
React ViewModel
     ↓
Frontend Service
     ↓
Tauri invoke
     ↓
Rust Command
     ↓
Application Service
     ↓
Domain
     ↓
Repository trait
     ↓
Repository implementation
     ↓
SQLite / API / filesystem
```

Pour les écrans sans formulaire :

```text
React View
     ↓
React ViewModel
     ↓
TanStack Query
     ↓
Frontend Service
     ↓
Tauri IPC
     ↓
Rust
```

---

# 49. Règles de dépendances

Respecter autant que possible :

```text
Rust

Presentation
     ↓
Application
     ↓
Domain
     ↑
Infrastructure
```

Le `Domain` ne doit pas dépendre de l'infrastructure.

Et côté React :

```text
View
 ↓
ViewModel
 ↓
Service
```

Les composants ne doivent pas contourner cette architecture.

---

# 50. Priorités

Les priorités sont dans cet ordre :

1. conservation fonctionnelle ;
2. conservation correcte du métier Rust existant ;
3. architecture propre ;
4. fidélité à `SPECDESIGN` ;
5. maintenabilité ;
6. qualité du code ;
7. testabilité ;
8. performances ;
9. simplicité.

Ne sacrifie pas la simplicité pour appliquer artificiellement un pattern.

---

# 51. Première étape obligatoire : audit

Avant de commencer massivement à coder :

## Analyser l'ancien projet

Produis une cartographie des modules actuels :

```text
module
responsabilité
dépendance Iced ?
métier réutilisable ?
accès données ?
destination dans la nouvelle architecture
```

## Analyser SPECDESIGN

Identifier :

```text
design system
pages
layouts
composants
états
templates HTML
tokens
navigation
formulaires
```

## Identifier les formulaires

Pour chaque formulaire présent dans les maquettes ou dans l'application existante, identifier :

```text
nom du formulaire
feature
champs
règles de validation
schéma Zod à créer
DTO Rust correspondant
Command Tauri correspondante
```

## Établir le mapping

Exemple :

```text
Ancien Rust/Iced
      ↓
Nouvelle feature Rust
      ↓
Commands Tauri
      ↓
Service React
      ↓
ViewModel
      ↓
Formulaire RHF + Zod
      ↓
View React
      ↓
Template SPECDESIGN
```

Ensuite seulement, commencer la migration.

---

# 52. Ne pas produire une simple démo

Je ne veux pas :

- une coquille React avec de fausses données ;
- un frontend statique ;
- un prototype visuel ;
- des boutons non connectés ;
- des TODO partout ;
- des mocks définitifs ;
- une réécriture du métier en TypeScript ;
- des formulaires sans React Hook Form ;
- des formulaires sans Zod ;
- du CSS bricolé hors Tailwind.

Je veux progressivement obtenir une **véritable migration fonctionnelle de l'application existante**.

Les données affichées dans les écrans finaux doivent venir du backend Rust réel.

---

# 53. Architecture finale attendue

Le résultat final doit être :

```text
Tauri 2
│
├── React / TypeScript
│   │
│   ├── Feature-first
│   ├── MVVM
│   ├── Tailwind CSS
│   ├── React Hook Form
│   ├── Zod
│   ├── TanStack Query
│   └── Zustand
│
└── Rust
    │
    ├── Feature-first
    ├── architecture hexagonale pragmatique
    ├── Commands Tauri
    ├── Application Services
    ├── Domain
    ├── Repositories
    └── Infrastructure
```

avec :

```text
Ancienne app Rust/Iced
        │
        └── source métier et fonctionnelle

SPECDESIGN
        │
        └── source de vérité UI / UX / design system
```

---

# 54. Règle finale

La migration doit respecter strictement cette séparation :

```text
ANCIEN PROJET RUST/ICED
→ comprendre le métier
→ récupérer le Rust pertinent
→ supprimer les dépendances Iced
→ refactoriser dans la nouvelle architecture

SPECDESIGN
→ comprendre le design
→ reprendre les templates HTML
→ transformer en composants React
→ utiliser Tailwind CSS
→ respecter le design system

FRONTEND
→ React + TypeScript
→ Feature-first + MVVM
→ React Hook Form pour chaque formulaire
→ Zod pour chaque formulaire
→ TanStack Query pour les données serveur/locales
→ Zustand uniquement pour l'état global UI pertinent

BACKEND
→ Rust
→ Feature-first
→ architecture hexagonale pragmatique
→ Tauri Commands fines
→ métier dans Domain/Application
→ SQL dans les repositories/infrastructure
```

Le résultat doit être une vraie application desktop Tauri moderne, maintenable, testable et fidèle à l'application existante sur le plan fonctionnel, tout en utilisant `SPECDESIGN` comme référence absolue pour l'interface.