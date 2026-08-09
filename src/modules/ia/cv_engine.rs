//! Moteur `CvEngine` — étape d'analyse d'offre (extraction `LLM`).

use crate::modules::ia::cv_model::{
    AtsAnalysis, DemandeLettre, GeneratedCv, LetterGenerationRequest, MatchScore, ParsedOffer,
};
use crate::modules::ia::cv_sections::split_cv;
use crate::modules::ia::mode::{ModeProfile, Step};
use crate::modules::ia::profile_extraction::{
    history_schema, identity_schema, portfolio_schema, skills_schema, ExtractedHistory,
    ExtractedPersonal, ExtractedPortfolio, ExtractedProfile, ExtractedSkillsLangs,
};
use crate::modules::ia::provider::LlmProvider;
use crate::modules::ia::{contacts, grounding};
use crate::shared::error::{AppError, AppResult};
use crate::shared::llm::AnalysisMode;
use crate::shared::profile::Profile;
use crate::shared::types::AnalyseEntretien;
use std::sync::Arc;

// ── Prompts système ──────────────────────────────────────────────────────────
// Rédigés en anglais et très structurés (rôle → règles → schéma), pour maximiser
// la fiabilité des petits modèles (1B). AUCUN exemple concret dans les prompts :
// les modèles compacts recopient les valeurs des exemples dans leurs réponses
// (constaté avec Gemma 3 1B). Le format est garanti autrement : schéma JSON natif
// (Ollama) + réparation/retry du moteur. Chaque prompt d'extraction impose de
// recopier VERBATIM dans la langue d'origine du CV — jamais de traduction. Les
// champs rédigés destinés à l'utilisateur (recap, suggestions…) sont explicitement
// demandés en français.

/// Prompt système d'extraction d'offre (→ `ParsedOffer`).
const PARSE_SYSTEM: &str = r#"You extract fields from a job offer into JSON.

RULES:
1. COPY words verbatim from the offer, in the offer's original language. NEVER translate, invent, or add anything.
2. Absent field: "" for title, [] for lists, null for experience.
3. "skills" = technical/hard skills. "soft_skills" = behavioral skills. "keywords" = the most important ATS terms of the offer.
4. One skill per array item, copied exactly as written.

OUTPUT — a single JSON object with EXACTLY these keys:
{"title":"","skills":[],"soft_skills":[],"experience":null,"keywords":[]}

Output only the JSON object. No markdown, no backticks, no comments."#;

/// Prompt système de génération de CV (reformulation → `GeneratedCv`).
const GENERATE_SYSTEM: &str = r#"You rewrite a candidate's CV to target a job offer. The input is a JSON object with the candidate "profile" and the parsed "offer".

RULES:
1. REPHRASE ONLY. Never add a skill, job, company, or fact that is not in the profile.
2. WRITE in the same language as the profile content (a French profile gives a French CV). Never translate.
3. Keep EVERY experience and education entry from the profile. Do not change title, company, degree, or school.
4. Rewrite "summary" and each experience "description" to highlight the profile elements that match the offer.
5. "skills" = the profile skills, offer-matching ones first. Do not add new skills.
6. If the profile has no summary, write ONE short sentence using only profile facts.

OUTPUT — a single JSON object with EXACTLY these keys:
{"summary":"","experiences":[{"title":"","company":"","description":""}],"skills":[],"education":[{"degree":"","school":""}]}

Output only the JSON object. No markdown, no backticks, no comments."#;

/// Prompt système d'analyse `ATS` (→ `AtsAnalysis`).
const ATS_SYSTEM: &str = r#"You compare a CV to a job offer and return an ATS analysis as JSON. The input is a JSON object {"cv":..., "offer":...}.

RULES:
1. Use ONLY the provided cv and offer. Never invent facts, skills, or companies.
2. "score": integer 0-100, how well the CV matches the offer.
3. "recap": 2-3 sentences IN FRENCH about the overall fit, main strengths, and main gap.
4. "suggestions": up to 5 short tips IN FRENCH. [] if none.
5. "recommandations": up to 5 concrete edits. For each one:
   - "section": EXACTLY "resume", "competences", or "experience_N" (N = 0-based index in cv.experiences).
   - "texte_original": the current text of that section, copied verbatim from the cv.
   - "texte_propose": an improved version targeting the offer, written in the same language as the cv (for "competences", a comma-separated list).
   - "impact": integer 1-15, estimated ATS points gained.
   [] if you have no concrete edit.

