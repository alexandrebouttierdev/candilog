//! Types échangés avec React pour les workflows IA.

use crate::features::profil::domain::Profil;
use serde::{Deserialize, Deserializer, Serialize};
use ts_rs::TS;

fn chaine_souple<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    let value = serde_json::Value::deserialize(deserializer)?;
    Ok(match value {
        serde_json::Value::String(v) => v,
        serde_json::Value::Number(v) => v.to_string(),
        serde_json::Value::Bool(v) => v.to_string(),
        serde_json::Value::Array(v) => v
            .into_iter()
            .map(|i| i.as_str().unwrap_or_default().to_owned())
            .collect::<Vec<_>>()
            .join(", "),
        serde_json::Value::Object(v) => v
            .into_values()
            .filter_map(|i| i.as_str().map(str::to_owned))
            .collect::<Vec<_>>()
            .join(" — "),
        serde_json::Value::Null => String::new(),
    })
}

fn liste_souple<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<String>, D::Error> {
    let value = serde_json::Value::deserialize(deserializer)?;
    Ok(match value {
        serde_json::Value::Array(items) => items
            .into_iter()
            .filter_map(|v| match v {
                serde_json::Value::String(s) => Some(s),
                serde_json::Value::Number(n) => Some(n.to_string()),
                _ => None,
            })
            .collect(),
        serde_json::Value::String(s) => vec![s],
        _ => Vec::new(),
    })
}

fn score_souple<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u8, D::Error> {
    let value = serde_json::Value::deserialize(deserializer)?;
    let score = match value {
        serde_json::Value::Number(n) => n.as_u64().unwrap_or_default(),
        serde_json::Value::String(s) => s.trim_end_matches('%').parse().unwrap_or_default(),
        _ => 0,
    };
    Ok(score.min(100) as u8)
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "ia.ts")]
pub struct OffreStructuree {
    #[serde(default, deserialize_with = "chaine_souple")]
    pub titre: String,
    #[serde(default, deserialize_with = "liste_souple")]
    pub competences: Vec<String>,
    #[serde(default, deserialize_with = "liste_souple")]
    pub savoir_etre: Vec<String>,
    #[serde(default)]
    pub experience: Option<String>,
    #[serde(default, deserialize_with = "liste_souple")]
    pub mots_cles: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "ia.ts")]
pub struct ScoreCorrespondance {
    pub total: u8,
    pub competences: u8,
    pub experience: u8,
    pub ats: u8,
    pub presentes: Vec<String>,
    pub absentes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "ia.ts")]
pub struct AnalyseOffre {
    pub offre: OffreStructuree,
    pub score: ScoreCorrespondance,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "ia.ts")]
pub struct ExperienceGeneree {
    #[serde(default, deserialize_with = "chaine_souple")]
    pub intitule: String,
    #[serde(default, deserialize_with = "chaine_souple")]
    pub entreprise: String,
    #[serde(default, deserialize_with = "chaine_souple")]
    pub description: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "ia.ts")]
pub struct FormationGeneree {
    #[serde(default, deserialize_with = "chaine_souple")]
    pub diplome: String,
    #[serde(default, deserialize_with = "chaine_souple")]
    pub etablissement: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "ia.ts")]
pub struct CvGenere {
    #[serde(default, deserialize_with = "chaine_souple")]
    pub resume: String,
    #[serde(default)]
    pub experiences: Vec<ExperienceGeneree>,
    #[serde(default, deserialize_with = "liste_souple")]
    pub competences: Vec<String>,
    #[serde(default)]
    pub formations: Vec<FormationGeneree>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "ia.ts")]
pub struct RecommandationAts {
    pub section: String,
    pub texte_original: String,
    pub texte_propose: String,
    #[serde(default, deserialize_with = "score_souple")]
    pub impact: u8,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "ia.ts")]
pub struct AnalyseAts {
    #[serde(default, alias = "score_ats", deserialize_with = "score_souple")]
    pub score: u8,
    #[serde(default, alias = "summary", deserialize_with = "chaine_souple")]
    pub recap: String,
    #[serde(default, deserialize_with = "liste_souple")]
    pub suggestions: Vec<String>,
    #[serde(default)]
    pub recommandations: Vec<RecommandationAts>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "ia.ts")]
pub struct GenerationCv {
    pub cv: CvGenere,
    pub analyse: AnalyseAts,
    pub offre: OffreStructuree,
    pub score_profil: ScoreCorrespondance,
}

#[derive(Debug, Clone, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "ia.ts")]
pub struct DemandeGenerationCv {
    pub generation_id: String,
    pub offre: String,
}

#[derive(Debug, Clone, Default, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "ia.ts")]
pub struct DemandeLettre {
    pub generation_id: String,
    pub entreprise: Option<String>,
    pub poste: Option<String>,
    pub ton: Option<String>,
    pub longueur: Option<String>,
    pub contexte: Option<String>,
    pub lettre_precedente: Option<String>,
    pub instruction: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "ia.ts")]
pub struct AnalyseCvImporte {
    pub cv: CvGenere,
    pub offre: OffreStructuree,
    pub score: ScoreCorrespondance,
    pub analyse: AnalyseAts,
}

#[derive(Debug, Clone, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "ia.ts")]
pub struct DemandeAnalyseCv {
    pub generation_id: String,
    pub chemin: String,
    pub offre: String,
}

#[derive(Debug, Clone, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "ia.ts")]
pub struct DemandeImportProfil {
    pub generation_id: String,
    pub chemin: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "ia.ts")]
pub struct ProfilExtrait {
    pub profil: Profil,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "ia.ts")]
pub struct ProgressionIa {
    pub generation_id: String,
    pub etape: String,
    pub progression: u8,
    pub fragment: Option<String>,
}
