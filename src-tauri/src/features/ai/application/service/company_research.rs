//! Compléter une fiche entreprise : établissements de l'annuaire, site officiel, classement.
//!
//! Deux étapes, chacune annulable par `ai_cancel` : `search_companies` liste les
//! établissements (sans appel au modèle) pour que l'utilisateur choisisse le bon ;
//! `research_company` complète celui qu'il a choisi. Une source indisponible n'arrête pas
//! la recherche : le résultat est alors partiel, et `warnings` dit ce qui manque.

use super::*;
use crate::features::referentials::domain::ReferentialRepository;

/// Sources et référentiels de la recherche d'entreprise, injectés par `app/state.rs`.
pub struct CompanyResearchDeps {
    pub sources: CompanySources,
    pub referentials: Arc<dyn ReferentialRepository>,
}

/// URL candidates au site officiel présentées au modèle, au plus.
const MAX_SITES_CANDIDATS: usize = 6;
/// Résultats Web transmis au modèle, au plus.
const MAX_RESULTATS_POUR_IA: usize = 6;

const ANNUAIRE: &str = "L'annuaire des entreprises";

/// Échec bloquant d'une source, en message pour l'utilisateur.
fn echec_source(source: &str, error: SourceError) -> AppError {
    match error {
        SourceError::RateLimited(Some(secondes)) => AppError::Provider(format!(
            "{source} limite le nombre de recherches. Réessayez dans {secondes} s."
        )),
        SourceError::RateLimited(None) => AppError::Provider(format!(
            "{source} limite le nombre de recherches. Réessayez dans un instant."
        )),
        // `Http` : l'interface y reconnaît une erreur réseau (« Vérifiez votre réseau »).
        SourceError::Unavailable => AppError::Http(format!("{source} est injoignable")),
        SourceError::Invalid(detail) => {
            tracing::warn!(%detail, source, "réponse inattendue d'une source de recherche");
            AppError::Provider(format!("{source} a renvoyé une réponse inattendue."))
        }
    }
}

/// Échec non bloquant d'une source : la recherche continue, le résultat le signale.
fn avertissement(source: &str, error: &SourceError) -> String {
    match error {
        SourceError::RateLimited(_) => {
            format!("{source} limite le nombre de recherches : il n'a pas été consulté.")
        }
        SourceError::Unavailable => format!("{source} est injoignable : il n'a pas été consulté."),
        SourceError::Invalid(detail) => {
            tracing::info!(%detail, source, "source de recherche écartée");
            format!("{source} n'a pas donné de réponse exploitable.")
        }
    }
}

fn champ(
    value: impl Into<String>,
    origin: FieldOrigin,
    source_url: Option<&str>,
    note: Option<String>,
) -> ResearchedField {
    ResearchedField {
        value: Some(value.into()),
        origin: Some(origin),
        source_url: source_url.map(str::to_owned),
        note,
    }
}

/// Champs que l'annuaire établit à lui seul, sans interprétation.
fn champs_de_l_annuaire(candidat: &CompanyCandidate) -> CompanyResearchFields {
    let source = Some(candidat.source_url.as_str());
    let lieu = if candidat.head_office {
        "Siège social de l'entreprise".to_owned()
    } else {
        format!(
            "Établissement local{}",
            candidat
                .city
                .as_deref()
                .map(|c| format!(" de {c}"))
                .unwrap_or_default()
        )
    };
    let activite = candidat.activity_code.as_deref().map(|code| {
        format!(
            "D'après le code d'activité {code}{}",
            candidat
                .activity_label
                .as_deref()
                .map(|label| format!(" ({label})"))
                .unwrap_or_default()
        )
    });
    let mut champs = CompanyResearchFields {
        name: champ(
            candidat.name.clone(),
            FieldOrigin::Registry,
            source,
            candidat
                .trade_name
                .as_deref()
                .map(|enseigne| format!("Enseigne de l'établissement : {enseigne}")),
        ),
        ..CompanyResearchFields::default()
    };
    if let Some(city) = &candidat.city {
        champs.city = champ(
            city.clone(),
            FieldOrigin::Registry,
            source,
            Some(lieu.clone()),
        );
    }
    if let Some(address) = &candidat.address {
        champs.address = champ(address.clone(), FieldOrigin::Registry, source, Some(lieu));
    }
    if let Some(size) = size_from_registry(
        candidat.size_category.as_deref(),
        candidat.workforce_range.as_deref(),
        candidat.legal_category.as_deref(),
    ) {
        let note = candidat.size_category.as_deref().map_or_else(
            || "D'après la tranche d'effectif de l'INSEE".to_owned(),
            |categorie| format!("Catégorie d'entreprise de l'INSEE : {categorie}"),
        );
        champs.company_size = champ(size_code(size), FieldOrigin::Registry, source, Some(note));
    }
    if let Some(sector) = candidat
        .activity_code
        .as_deref()
        .and_then(sector_from_activity)
    {
        champs.sector_id = champ(sector, FieldOrigin::Registry, source, activite.clone());
    }
    if let Some(company_type) = company_type_from_registry(
        candidat.legal_category.as_deref(),
        candidat.activity_code.as_deref(),
    ) {
        let note = if company_type_from_registry(candidat.legal_category.as_deref(), None).is_some()
        {
            Some("D'après la nature juridique de l'entreprise".to_owned())
        } else {
            activite
        };
        champs.company_type_id = champ(company_type, FieldOrigin::Registry, source, note);
    }
    champs
}