OUTPUT — a single JSON object with EXACTLY these keys:
{"score":0,"recap":"","suggestions":[],"recommandations":[{"section":"resume","texte_original":"","texte_propose":"","impact":0}]}

Output only the JSON object. No markdown, no backticks, no comments."#;

/// Prompt système de structuration d'un CV importé (→ `GeneratedCv`).
const PARSE_CV_SYSTEM: &str = r#"You convert raw CV text into JSON.

RULES:
1. COPY text verbatim from the CV, in the CV's original language (French stays French). NEVER translate, rephrase, invent, or guess.
2. Missing value: "" for strings, [] for lists.
3. One item per job in "experiences"; one item per diploma in "education". A company is an organization name written in the CV — technology names are NEVER companies.

OUTPUT — a single JSON object with EXACTLY these keys:
{"summary":"","experiences":[{"title":"","company":"","description":""}],"skills":[],"education":[{"degree":"","school":""}]}

Output only the JSON object. No markdown, no backticks, no comments."#;

/// Prompt système — extraction de l'identité d'un CV (bloc `personal`).
const EXTRACT_IDENTITY_SYSTEM: &str = r#"You extract the identity block from raw CV text into JSON.

RULES:
1. COPY values verbatim from the CV, in the CV's original language. NEVER translate, rephrase, or invent.
2. "first_name" and "last_name": split the candidate's printed name. NEVER derive them from an email address or a URL.
3. Absent field: null ("" for first_name, last_name, email). If unsure a value is really in the CV, use null.
4. "headline" = the professional title/tagline line. "summary" = the profile/about paragraph, copied word for word.
5. "email", "phone", "linkedin", "github", "website": copy the EXACT value if printed, else null. Never guess or reconstruct them — the application extracts these separately, so null is perfectly fine.

OUTPUT — a single JSON object with EXACTLY these keys:
{"first_name":"","last_name":"","email":"","phone":null,"city":null,"headline":null,"summary":null,"linkedin":null,"github":null,"website":null}

Output only the JSON object. No markdown, no backticks, no comments."#;

/// Prompt système — extraction du parcours (expériences + formations) d'un CV.
const EXTRACT_HISTORY_SYSTEM: &str = r#"You extract work experience and education from raw CV text into JSON.

RULES:
1. COPY values verbatim from the CV, in the CV's original language. NEVER translate, rephrase, or invent.
2. An experience = a job title held at an employer, as written in the CV. "company" is an organization name — technology names (frameworks, databases, tools) are NEVER companies. A list of technologies is NOT an experience.
3. "education" = diplomas/studies only. Do NOT put them in "experiences".
4. Dates: "YYYY-MM" when the month is known, else "YYYY". Unknown: null. "current": true ONLY if the CV marks the job as ongoing; then end_date = null.
5. List each real entry ONCE. Absent or unsure: [] / null.

OUTPUT — a single JSON object with EXACTLY these keys:
{"experiences":[{"title":"","company":"","location":null,"start_date":null,"end_date":null,"current":false,"description":null}],"education":[{"degree":"","school":"","location":null,"start_date":null,"end_date":null,"description":null}]}

Output only the JSON object. No markdown, no backticks, no comments."#;

/// Prompt système — extraction des compétences et langues d'un CV.
const EXTRACT_SKILLS_SYSTEM: &str = r#"You extract skills and languages from raw CV text into JSON.

RULES:
1. COPY names verbatim from the CV, in the CV's original language. NEVER translate or invent.
2. "skills" = technical/professional skills written in the CV. Name only — never append or invent a level.
3. "languages" = spoken human languages only (French, English…). Programming languages go in "skills". "level" = the stated level, else "".
4. One item per skill and per language, each listed ONCE. Absent section: [].

OUTPUT — a single JSON object with EXACTLY these keys:
{"skills":[{"name":""}],"languages":[{"name":"","level":""}]}

Output only the JSON object. No markdown, no backticks, no comments."#;

/// Prompt système — extraction des projets et certifications d'un CV.
const EXTRACT_PORTFOLIO_SYSTEM: &str = r#"You extract projects and certifications from raw CV text into JSON.

