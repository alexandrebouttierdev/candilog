//! Chaîne complète de génération de documents, de bout en bout.
//!
//! Le scénario traverse la vraie logique de Candilog — `AiService`, `prepare_workspace`,
//! `ResumePdf`, `CoverLetterPdf` — pour dix profils fictifs et une offre réelle, puis
//! dépose ses artefacts (profil source, génération, poste de travail, PDF) dans un dossier
//! que les contrôles Playwright et PDF relisent ensuite.
//!
//! Deux modes, parce qu'une suite de tests ne doit pas dépendre d'un appel payant :
//!
//! - **rejeu** (défaut) : la génération enregistrée dans `generation.json` / `letter.json`
//!   est relue, seuls la composition du document et l'export PDF sont rejoués. Déterministe,
//!   sans réseau : c'est le mode qui doit tourner après chaque correction de gabarit.
//! - **live** (`CANDILOG_E2E_LIVE=1`) : la génération est réellement demandée au fournisseur
//!   IA configuré, puis enregistrée pour les rejeux suivants.
//!
//! Variables d'environnement :
//!
//! | Nom | Rôle | Défaut |
//! | --- | --- | --- |
//! | `CANDILOG_E2E` | active le scénario | absent → ignoré |
//! | `CANDILOG_E2E_LIVE` | appelle réellement le fournisseur IA | absent → rejeu |
//! | `CANDILOG_E2E_OFFER` | fichier de l'offre | requis en live |
//! | `CANDILOG_E2E_SETTINGS_DB` | base dont on copie les réglages IA | base de développement |
//! | `CANDILOG_E2E_OUT` | dossier des artefacts | `<repo>/test-output` |
//! | `CANDILOG_E2E_ONLY` | liste de profils à traiter (`01,07`) | tous |

use candilog_lib::core::database::helpers::connection;
use candilog_lib::core::database::{open_pool, run_local_migrations, SqlitePool};
use candilog_lib::features::ai::application::AiService;
use candilog_lib::features::ai::domain::{
    CoverLetterRequest, ResumeGeneration, ResumeGenerationRequest,
};
use candilog_lib::features::documents::application::{
    build, build_cover_letter, prepare_workspace,
};
use candilog_lib::features::documents::domain::CoverLetterExport;
use candilog_lib::features::profile::domain::{Profile, ProfileRepository};
use candilog_lib::features::profile::infrastructure::SqliteProfileRepository;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Poste et entreprise de l'offre de référence, passés à la lettre comme le fait l'écran.
const JOB_TITLE: &str = "Administrateur Système et Réseau";
const COMPANY: &str = "Astek";

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn out_dir() -> PathBuf {
    std::env::var_os("CANDILOG_E2E_OUT")
        .map(PathBuf::from)
        .unwrap_or_else(|| repo_root().join("test-output"))
}

fn settings_source() -> PathBuf {
    std::env::var_os("CANDILOG_E2E_SETTINGS_DB")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join(".candilog-dev")
                .join("candilog.sqlite")
        })
}

fn live() -> bool {
    std::env::var("CANDILOG_E2E_LIVE").is_ok_and(|value| value == "1")
}

/// Prépare une base isolée : schéma neuf, réglages IA recopiés, profil du cas de test.
///
/// La base vit en mémoire — le scénario ne touche jamais les données de l'utilisateur.
fn seeded_pool(profile: &Profile) -> SqlitePool {
    let pool = open_pool(None).expect("base de test");
    run_local_migrations(&pool).expect("migrations");

    let source = settings_source();
    let reglages: Option<String> = open_pool(Some(&source)).ok().and_then(|source_pool| {
        connection(&source_pool)
            .ok()?
            .query_row("SELECT data FROM settings WHERE id = 1", [], |row| {
                row.get(0)
            })
            .ok()
    });
    if let Some(reglages) = reglages {
        connection(&pool)
            .expect("connexion")
            .execute(
                "INSERT INTO settings (id, data, updated_at) VALUES (1, ?1, datetime('now'))
                 ON CONFLICT(id) DO UPDATE SET data = excluded.data",
                [reglages.as_str()],
            )
            .expect("réglages IA");
    }
    SqliteProfileRepository::new(pool.clone())
        .save(profile)
        .expect("profil de test");
    pool
}

