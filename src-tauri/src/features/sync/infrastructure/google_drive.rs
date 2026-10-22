//! Fournisseur Google Drive (API Drive v3), limité au dossier d'application
//! `appDataFolder`.
//!
//! Permission demandée : `https://www.googleapis.com/auth/drive.appdata`, non sensible. Elle
//! ne donne accès qu'aux fichiers que Candilog crée dans son dossier caché : ni le reste du
//! Drive, ni les fichiers de l'utilisateur. Conséquences documentées (`docs/SYNC.md`) : le
//! dossier est invisible dans l'interface de Drive, il disparaît si l'utilisateur retire
//! l'accès de Candilog à son Drive, et seul Candilog peut le restaurer.
//!
//! Drive autorise deux fichiers de même nom : l'adaptateur vérifie l'existence avant toute
//! création et, s'il trouve des doublons (envoi rejoué), lit le premier — leurs contenus
//! sont identiques par construction, chaque lot étant encodé une fois puis conservé.

use crate::features::sync::domain::remote::{PutOutcome, RemoteError, RemoteObject, RemoteStore};
use crate::features::sync::infrastructure::authorized::{self, valid_object_name};
use crate::features::sync::infrastructure::oauth::{OAuthClient, OAuthSession};
use async_trait::async_trait;
use std::sync::Arc;

pub const SCOPE: &str = "https://www.googleapis.com/auth/drive.appdata";
const API_ROOT: &str = "https://www.googleapis.com/drive/v3";
const UPLOAD_ROOT: &str = "https://www.googleapis.com/upload/drive/v3";

/// Application OAuth Google, si cette version de Candilog en a reçu une à la compilation.
#[must_use]
pub fn oauth_client() -> Option<OAuthClient> {
    let client_id = option_env!("CANDILOG_GOOGLE_CLIENT_ID")?.trim();
    if client_id.is_empty() {
        return None;
    }
    Some(OAuthClient {
        provider: "google_drive",
        client_id: client_id.to_owned(),
        client_secret: option_env!("CANDILOG_GOOGLE_CLIENT_SECRET")
            .map(str::trim)
            .filter(|secret| !secret.is_empty())
            .map(str::to_owned),
        authorize_url: "https://accounts.google.com/o/oauth2/v2/auth".into(),
        token_url: "https://oauth2.googleapis.com/token".into(),
        revoke_url: Some("https://oauth2.googleapis.com/revoke".into()),
        scopes: SCOPE.into(),
        extra_authorize: vec![("access_type", "offline"), ("prompt", "consent")],
        scope_on_token: false,
    })
}

#[derive(serde::Deserialize)]
struct FileList {
    #[serde(default)]
    files: Vec<DriveFile>,
    #[serde(rename = "nextPageToken")]
    next_page_token: Option<String>,
}

#[derive(serde::Deserialize)]
struct DriveFile {
    id: String,
    name: String,
    #[serde(default)]
    size: Option<String>,
}

/// Espace de synchronisation dans le dossier d'application Google Drive.
pub struct GoogleDriveStore {
    session: Arc<OAuthSession>,
    api_root: String,
    upload_root: String,
}

impl GoogleDriveStore {
    #[must_use]
    pub fn new(session: Arc<OAuthSession>) -> Self {
        Self::with_roots(session, API_ROOT, UPLOAD_ROOT)
    }

    #[must_use]
    pub fn with_roots(session: Arc<OAuthSession>, api_root: &str, upload_root: &str) -> Self {
        Self {
            session,
            api_root: api_root.trim_end_matches('/').to_owned(),
            upload_root: upload_root.trim_end_matches('/').to_owned(),
        }
    }

    async fn files(&self, query: Option<&str>) -> Result<Vec<DriveFile>, RemoteError> {
        let mut files = Vec::new();
        let mut page: Option<String> = None;
        // Borne de sécurité : 200 pages de 1 000 fichiers.
        for _ in 0..200 {
            let url = format!("{}/files", self.api_root);
            let token = page.clone();
            let response = authorized::send(&self.session, |http| {
                let mut request = http.get(&url).query(&[
                    ("spaces", "appDataFolder"),
                    ("fields", "nextPageToken,files(id,name,size)"),
                    ("pageSize", "1000"),
                    ("orderBy", "name"),
                ]);
                if let Some(query) = query {
                    request = request.query(&[("q", query)]);
                }
                if let Some(token) = &token {
                    request = request.query(&[("pageToken", token.as_str())]);
                }
                request
            })
            .await?;
            if response.status().as_u16() == 404 {
                return Ok(files);
            }
            let list: FileList = authorized::json(response).await?;
            files.extend(list.files);
            match list.next_page_token {
                Some(next) => page = Some(next),
                None => return Ok(files),
            }
        }
        Err(RemoteError::Provider(
            "liste de fichiers trop longue".into(),
        ))
    }