RULES:
1. COPY values verbatim from the CV, in the CV's original language. NEVER translate, rephrase, or invent.
2. "projects" = named projects/apps described in the CV. "name" = the project name alone, without its description.
3. "certifications" = named certifications/accreditations only. A diploma or degree is NOT a certification.
4. "url": the exact URL printed in the CV, else null. Absent or unsure: [] / null.

OUTPUT — a single JSON object with EXACTLY these keys:
{"projects":[{"name":"","description":null,"url":null,"technologies":null}],"certifications":[{"name":"","issuer":null,"date":null,"url":null}]}

Output only the JSON object. No markdown, no backticks, no comments."#;

/// Prompt système de rédaction/itération d'une lettre de motivation, en **streaming texte brut**.
///
/// L'entrée est un contexte `JSON` compact (profil + demande) assemblé par `build_letter_context`.
/// La sortie est le **texte de la lettre** directement (aucun `JSON`), pour un streaming
/// concaténable. Couvre les deux modes (offre / demande libre), l'absence d'offre et l'itération.
const LETTER_STREAM_SYSTEM: &str = r#"Tu es un expert en rédaction de lettres de motivation pour le marché français. À partir du contexte JSON fourni (profil du candidat + demande), tu rédiges une lettre personnalisée.

RÈGLES :
1. Base-toi UNIQUEMENT sur les informations fournies. N'invente JAMAIS une expérience, une compétence, une entreprise ni une information personnelle.
2. Les informations du profil sont PRIORITAIRES. "userInstruction" est une instruction complémentaire de l'utilisateur.
3. Longueur selon "length" : "short" ≈ 150-200 mots, "medium" ≈ 250-350 mots, "long" ≈ 400-500 mots (défaut : medium). Français impeccable, sans faute.
4. Structure : une accroche personnalisée, un corps reliant le profil au poste ou à l'entreprise, une conclusion avec un appel à l'action.
5. Respecte le ton "tone" ("formal" = formel, "casual" = décontracté, "creative" = créatif).
6. En l'absence d'offre ("jobDescription"), reste pertinent pour une candidature spontanée ou une recherche d'alternance, en t'appuyant sur "companyName", "sector" ou "jobTitle".
7. Si "previousLetter" et "instruction" sont fournis, RÉÉCRIS la lettre précédente en appliquant l'instruction, sans perdre le contenu factuel.

Réponds UNIQUEMENT par le texte de la lettre : pas de titre, pas de balise, pas de commentaire, pas de JSON."#;

/// Prompt système d'analyse d'une demande libre de lettre (→ `DemandeLettre`, `JSON` strict).
const LETTER_INTENT_SYSTEM: &str = r#"Tu analyses une demande de lettre de motivation écrite en langage libre et tu en extrais une structure JSON. N'INVENTE RIEN : laisse un champ à null si l'information n'est pas explicitement dans la demande.

RÈGLES :
1. "companyName" : l'entreprise citée, sinon null.
2. "jobTitle" : le poste ou la formation visée, sinon null.
3. "contractType" : type de contrat (alternance, professionnalisation, CDI, stage…), sinon null.
4. "applicationType" : "spontaneous" pour une candidature spontanée, "job_offer" pour une réponse à une offre, sinon null.
5. "tone" : "formal", "casual" ou "creative" si un ton est demandé, sinon null.
6. "length" : "short", "medium" ou "long" si une longueur est demandée, sinon null.
7. "keyPoints" : liste des éléments importants explicitement demandés, sinon [].

SORTIE — un unique objet JSON avec EXACTEMENT ces clés :
{"companyName":null,"jobTitle":null,"contractType":null,"applicationType":null,"tone":null,"length":null,"keyPoints":[]}

Ne renvoie que l'objet JSON. Pas de markdown, pas de backticks, pas de commentaire."#;

/// Prompt système d'analyse d'un compte rendu d'entretien (→ `AnalyseEntretien`).
const ANALYSE_ENTRETIEN_SYSTEM: &str = r#"You analyze a candidate's own interview report and return JSON. The input is the report text.

RULES:
1. Use ONLY the report. Never invent.
2. Write ALL output text IN FRENCH.
3. "resume": 2 sentences max. "points_forts", "points_faibles", "suggestions": up to 5 short, actionable items each. [] if none.

OUTPUT — a single JSON object with EXACTLY these keys:
{"resume":"","points_forts":[],"points_faibles":[],"suggestions":[]}