/// Un cas de test : le profil source et le dossier où déposer ses artefacts.
struct Cas {
    slug: String,
    profile: Profile,
    dossier: PathBuf,
    /// Comportement attendu de l'export, quand il n'est pas la réussite.
    attendu: Option<String>,
}

/// Attente déclarée à côté d'un profil, pour les cas dont le résultat correct est un refus.
#[derive(serde::Deserialize)]
struct Attente {
    cv_pdf: String,
}

fn cas_a_traiter() -> Vec<Cas> {
    let filtre: Option<Vec<String>> = std::env::var("CANDILOG_E2E_ONLY").ok().map(|value| {
        value
            .split(',')
            .map(|part| part.trim().to_owned())
            .filter(|part| !part.is_empty())
            .collect()
    });
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("profiles");
    let mut entrees: Vec<_> = std::fs::read_dir(&fixtures)
        .expect("fixtures de profils")
        .filter_map(Result::ok)
        .map(|entree| entree.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        // Les attentes (`profile-10.expected.json`) accompagnent un profil, elles n'en
        // sont pas un.
        .filter(|path| {
            !path
                .file_stem()
                .and_then(|nom| nom.to_str())
                .is_some_and(|nom| nom.ends_with(".expected"))
        })
        .collect();
    entrees.sort();
    entrees
        .into_iter()
        .filter_map(|path| {
            let nom = path.file_stem()?.to_str()?.to_owned();
            let numero = nom.strip_prefix("profile-")?.to_owned();
            if filtre
                .as_ref()
                .is_some_and(|filtre| !filtre.contains(&numero))
            {
                return None;
            }
            let brut = std::fs::read_to_string(&path).ok()?;
            let profile: Profile = serde_json::from_str(&brut)
                .unwrap_or_else(|error| panic!("profil {nom} illisible : {error}"));
            let attendu =
                std::fs::read_to_string(path.with_file_name(format!("{nom}.expected.json")))
                    .ok()
                    .and_then(|brut| serde_json::from_str::<Attente>(&brut).ok())
                    .map(|attente| attente.cv_pdf);
            Some(Cas {
                dossier: out_dir().join(&nom),
                slug: nom,
                profile,
                attendu,
            })
        })
        .collect()
}

/// Efface les artefacts dérivés d'un cas avant de le rejouer.
///
/// Seule la génération enregistrée (`generation.json`, `cover-letter.txt`) survit : c'est
/// le cache des appels au fournisseur. Sans ce ménage, un `cv.pdf` d'une exécution
/// précédente restait en place quand l'export venait d'être refusé, et les contrôles
/// validaient un fichier que le code courant ne produit plus.
fn nettoyer_artefacts(dossier: &Path) {
    for nom in [
        "cv.pdf",
        "cv.png",
        "cv.html",
        "cv-layout.json",
        "cv-pdf.json",
        "cover-letter.pdf",
        "cover-letter.png",
        "cover-letter.html",
        "cover-letter-layout.json",
        "cover-letter-pdf.json",
        "workspace.json",
        "letter.json",
    ] {
        let _ = std::fs::remove_file(dossier.join(nom));
    }
    for nom in ["cv-pdf-pages", "cover-letter-pdf-pages"] {
        let _ = std::fs::remove_dir_all(dossier.join(nom));
    }
}

fn ecrire(chemin: &Path, contenu: &[u8]) {
    if let Some(parent) = chemin.parent() {
        std::fs::create_dir_all(parent).expect("dossier d'artefacts");
    }
    std::fs::write(chemin, contenu).unwrap_or_else(|error| {
        panic!("écriture de {} impossible : {error}", chemin.display());
    });
}

fn ecrire_json<T: serde::Serialize>(chemin: &Path, valeur: &T) {
    let rendu = serde_json::to_vec_pretty(valeur).expect("sérialisation d'artefact");
    ecrire(chemin, &rendu);
}

/// Résultat d'un cas, repris tel quel dans le rapport de fin de scénario.
#[derive(serde::Serialize, Default)]
struct Compte {
    profil: String,
    generation_ms: u128,
    lettre_ms: u128,
    cv_pdf_ms: u128,
    lettre_pdf_ms: u128,
    cv_pdf_octets: usize,
    lettre_pdf_octets: usize,
    /// Refus d'export attendu et obtenu : le cas est conforme, pas en échec.
    #[serde(skip_serializing_if = "Option::is_none")]
    refus_attendu: Option<String>,
    erreurs: Vec<String>,
}

#[tokio::test(flavor = "multi_thread")]
async fn chaine_de_generation_de_documents() {
    if std::env::var("CANDILOG_E2E").is_err() {
        eprintln!("CANDILOG_E2E absent : scénario de bout en bout ignoré.");
        return;
    }
    let offre = std::env::var("CANDILOG_E2E_OFFER")
        .ok()
        .map(PathBuf::from)
        .and_then(|path| std::fs::read_to_string(path).ok());
    let cas = cas_a_traiter();
    assert!(!cas.is_empty(), "aucun profil de test à traiter");

    let mut comptes = Vec::new();
    let mut echecs = Vec::new();
    for cas in &cas {
        let mut compte = Compte {
            profil: cas.slug.clone(),
            ..Compte::default()
        };
        nettoyer_artefacts(&cas.dossier);
        ecrire_json(&cas.dossier.join("profile.json"), &cas.profile);

        let generation = match obtenir_generation(cas, offre.as_deref(), &mut compte).await {
            Ok(generation) => generation,
            Err(erreur) => {
                compte.erreurs.push(format!("génération CV : {erreur}"));
                echecs.push(format!("{} — génération CV : {erreur}", cas.slug));
                comptes.push(compte);
                continue;
            }
        };
        ecrire_json(&cas.dossier.join("generation.json"), &generation);

        match prepare_workspace(&cas.profile, generation) {
            Ok(workspace) => {
                ecrire_json(&cas.dossier.join("workspace.json"), &workspace);
                let debut = Instant::now();
                let rendu = build(&workspace.document, None).render_bytes();
                compte.cv_pdf_ms = debut.elapsed().as_millis();
                // Un profil peut avoir pour résultat correct un refus : le CV de Candilog
                // tient sur une page, et un parcours qui la dépasse doit être annoncé, pas
                // tronqué. L'attente est déclarée à côté du profil, jamais devinée ici.
                match (rendu, cas.attendu.as_deref()) {
                    (Ok(octets), None) => {
                        compte.cv_pdf_octets = octets.len();
                        ecrire(&cas.dossier.join("cv.pdf"), &octets);
                    }
                    (Ok(_), Some(attendu)) => {
                        let message = format!(
                            "export PDF du CV accepté alors que « {attendu} » était attendu"
                        );
                        compte.erreurs.push(message.clone());
                        echecs.push(format!("{} — {message}", cas.slug));
                    }
                    (Err(erreur), Some("refus_longueur")) => {
                        let message = erreur.to_string();
                        if message.contains("ne tient pas sur une page") {
                            compte.refus_attendu = Some(message);
                        } else {
                            compte.erreurs.push(format!("refus inattendu : {message}"));
                            echecs.push(format!("{} — refus inattendu : {message}", cas.slug));
                        }
                    }
                    (Err(erreur), _) => {
                        compte.erreurs.push(format!("export PDF du CV : {erreur}"));
                        echecs.push(format!("{} — export PDF du CV : {erreur}", cas.slug));
                    }
                }
            }
            Err(erreur) => {
                compte.erreurs.push(format!("composition du CV : {erreur}"));
                echecs.push(format!("{} — composition du CV : {erreur}", cas.slug));
            }
        }

        match obtenir_lettre(cas, offre.as_deref(), &mut compte).await {
            Ok(corps) => {
                let export = CoverLetterExport {
                    name: format!("Lettre — {JOB_TITLE}"),
                    company: Some(COMPANY.to_owned()),
                    job_title: Some(JOB_TITLE.to_owned()),
                    recipient: None,
                    recipient_address: None,
                    job_reference: None,
                    content: corps.clone(),
                };
                ecrire(&cas.dossier.join("cover-letter.txt"), corps.as_bytes());
                ecrire_json(
                    &cas.dossier.join("letter.json"),
                    &serde_json::json!({
                        "identity": cas.profile.identity,
                        "company": export.company,
                        "job_title": export.job_title,
                        "recipient": export.recipient,
                        "recipient_address": export.recipient_address,
                        "job_reference": export.job_reference,
                        "content": export.content,
                    }),
                );
                let debut = Instant::now();
                match build_cover_letter(&cas.profile, &export).render_bytes() {
                    Ok(octets) => {
                        compte.lettre_pdf_ms = debut.elapsed().as_millis();
                        compte.lettre_pdf_octets = octets.len();
                        ecrire(&cas.dossier.join("cover-letter.pdf"), &octets);
                    }
                    Err(erreur) => {
                        compte
                            .erreurs
                            .push(format!("export PDF de la lettre : {erreur}"));
                        echecs.push(format!("{} — export PDF de la lettre : {erreur}", cas.slug));
                    }
                }
            }
            Err(erreur) => {
                compte
                    .erreurs
                    .push(format!("génération de la lettre : {erreur}"));
                echecs.push(format!("{} — génération de la lettre : {erreur}", cas.slug));
            }
        }
        comptes.push(compte);
    }

    ecrire_json(&out_dir().join("generation-report.json"), &comptes);
    assert!(
        echecs.is_empty(),
        "la chaîne de génération a échoué :\n  - {}",
        echecs.join("\n  - ")
    );
}

/// Génération du CV : appel réel au fournisseur en mode live, relecture sinon.
async fn obtenir_generation(
    cas: &Cas,
    offre: Option<&str>,
    compte: &mut Compte,
) -> Result<ResumeGeneration, String> {
    let cache = cas.dossier.join("generation.json");
    if !live() {
        let brut = std::fs::read_to_string(&cache)
            .map_err(|_| format!("aucune génération enregistrée dans {}", cache.display()))?;
        return serde_json::from_str(&brut).map_err(|error| error.to_string());
    }
    let offre = offre.ok_or_else(|| "CANDILOG_E2E_OFFER est requis en mode live".to_owned())?;
    let service = AiService::new(seeded_pool(&cas.profile));
    let debut = Instant::now();
    let generation = service
        .generate_resume(
            ResumeGenerationRequest {
                generation_id: format!("e2e-cv-{}", cas.slug),
                job_offer: offre.to_owned(),
            },
            |_| {},
        )
        .await
        .map_err(|error| error.to_string());
    compte.generation_ms = debut.elapsed().as_millis();
    generation
}

/// Lettre de motivation : appel réel au fournisseur en mode live, relecture sinon.
async fn obtenir_lettre(
    cas: &Cas,
    offre: Option<&str>,
    compte: &mut Compte,
) -> Result<String, String> {
    let cache = cas.dossier.join("cover-letter.txt");
    if !live() {
        return std::fs::read_to_string(&cache)
            .map_err(|_| format!("aucune lettre enregistrée dans {}", cache.display()));
    }
    let offre = offre.ok_or_else(|| "CANDILOG_E2E_OFFER est requis en mode live".to_owned())?;
    let service = AiService::new(seeded_pool(&cas.profile));
    let debut = Instant::now();
    let lettre = service
        .generate_cover_letter(
            CoverLetterRequest {
                generation_id: format!("e2e-lettre-{}", cas.slug),
                company: Some(COMPANY.to_owned()),
                job_title: Some(JOB_TITLE.to_owned()),
                tone: Some("formal".to_owned()),
                length: Some("medium".to_owned()),
                context: Some(offre.to_owned()),
                previous_cover_letter: None,
                instruction: None,
            },
            |_| {},
        )
        .await
        .map_err(|error| error.to_string());
    compte.lettre_ms = debut.elapsed().as_millis();
    lettre
}