    async fn ids_of(&self, name: &str) -> Result<Vec<String>, RemoteError> {
        if !valid_object_name(name) {
            return Err(RemoteError::Provider(format!(
                "nom d'objet refusé : {name}"
            )));
        }
        // Le nom ne contient ni apostrophe ni barre oblique inverse : il ne peut pas sortir
        // de la chaîne de la requête de recherche.
        let query = format!("name = '{name}' and 'appDataFolder' in parents");
        let mut ids: Vec<String> = self
            .files(Some(&query))
            .await?
            .into_iter()
            .filter(|file| file.name == name)
            .map(|file| file.id)
            .collect();
        ids.sort();
        Ok(ids)
    }
}

#[async_trait]
impl RemoteStore for GoogleDriveStore {
    async fn list(&self, prefix: &str) -> Result<Vec<RemoteObject>, RemoteError> {
        let mut objects: Vec<RemoteObject> = self
            .files(None)
            .await?
            .into_iter()
            .filter(|file| file.name.starts_with(prefix) && valid_object_name(&file.name))
            .map(|file| RemoteObject {
                size: file.size.and_then(|size| size.parse().ok()).unwrap_or(0),
                name: file.name,
            })
            .collect();
        objects.sort_by(|left, right| left.name.cmp(&right.name));
        objects.dedup_by(|left, right| left.name == right.name);
        Ok(objects)
    }

    async fn get(&self, name: &str) -> Result<Option<Vec<u8>>, RemoteError> {
        let Some(id) = self.ids_of(name).await?.into_iter().next() else {
            return Ok(None);
        };
        let url = format!("{}/files/{id}", self.api_root);
        let response = authorized::send(&self.session, |http| {
            http.get(&url).query(&[("alt", "media")])
        })
        .await?;
        if response.status().as_u16() == 404 {
            return Ok(None);
        }
        authorized::body(response).await.map(Some)
    }

    async fn put_new(&self, name: &str, bytes: &[u8]) -> Result<PutOutcome, RemoteError> {
        if !self.ids_of(name).await?.is_empty() {
            return Ok(PutOutcome::AlreadyExists);
        }
        let boundary = format!("candilog-{}", uuid::Uuid::new_v4().simple());
        let metadata = serde_json::json!({ "name": name, "parents": ["appDataFolder"] });
        let mut body = Vec::with_capacity(bytes.len() + 512);
        body.extend_from_slice(
            format!("--{boundary}\r\nContent-Type: application/json; charset=UTF-8\r\n\r\n{metadata}\r\n--{boundary}\r\nContent-Type: application/octet-stream\r\n\r\n").as_bytes(),
        );
        body.extend_from_slice(bytes);
        body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
        let url = format!("{}/files", self.upload_root);
        let content_type = format!("multipart/related; boundary={boundary}");
        let response = authorized::send(&self.session, |http| {
            http.post(&url)
                .query(&[("uploadType", "multipart"), ("fields", "id")])
                .header(reqwest::header::CONTENT_TYPE, content_type.as_str())
                .body(body.clone())
        })
        .await?;
        if !response.status().is_success() {
            return Err(RemoteError::Provider(format!(
                "envoi refusé ({})",
                response.status().as_u16()
            )));
        }
        Ok(PutOutcome::Created)
    }