Output only the JSON object. No markdown, no backticks, no comments."#;

/// Moteur d'opérations `IA` sur les CV, adossé à un fournisseur `LLM`.
///
/// Le [`ModeProfile`] adapte chaque appel à la capacité du modèle : fenêtre de contexte,
/// budget de tokens, nombre de tentatives, validation d'ancrage, parallélisme.
pub struct CvEngine {
    provider: Arc<dyn LlmProvider>,
    profile: ModeProfile,
}

impl CvEngine {
    /// Construit le moteur en mode `Standard` (comportement historique par défaut).
    #[must_use]
    pub fn new(provider: Arc<dyn LlmProvider>) -> Self {
        Self {
            provider,
            profile: ModeProfile::for_mode(AnalysisMode::Standard),
        }
    }

    /// Construit le moteur avec un mode d'analyse explicite.
    #[must_use]
    pub fn with_mode(provider: Arc<dyn LlmProvider>, mode: AnalysisMode) -> Self {
        Self {
            provider,
            profile: ModeProfile::for_mode(mode),
        }
    }

    /// Analyse une offre brute et en extrait une `ParsedOffer` (via `LLM`, `JSON` strict).
    ///
    /// # Errors
    /// `AppError::Provider` si le fournisseur échoue ; `AppError::Serialization` si aucune
    /// tentative ne produit de `JSON` exploitable.
    pub async fn parse_offer(&self, raw_text: &str) -> AppResult<ParsedOffer> {
        self.generate_json_step(raw_text, PARSE_SYSTEM, None, Step::Offer)
            .await
    }

    /// Structure le texte brut d'un CV importé en `GeneratedCv` (via `LLM`, `JSON` strict).
    ///
    /// # Errors
    /// `AppError::Provider` si le fournisseur échoue ; `AppError::Serialization` si aucune
    /// tentative ne produit de `JSON` exploitable.
    pub async fn parse_cv(&self, raw_text: &str) -> AppResult<GeneratedCv> {
        self.generate_json_step(raw_text, PARSE_CV_SYSTEM, None, Step::ParseCv)
            .await
    }

    /// Extrait un profil complet et normalisé depuis le texte brut d'un CV importé.
    ///
    /// L'extraction est **découpée en 4 appels `LLM` spécialisés** (identité, parcours,
    /// compétences/langues, projets/certifications). Deux optimisations par appel :
    /// - le texte du CV est **pré-découpé en Rust** ([`split_cv`]) pour n'envoyer que le
    ///   fragment concerné (repli automatique sur le texte complet, jamais de perte) ;
    /// - la sortie est **contrainte par un schéma `JSON`** quand le fournisseur le
    ///   supporte (Ollama), garantissant le format au niveau du décodage.
    ///
    /// Les sorties sont assemblées puis normalisées (dates, niveaux de langue,
    /// dédoublonnage). Une section dont la réponse reste inexploitable après retries est
    /// traitée comme **vide** (jamais inventée) ; seules les erreurs de
    /// fournisseur/réseau interrompent.
    ///
    /// # Errors
    /// `AppError::Provider` / `AppError::Http` si le fournisseur échoue.
    pub async fn extract_profile(&self, cv_text: &str) -> AppResult<Profile> {
        let sections = split_cv(cv_text);
        // Les sections sont indépendantes : parallèles pour un gros modèle / cloud (latence
        // divisée), séquentielles en local (un petit modèle traite une génération à la fois —
        // le parallélisme provoquerait de la contention mémoire et de la mise en attente).
        // Schémas liés à des variables : leurs références doivent vivre pendant tout l'await
        // (notamment sous `try_join!`, où les temporaires seraient sinon libérés trop tôt).
        let (id_schema, hist_schema, sk_schema, pf_schema) = (
            identity_schema(),
            history_schema(),
            skills_schema(),
            portfolio_schema(),
        );
        let (personal, history, lists, portfolio) = if self.profile.parallel_sections {
            tokio::try_join!(
                self.section::<ExtractedPersonal>(
                    &sections.identity,
                    EXTRACT_IDENTITY_SYSTEM,
                    &id_schema,
                    Step::Identity
                ),
                self.section::<ExtractedHistory>(
                    &sections.history,
                    EXTRACT_HISTORY_SYSTEM,
                    &hist_schema,
                    Step::History
                ),
                self.section::<ExtractedSkillsLangs>(
                    &sections.skills,
                    EXTRACT_SKILLS_SYSTEM,
                    &sk_schema,
                    Step::Skills
                ),
                self.section::<ExtractedPortfolio>(
                    &sections.portfolio,
                    EXTRACT_PORTFOLIO_SYSTEM,
                    &pf_schema,
                    Step::Portfolio
                ),
            )?
        } else {
            let personal = self
                .section::<ExtractedPersonal>(
                    &sections.identity,
                    EXTRACT_IDENTITY_SYSTEM,
                    &id_schema,
                    Step::Identity,
                )
                .await?;
            let history = self
                .section::<ExtractedHistory>(
                    &sections.history,
                    EXTRACT_HISTORY_SYSTEM,
                    &hist_schema,
                    Step::History,
                )
                .await?;
            let lists = self
                .section::<ExtractedSkillsLangs>(
                    &sections.skills,
                    EXTRACT_SKILLS_SYSTEM,
                    &sk_schema,
                    Step::Skills,
                )
                .await?;
            let portfolio = self
                .section::<ExtractedPortfolio>(
                    &sections.portfolio,
                    EXTRACT_PORTFOLIO_SYSTEM,
                    &pf_schema,
                    Step::Portfolio,
                )
                .await?;
            (personal, history, lists, portfolio)
        };
        let mut profile: Profile =
            ExtractedProfile::from_sections(personal, history, lists, portfolio).into();

        // Coordonnées : extraites du texte source en Rust (regex, déterministe et fiable),
        // elles priment sur la sortie du `LLM` — un petit modèle déforme/invente souvent un
        // e-mail ou une URL. La valeur du modèle n'est conservée qu'à défaut de trouvaille regex.
        let found = contacts::extract_contacts(cv_text);
        overlay_contacts(&mut profile, found);

        // Validation d'ancrage : rejette les faits absents du texte source (petits modèles).
        if self.profile.grounding {
            grounding::ground_profile(&mut profile, cv_text);
        }
        Ok(profile)
    }