/// Demande envoyée au modèle : faits vérifiés, listes fermées des seuls champs à
/// déterminer, puis contenus Web encadrés comme données non fiables.
fn invite_classement(
    nom: &str,
    candidat: Option<&CompanyCandidate>,
    secteurs: Option<&[(String, String)]>,
    types: Option<&[(String, String)]>,
    sites: &[String],
    page: Option<&PageSummary>,
    resultats: &[WebResult],
) -> String {
    let mut invite = format!("Entreprise : {nom}\n");
    if let Some(candidat) = candidat {
        if let Some(code) = &candidat.activity_code {
            invite.push_str(&format!(
                "Code d'activité (NAF) : {code}{}\n",
                candidat
                    .activity_label
                    .as_deref()
                    .map(|label| format!(" — {label}"))
                    .unwrap_or_default()
            ));
        }
        if let Some(city) = &candidat.city {
            invite.push_str(&format!("Ville de l'établissement : {city}\n"));
        }
    }
    if let Some(secteurs) = secteurs {
        invite.push_str("\nSecteurs possibles :\n");
        for (index, (_, nom)) in secteurs.iter().enumerate() {
            invite.push_str(&format!("{} : {nom}\n", sector_code(index)));
        }
    }
    if let Some(types) = types {
        invite.push_str("\nTypes possibles :\n");
        for (code, nom) in types {
            invite.push_str(&format!("{code} : {nom}\n"));
        }
    }
    if !sites.is_empty() {
        invite.push_str("\nSites candidats :\n");
        for (index, site) in sites.iter().enumerate() {
            invite.push_str(&format!("{} : {site}\n", url_code(index)));
        }
    }
    if let Some(page) = page {
        let texte = [page.title.as_deref(), page.description.as_deref()]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join("\n");
        if !texte.is_empty() {
            invite.push('\n');
            invite.push_str(&bloc_donnees("site_officiel", &texte));
            invite.push('\n');
        }
    }
    if !resultats.is_empty() {
        let lignes = resultats
            .iter()
            .take(MAX_RESULTATS_POUR_IA)
            .map(|resultat| {
                format!(
                    "- {} — {} ({})",
                    resultat.title, resultat.snippet, resultat.url
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        invite.push('\n');
        invite.push_str(&bloc_donnees("resultats_web", &lignes));
        invite.push('\n');
    }
    invite.push_str(
        "\nRéponds avec {\"secteur\":null,\"type\":null,\"site\":null} ; laisse null ce qui n'a pas de liste.",
    );
    invite
}

/// Origines distinctes des résultats Web qui peuvent être un site d'entreprise.
fn sites_candidats(resultats: &[WebResult]) -> Vec<String> {
    let mut sites: Vec<String> = Vec::new();
    for resultat in resultats {
        if is_directory_site(&resultat.url) {
            continue;
        }
        if let Some(origin) = website_origin(&resultat.url) {
            if !sites.contains(&origin) {
                sites.push(origin);
            }
        }
        if sites.len() == MAX_SITES_CANDIDATS {
            break;
        }
    }
    sites
}

impl AiService {
    /// Établissements de l'annuaire qui correspondent au nom (et au lieu) recherchés.
    ///
    /// # Errors
    /// `Validation` sur une entrée invalide, `Http` si l'annuaire est injoignable,
    /// `Provider` s'il limite le débit ou répond de travers, `Cancelled` sur `ai_cancel`.
    pub async fn search_companies(
        &self,
        request: CompanySearchRequest,
        notifier: impl Fn(AiProgress),
    ) -> AppResult<AiExecution<CompanySearchResult>> {
        let started_at = std::time::Instant::now();
        let name = validate_company_query(&request.name)?;
        let location = validate_location(request.location.as_deref())?;
        let id = request.generation_id.clone();
        let token = self.start(&id);
        let _guard = GenerationEnCours {
            service: self,
            id: id.clone(),
            token: Arc::clone(&token),
        };
        progres(
            &notifier,
            &id,
            "Recherche dans l'annuaire des entreprises",
            None,
            None,
        );
        let requete = location
            .as_deref()
            .map_or_else(|| name.clone(), |lieu| format!("{name} {lieu}"));
        let registry = Arc::clone(&self.company_research.sources.registry);
        let candidats = cancel(&token, async move {
            registry
                .search(&requete)
                .await
                .map_err(|error| echec_source(ANNUAIRE, error))
        })
        .await?;
        Ok(execution(
            started_at,
            rank_candidates(&name, location.as_deref(), candidats),
            None,
        ))
    }

    /// Complète l'établissement choisi — ou, sans SIRET, l'entreprise d'après le Web seul.
    ///
    /// # Errors
    /// `Validation` sur une entrée invalide, `NotFound` si l'établissement a disparu de
    /// l'annuaire, erreurs de l'annuaire comme `search_companies`, `Cancelled`. Les autres
    /// sources et l'IA ne font jamais échouer : leur absence devient un avertissement.
    pub async fn research_company(
        &self,
        request: CompanyResearchRequest,
        notifier: impl Fn(AiProgress),
    ) -> AppResult<AiExecution<CompanyResearchResult>> {
        let started_at = std::time::Instant::now();
        let name = validate_company_query(&request.name)?;
        let siret = request
            .siret
            .as_deref()
            .map(validate_siret)
            .transpose()?
            .map(str::to_owned);
        let id = request.generation_id.clone();
        let token = self.start(&id);
        let _guard = GenerationEnCours {
            service: self,
            id: id.clone(),
            token: Arc::clone(&token),
        };
        let sources_de_recherche = &self.company_research.sources;
        let mut warnings = Vec::new();
        let mut sources = Vec::new();

        // 1. L'établissement, relu dans l'annuaire : rien de ce que l'interface a affiché
        // n'est repris tel quel.
        let candidat = match siret {
            Some(siret) => {
                progres(
                    &notifier,
                    &id,
                    "Lecture de l'établissement dans l'annuaire",
                    None,
                    None,
                );
                let registry = Arc::clone(&sources_de_recherche.registry);
                let trouve = cancel(&token, async move {
                    registry
                        .establishment(&siret)
                        .await
                        .map_err(|error| echec_source(ANNUAIRE, error))
                })
                .await?;
                Some(trouve.ok_or_else(|| {
                    AppError::NotFound("cet établissement dans l'annuaire des entreprises".into())
                })?)
            }
            None => None,
        };
        let mut fields = candidat
            .as_ref()
            .map(champs_de_l_annuaire)
            .unwrap_or_default();
        if let Some(candidat) = &candidat {
            sources.push(ResearchSource {
                label: "Annuaire des entreprises (data.gouv.fr)".into(),
                url: candidat.source_url.clone(),
            });
        }

        // 2. Site officiel déclaré sur Wikidata, rapproché par le SIREN.
        if let Some(candidat) = &candidat {
            progres(&notifier, &id, "Recherche du site officiel", None, None);
            let knowledge = Arc::clone(&sources_de_recherche.knowledge);
            let siren = candidat.siren.clone();
            match cancel(&token, async move {
                Ok::<_, AppError>(knowledge.official_website(&siren).await)
            })
            .await?
            {
                Ok(Some((site, entite))) => {
                    if let Some(origin) = website_origin(&site).filter(|o| !is_directory_site(o)) {
                        fields.website = champ(
                            origin,
                            FieldOrigin::Wikidata,
                            Some(&entite),
                            Some("Site officiel déclaré sur Wikidata pour ce SIREN".into()),
                        );
                        sources.push(ResearchSource {
                            label: "Wikidata".into(),
                            url: entite,
                        });
                    }
                }
                Ok(None) => {}
                Err(error) => warnings.push(avertissement("Wikidata", &error)),
            }
        }

        // 3. Recherche Web : contexte pour le classement et, à défaut, site officiel.
        progres(&notifier, &id, "Recherche Web", None, None);
        let requete = candidat
            .as_ref()
            .and_then(|c| c.city.as_deref())
            .map_or_else(|| name.clone(), |city| format!("{name} {city}"));
        let preferee = load_searxng_url(&self.pool).unwrap_or_default();
        let web = Arc::clone(&sources_de_recherche.web);
        let resultats = match cancel(&token, async move {
            Ok::<_, AppError>(web.search(&requete, preferee.as_deref()).await)
        })
        .await?
        {
            Ok(resultats) => resultats,
            Err(error) => {
                warnings.push(avertissement("La recherche Web (SearXNG)", &error));
                Vec::new()
            }
        };

        // 4. Page d'accueil du site officiel connu, pour le classement.
        let page = match fields.website.value.clone() {
            Some(site) => {
                progres(&notifier, &id, "Lecture du site officiel", None, None);
                let pages = Arc::clone(&sources_de_recherche.pages);
                match cancel(&token, async move {
                    Ok::<_, AppError>(pages.summary(&site).await)
                })
                .await?
                {
                    Ok(page) => Some(page),
                    Err(error) => {
                        warnings.push(avertissement("Le site officiel", &error));
                        None
                    }
                }
            }
            None => None,
        };

        // 5. Référentiels : une valeur déterministe absente du référentiel est écartée.
        let referentiels = self.company_research.referentials.load()?;
        let secteurs: Vec<(String, String)> = referentiels
            .sectors
            .iter()
            .map(|secteur| (secteur.id.to_string(), secteur.name.clone()))
            .collect();
        let types: Vec<(String, String)> = referentiels
            .company_types
            .iter()
            .map(|item| (item.code.clone(), item.name.clone()))
            .collect();
        if fields
            .sector_id
            .value
            .as_ref()
            .is_some_and(|value| !secteurs.iter().any(|(id, _)| id == value))
        {
            fields.sector_id = ResearchedField::default();
        }
        if fields
            .company_type_id
            .value
            .as_ref()
            .is_some_and(|value| !types.iter().any(|(code, _)| code == value))
        {
            fields.company_type_id = ResearchedField::default();
        }

        // 6. Le modèle ne complète que ce qui manque, dans des listes fermées.
        let sites = if fields.website.value.is_none() {
            sites_candidats(&resultats)
        } else {
            Vec::new()
        };
        let besoin_secteur = fields.sector_id.value.is_none();
        let besoin_type = fields.company_type_id.value.is_none();
        let mut ai_used = false;
        let mut tokens_used = None;
        if besoin_secteur || besoin_type || !sites.is_empty() {
            progres(&notifier, &id, "Analyse par l'IA", None, None);
            match self.provider_for(AiTask::ResearchCompany).await {
                Ok(provider) => {
                    let invite = invite_classement(
                        &name,
                        candidat.as_ref(),
                        besoin_secteur.then_some(secteurs.as_slice()),
                        besoin_type.then_some(types.as_slice()),
                        &sites,
                        page.as_ref(),
                        &resultats,
                    );
                    let appel = generate_json::<CompanyClassification>(
                        provider,
                        &invite,
                        COMPANY_RESEARCH_SYSTEM,
                    );
                    match cancel(&token, async move { Ok::<_, AppError>(appel.await) }).await? {
                        Ok((classement, tokens)) => {
                            ai_used = true;
                            tokens_used = tokens;
                            let ids: Vec<String> =
                                secteurs.iter().map(|(id, _)| id.clone()).collect();
                            let codes: Vec<String> =
                                types.iter().map(|(code, _)| code.clone()).collect();
                            let retenu = ground_classification(&classement, &ids, &codes, &sites);
                            let note = Some("Proposé par l'IA d'après le site et les résultats de recherche — à vérifier".to_owned());
                            if besoin_secteur {
                                if let Some(sector) = retenu.sector_id {
                                    fields.sector_id =
                                        champ(sector, FieldOrigin::Ai, None, note.clone());
                                }
                            }
                            if besoin_type {
                                if let Some(code) = retenu.company_type {
                                    fields.company_type_id =
                                        champ(code, FieldOrigin::Ai, None, note.clone());
                                }
                            }
                            if let Some(site) = retenu.website {
                                let preuve = resultats
                                    .iter()
                                    .find(|resultat| {
                                        website_origin(&resultat.url).as_deref()
                                            == Some(site.as_str())
                                    })
                                    .map(|resultat| resultat.url.clone());
                                if let Some(url) = &preuve {
                                    sources.push(ResearchSource {
                                        label: "Résultat de recherche Web".into(),
                                        url: url.clone(),
                                    });
                                }
                                fields.website = champ(
                                    site,
                                    FieldOrigin::Ai,
                                    preuve.as_deref(),
                                    Some("Site désigné par l'IA parmi les résultats de recherche — à vérifier".into()),
                                );
                            }
                        }
                        Err(error) => warnings.push(format!(
                            "L'IA n'a pas pu compléter le classement : {}",
                            error.user_message()
                        )),
                    }
                }
                Err(error) => warnings.push(format!("IA indisponible : {}", error.user_message())),
            }
        }
        if let Some(page) = &page {
            sources.push(ResearchSource {
                label: page.title.clone().unwrap_or_else(|| "Site officiel".into()),
                url: page.url.clone(),
            });
        }

        Ok(execution(
            started_at,
            CompanyResearchResult {
                fields,
                sources,
                warnings,
                ai_used,
            },
            tokens_used,
        ))
    }
}

#[cfg(test)]
#[path = "company_research_tests.rs"]
mod tests;