    async fn delete(&self, name: &str) -> Result<(), RemoteError> {
        for id in self.ids_of(name).await? {
            let url = format!("{}/files/{id}", self.api_root);
            authorized::send(&self.session, |http| http.delete(&url)).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::sync::infrastructure::authorized::test_server::{
        serve, token_ok, Handler, Request,
    };
    use crate::features::sync::infrastructure::secrets::{MemorySecrets, SyncSecrets};
    use std::collections::BTreeMap;
    use std::sync::Mutex;

    /// Fichiers du Drive simulé : identifiant → (nom, contenu).
    type Files = Arc<Mutex<BTreeMap<String, (String, Vec<u8>)>>>;

    /// Émulation minimale de l'API Drive v3 pour `appDataFolder`.
    fn fake_drive(files: Files, seen: Arc<Mutex<Vec<Request>>>) -> Handler {
        Arc::new(move |request: &Request| {
            seen.lock().unwrap().push(request.clone());
            let url = url::Url::parse(&format!("http://x{}", request.target)).unwrap();
            let query: BTreeMap<String, String> = url.query_pairs().into_owned().collect();
            let path = url.path().to_owned();
            if path == "/token" {
                return token_ok();
            }
            if request.header("authorization") != Some("Bearer acces-test") {
                return (401, vec![], vec![]);
            }
            let mut files = files.lock().unwrap();
            match (request.method.as_str(), path.as_str()) {
                ("GET", "/drive/v3/files") => {
                    assert_eq!(
                        query.get("spaces").map(String::as_str),
                        Some("appDataFolder")
                    );
                    let wanted = query
                        .get("q")
                        .and_then(|q| q.split('\'').nth(1).map(str::to_owned));
                    let list: Vec<_> = files
                        .iter()
                        .filter(|(_, (name, _))| wanted.as_ref().is_none_or(|w| w == name))
                        .map(|(id, (name, bytes))| serde_json::json!({"id": id, "name": name, "size": bytes.len().to_string()}))
                        .collect();
                    (
                        200,
                        vec![],
                        serde_json::to_vec(&serde_json::json!({"files": list})).unwrap(),
                    )
                }
                ("POST", "/upload/drive/v3/files") => {
                    let boundary = request
                        .header("content-type")
                        .unwrap()
                        .split("boundary=")
                        .nth(1)
                        .unwrap()
                        .to_owned();
                    let text = request.body.clone();
                    let marker = format!("--{boundary}");
                    let parts: Vec<&[u8]> = split(&text, marker.as_bytes());
                    let meta_part = parts[1];
                    let meta_json = &meta_part
                        [meta_part.windows(4).position(|w| w == b"\r\n\r\n").unwrap() + 4..];
                    let metadata: serde_json::Value =
                        serde_json::from_slice(trim_crlf(meta_json)).unwrap();
                    assert_eq!(metadata["parents"][0], "appDataFolder");
                    let data_part = parts[2];
                    let data = trim_crlf(
                        &data_part
                            [data_part.windows(4).position(|w| w == b"\r\n\r\n").unwrap() + 4..],
                    )
                    .to_vec();
                    let id = format!("id{}", files.len() + 1);
                    files.insert(
                        id.clone(),
                        (metadata["name"].as_str().unwrap().to_owned(), data),
                    );
                    (
                        200,
                        vec![],
                        serde_json::to_vec(&serde_json::json!({"id": id})).unwrap(),
                    )
                }
                ("GET", other) if other.starts_with("/drive/v3/files/") => {
                    let id = other.trim_start_matches("/drive/v3/files/");
                    files.get(id).map_or((404, vec![], vec![]), |(_, bytes)| {
                        (200, vec![], bytes.clone())
                    })
                }
                ("DELETE", other) if other.starts_with("/drive/v3/files/") => {
                    files.remove(other.trim_start_matches("/drive/v3/files/"));
                    (204, vec![], vec![])
                }
                _ => (400, vec![], vec![]),
            }
        })
    }

    fn split<'a>(data: &'a [u8], marker: &[u8]) -> Vec<&'a [u8]> {
        let mut parts = Vec::new();
        let mut rest = data;
        while let Some(position) = rest.windows(marker.len()).position(|w| w == marker) {
            parts.push(&rest[..position]);
            rest = &rest[position + marker.len()..];
        }
        parts.push(rest);
        parts
    }

    fn trim_crlf(data: &[u8]) -> &[u8] {
        data.strip_suffix(b"\r\n").unwrap_or(data)
    }

    async fn store() -> (GoogleDriveStore, Arc<Mutex<Vec<Request>>>) {
        let files = Arc::new(Mutex::new(BTreeMap::new()));
        let seen = Arc::new(Mutex::new(Vec::new()));
        let base = serve(fake_drive(files, Arc::clone(&seen))).await;
        let secrets = Arc::new(MemorySecrets::default());
        secrets
            .store("sync-oauth-google_drive", "renouvellement")
            .unwrap();
        let mut client = oauth_client().unwrap_or(OAuthClient {
            provider: "google_drive",
            client_id: "test".into(),
            client_secret: None,
            authorize_url: String::new(),
            token_url: String::new(),
            revoke_url: None,
            scopes: SCOPE.into(),
            extra_authorize: vec![],
            scope_on_token: false,
        });
        client.token_url = format!("{base}/token");
        let session = Arc::new(OAuthSession::new(client, reqwest::Client::new(), secrets));
        (
            GoogleDriveStore::with_roots(
                session,
                &format!("{base}/drive/v3"),
                &format!("{base}/upload/drive/v3"),
            ),
            seen,
        )
    }

    #[tokio::test]
    async fn un_objet_fait_l_aller_retour_dans_le_dossier_d_application() {
        let (drive, _) = store().await;
        let name = "changes/11111111-1111-4111-8111-111111111111/000000000001.batch";
        assert_eq!(
            drive.put_new(name, b"\x00chiffre\xff").await.unwrap(),
            PutOutcome::Created
        );
        assert_eq!(drive.get(name).await.unwrap().unwrap(), b"\x00chiffre\xff");
        assert_eq!(drive.list("changes/").await.unwrap().len(), 1);
        assert_eq!(
            drive.put_new(name, b"autre").await.unwrap(),
            PutOutcome::AlreadyExists
        );
        drive.delete(name).await.unwrap();
        assert_eq!(drive.get(name).await.unwrap(), None);
    }

    #[tokio::test]
    async fn seul_le_dossier_d_application_est_interroge() {
        let (drive, seen) = store().await;
        drive.list("").await.unwrap();
        let requests = seen.lock().unwrap();
        assert!(requests
            .iter()
            .filter(|r| r.target.starts_with("/drive"))
            .all(|r| r.target.contains("spaces=appDataFolder")));
    }

    #[tokio::test]
    async fn un_nom_qui_pourrait_detourner_la_recherche_est_refuse() {
        let (drive, _) = store().await;
        assert!(drive.get("x' or name contains '").await.is_err());
        assert!(drive.put_new("../x", b"").await.is_err());
    }
}