    /// Extrait une section de profil (sortie contrainte par `schema` si supporté), en
    /// tolérant une réponse inexploitable (→ section vide, jamais inventée) mais en
    /// propageant les erreurs de fournisseur/réseau.
    async fn section<T: serde::de::DeserializeOwned + Default>(
        &self,
        cv_text: &str,
        system: &str,
        schema: &serde_json::Value,
        step: Step,
    ) -> AppResult<T> {
        match self
            .generate_json_step(cv_text, system, Some(schema), step)
            .await
        {
            Ok(value) => Ok(value),
            Err(AppError::Serialization(_)) => Ok(T::default()),
            Err(other) => Err(other),
        }
    }

    /// Génère un CV reformulé pour l'offre (via `LLM`, `JSON` strict).
    ///
    /// # Errors
    /// `AppError::Provider` si le fournisseur échoue ; `AppError::Serialization` si aucune
    /// tentative ne produit de `JSON` exploitable.
    pub async fn generate_cv(
        &self,
        profile: &Profile,
        offer: &ParsedOffer,
        score: &MatchScore,
    ) -> AppResult<GeneratedCv> {
        let user = serde_json::json!({
            "profile": profile,
            "offer": offer,
            "matched": score.matched,
            "missing": score.missing,
        })
        .to_string();
        self.generate_json_step(&user, GENERATE_SYSTEM, None, Step::Generate)
            .await
    }

    /// Analyse la compatibilité `ATS` du CV généré avec l'offre (via `LLM`).
    ///
    /// # Errors
    /// `AppError::Provider` si le fournisseur échoue ; `AppError::Serialization` si aucune
    /// tentative ne produit de `JSON` exploitable.
    pub async fn analyze_ats(
        &self,
        cv: &GeneratedCv,
        offer: &ParsedOffer,
    ) -> AppResult<AtsAnalysis> {
        let user = serde_json::json!({ "cv": cv, "offer": offer }).to_string();
        self.generate_json_step(&user, ATS_SYSTEM, None, Step::Ats)
            .await
    }

    /// Rédige (ou retravaille) une lettre de motivation **en streaming texte brut**.
    ///
    /// `on_chunk` reçoit chaque fragment au fil de l'eau (affichage progressif) ; la méthode
    /// renvoie le texte complet accumulé. Couvre les deux modes (offre / demande libre), l'absence
    /// d'offre et l'itération : le contexte compact est assemblé par [`build_letter_context`]
    /// (profil prioritaire, jamais inventé). Créative → l'appelant ne met jamais en cache.
    ///
    /// # Errors
    /// `AppError::Provider`/`Http` si le fournisseur échoue.
    pub async fn stream_cover_letter(
        &self,
        profile: &Profile,
        req: &LetterGenerationRequest,
        on_chunk: &mut (dyn FnMut(String) + Send),
    ) -> AppResult<String> {
        let user = build_letter_context(profile, req);
        let options = self
            .profile
            .gen_options(Step::CoverLetter, user.chars().count());
        self.provider
            .stream(&user, LETTER_STREAM_SYSTEM, &options, on_chunk)
            .await
    }

    /// Analyse une demande libre et en extrait une structure ([`DemandeLettre`]) via le `LLM`.
    ///
    /// Prétraitement du mode « demande personnalisée » : permet de préremplir les champs détectés
    /// et de ne demander à l'utilisateur que ce qui manque réellement. N'invente rien.
    ///
    /// # Errors
    /// `AppError::Provider` si le fournisseur échoue ; `AppError::Serialization` si aucune
    /// tentative ne produit de `JSON` exploitable.
    pub async fn analyser_demande_lettre(&self, instruction: &str) -> AppResult<DemandeLettre> {
        let schema = letter_intent_schema();
        self.generate_json_step(
            instruction,
            LETTER_INTENT_SYSTEM,
            Some(&schema),
            Step::LetterIntent,
        )
        .await
    }

    /// Analyse le compte rendu d'un entretien (via `LLM`, `JSON` strict).
    ///
    /// # Errors
    /// `AppError::Provider` si le fournisseur échoue ; `AppError::Serialization` si aucune
    /// tentative ne produit de `JSON` exploitable.
    pub async fn analyser_entretien(&self, compte_rendu: &str) -> AppResult<AnalyseEntretien> {
        self.generate_json_step(
            compte_rendu,
            ANALYSE_ENTRETIEN_SYSTEM,
            None,
            Step::Interview,
        )
        .await
    }

    /// Appelle le fournisseur avec les options du mode pour l'étape `step`, puis désérialise
    /// sa réponse `JSON` avec retry (nombre de tentatives fixé par le mode).
    ///
    /// Le contexte (`num_ctx`) est dimensionné au texte `user` réellement envoyé et la sortie
    /// est bornée (`num_predict`/`max_tokens`) selon l'étape — sauf en mode avancé (non borné).
    /// Si `schema` est fourni, la sortie est aussi contrainte au décodage chez les fournisseurs
    /// qui le supportent (Ollama).
    async fn generate_json_step<T: serde::de::DeserializeOwned>(
        &self,
        user: &str,
        system: &str,
        schema: Option<&serde_json::Value>,
        step: Step,
    ) -> AppResult<T> {
        let options = self.profile.gen_options(step, user.chars().count());
        let mut last_err = AppError::Serialization("aucune réponse exploitable".into());
        let mut prompt = user.to_owned();
        for _ in 0..self.profile.max_attempts {
            let raw = self
                .provider
                .generate_with_options(&prompt, system, schema, &options)
                .await?;
            match parse_llm_json::<T>(&raw) {
                Ok(value) => return Ok(value),
                Err(e) => {
                    last_err = AppError::Serialization(e.to_string());
                    prompt = correction_prompt(user, &raw, &e);
                }
            }
        }
        Err(last_err)
    }
}

/// Applique au profil les coordonnées extraites en Rust : chaque champ trouvé par regex
/// **remplace** la valeur du `LLM` (autoritaire) ; un champ non trouvé laisse la valeur
/// existante intacte (le modèle a pu la lire correctement).
fn overlay_contacts(profile: &mut Profile, found: contacts::Contacts) {
    let p = &mut profile.personal;
    if let Some(email) = found.email {
        p.email = email;
    }
    if found.phone.is_some() {
        p.phone = found.phone;
    }
    if found.linkedin.is_some() {
        p.linkedin = found.linkedin;
    }
    if found.github.is_some() {
        p.github = found.github;
    }
    if found.website.is_some() {
        p.website = found.website;
    }
}

/// Insère `key: val` dans `map` uniquement si `val` est présent et non vide (contexte compact).
fn insert_opt(
    map: &mut serde_json::Map<String, serde_json::Value>,
    key: &str,
    val: Option<&String>,
) {
    if let Some(v) = val {
        if !v.trim().is_empty() {
            map.insert(key.to_string(), serde_json::Value::String(v.clone()));
        }
    }
}

/// Assemble un **contexte compact et structuré** (`JSON`) pour la génération de lettre.
///
/// N'inclut que les champs réellement renseignés : indispensable pour les petits modèles, qui ne
/// doivent pas avoir à démêler une longue demande, un profil complet et une offre en même temps.
fn build_letter_context(profile: &Profile, req: &LetterGenerationRequest) -> String {
    let mut map = serde_json::Map::new();
    map.insert(
        "profile".to_string(),
        serde_json::to_value(profile).unwrap_or_default(),
    );
    map.insert(
        "tone".to_string(),
        serde_json::Value::String(req.tone.clone().unwrap_or_else(|| "formal".to_string())),
    );
    if !req.source.is_empty() {
        map.insert(
            "source".to_string(),
            serde_json::Value::String(req.source.clone()),
        );
    }
    insert_opt(&mut map, "userInstruction", req.user_instruction.as_ref());
    insert_opt(&mut map, "companyName", req.company_name.as_ref());
    insert_opt(&mut map, "jobTitle", req.job_title.as_ref());
    insert_opt(&mut map, "contractType", req.contract_type.as_ref());
    insert_opt(&mut map, "applicationType", req.application_type.as_ref());
    insert_opt(&mut map, "jobDescription", req.job_description.as_ref());
    insert_opt(&mut map, "sector", req.sector.as_ref());
    insert_opt(&mut map, "length", req.length.as_ref());
    insert_opt(&mut map, "previousLetter", req.previous_letter.as_ref());
    insert_opt(&mut map, "instruction", req.instruction.as_ref());
    if !req.chat_history.is_empty() {
        map.insert(
            "chatHistory".to_string(),
            serde_json::to_value(&req.chat_history).unwrap_or_default(),
        );
    }
    serde_json::Value::Object(map).to_string()
}

/// Schéma `JSON` de l'analyse de demande (décodage contraint par grammaire sur Ollama).
fn letter_intent_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "companyName": {"type": ["string", "null"]},
            "jobTitle": {"type": ["string", "null"]},
            "contractType": {"type": ["string", "null"]},
            "applicationType": {"type": ["string", "null"]},
            "tone": {"type": ["string", "null"]},
            "length": {"type": ["string", "null"]},
            "keyPoints": {"type": "array", "items": {"type": "string"}}
        },
        "required": ["companyName", "jobTitle", "contractType", "applicationType", "tone", "length", "keyPoints"]
    })
}

/// Désérialise d'abord le JSON strict, puis tente une réparation locale des
/// erreurs classiques des LLM (clés sans guillemets, virgules ou crochets
/// manquants, apostrophes, JSON tronqué).
fn parse_llm_json<T: serde::de::DeserializeOwned>(raw: &str) -> Result<T, serde_json::Error> {
    let json = extract_json(raw);
    match serde_json::from_str::<T>(json) {
        Ok(value) => Ok(value),
        Err(strict_error) => match jsonrepair_rs::jsonrepair(json) {
            Ok(repaired) => serde_json::from_str::<T>(&repaired),
            Err(_) => Err(strict_error),
        },
    }
}

/// Demande explicitement au LLM de réparer sa réponse précédente. Sans ce
/// retour, certains modèles reproduisent trois fois la même virgule manquante.
/// Rédigé en anglais, comme les prompts système, pour rester clair sur un petit modèle.
fn correction_prompt(original: &str, invalid_response: &str, error: &serde_json::Error) -> String {
    format!(
        "{original}\n\nYour previous answer was not valid JSON ({error}). Fix ONLY the syntax and return \
the complete JSON object required by the schema. Output only the JSON object, no comments, no markdown.\n\n\
Invalid answer:\n{invalid_response}"
    )
}

/// Isole le premier objet `JSON` d'un texte (retire fences et prose autour).
fn extract_json(text: &str) -> &str {
    match (text.find('{'), text.rfind('}')) {
        (Some(s), Some(e)) if e >= s => &text[s..=e],
        _ => text,
    }
}

#[cfg(test)]
#[path = "tests/cv_engine/mod.rs"]
mod tests;
