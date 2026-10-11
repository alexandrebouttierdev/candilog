//! Fournisseur Microsoft OneDrive (Microsoft Graph v1.0), limité au dossier d'application
//! `approot` (« Applications/Candilog » dans le OneDrive de l'utilisateur).
//!
//! Permissions demandées : `Files.ReadWrite.AppFolder` (lecture et écriture du seul dossier
//! de l'application) et `offline_access` (jeton de renouvellement). Application publique :
//! aucun secret client. Contrairement au dossier Google, celui-ci est **visible** par
//! l'utilisateur, qui peut le déplacer, le renommer ou y supprimer des fichiers : le moteur
//! traite ces cas comme des objets manquants (`docs/SYNC.md`).

use crate::features::sync::domain::remote::{PutOutcome, RemoteError, RemoteObject, RemoteStore};
use crate::features::sync::infrastructure::authorized::{self, valid_object_name};
use crate::features::sync::infrastructure::oauth::{OAuthClient, OAuthSession};
use async_trait::async_trait;
use std::sync::Arc;

pub const SCOPES: &str = "https://graph.microsoft.com/Files.ReadWrite.AppFolder offline_access";
const APP_ROOT: &str = "https://graph.microsoft.com/v1.0/me/drive/special/approot";

/// Application OAuth Microsoft, si cette version de Candilog en a reçu une à la compilation.
#[must_use]
pub fn oauth_client() -> Option<OAuthClient> {
    let client_id = option_env!("CANDILOG_MICROSOFT_CLIENT_ID")?.trim();
    if client_id.is_empty() {
        return None;
    }
    Some(OAuthClient {
        provider: "one_drive",
        client_id: client_id.to_owned(),
        client_secret: None,
        authorize_url: "https://login.microsoftonline.com/common/oauth2/v2.0/authorize".into(),
        token_url: "https://login.microsoftonline.com/common/oauth2/v2.0/token".into(),
        revoke_url: None,
        scopes: SCOPES.into(),
        extra_authorize: vec![("response_mode", "query"), ("prompt", "select_account")],
        scope_on_token: true,
    })
}

#[derive(serde::Deserialize)]
struct Children {
    #[serde(default)]
    value: Vec<Item>,
    #[serde(rename = "@odata.nextLink")]
    next_link: Option<String>,
}

#[derive(serde::Deserialize)]
struct Item {
    name: String,
    #[serde(default)]
    size: u64,
    #[serde(default)]
    folder: Option<serde_json::Value>,
}

/// Espace de synchronisation dans le dossier d'application OneDrive.
pub struct OneDriveStore {
    session: Arc<OAuthSession>,
    root: String,
}

impl OneDriveStore {
    #[must_use]
    pub fn new(session: Arc<OAuthSession>) -> Self {
        Self::with_root(session, APP_ROOT)
    }

    #[must_use]
    pub fn with_root(session: Arc<OAuthSession>, root: &str) -> Self {
        Self {
            session,
            root: root.trim_end_matches('/').to_owned(),
        }
    }

    fn item_url(&self, name: &str) -> Result<String, RemoteError> {
        if !valid_object_name(name) {
            return Err(RemoteError::Provider(format!(
                "nom d'objet refusé : {name}"
            )));
        }
        Ok(format!("{}:/{name}", self.root))
    }

    /// Enfants d'un dossier (`None` : le dossier n'existe pas encore).
    async fn children(&self, folder: &str) -> Result<Option<Vec<Item>>, RemoteError> {
        let mut url = if folder.is_empty() {
            format!("{}/children", self.root)
        } else {
            format!("{}:/children", self.item_url(folder)?)
        };
        let mut items = Vec::new();
        let mut first = true;
        for _ in 0..200 {
            let current = url.clone();
            let response = authorized::send(&self.session, |http| {
                let request = http.get(&current);
                if first {
                    request.query(&[("$select", "name,size,folder,file"), ("$top", "999")])
                } else {
                    request
                }
            })
            .await?;
            if response.status().as_u16() == 404 {
                return Ok(None);
            }
            let page: Children = authorized::json(response).await?;
            items.extend(page.value);
            match page.next_link {
                // La page suivante doit rester chez le même service : un lien vers un autre
                // hôte recevrait le jeton d'accès.
                Some(next) if next.starts_with(&self.origin()) => {
                    url = next;
                    first = false;
                }
                Some(_) => {
                    return Err(RemoteError::Provider("lien de pagination inattendu".into()))
                }
                None => return Ok(Some(items)),
            }
        }
        Err(RemoteError::Provider(
            "liste de fichiers trop longue".into(),
        ))
    }

    fn origin(&self) -> String {
        url::Url::parse(&self.root)
            .map(|url| url.origin().ascii_serialization())
            .unwrap_or_default()
    }

    async fn exists(&self, name: &str) -> Result<bool, RemoteError> {
        let url = self.item_url(name)?;
        let response = authorized::send(&self.session, |http| {
            http.get(&url).query(&[("$select", "name")])
        })
        .await?;
        Ok(response.status().as_u16() != 404)
    }
}

#[async_trait]
impl RemoteStore for OneDriveStore {
    async fn list(&self, prefix: &str) -> Result<Vec<RemoteObject>, RemoteError> {
        // Parcours en profondeur bornée depuis le dossier le plus précis du préfixe.
        let start = prefix
            .rsplit_once('/')
            .map_or("", |(folder, _)| folder)
            .to_owned();
        let mut objects = Vec::new();
        let mut stack = vec![(start, 0_usize)];
        while let Some((folder, depth)) = stack.pop() {
            let Some(items) = self.children(&folder).await? else {
                continue;
            };
            for item in items {
                let name = if folder.is_empty() {
                    item.name.clone()
                } else {
                    format!("{folder}/{}", item.name)
                };
                if !valid_object_name(&name) {
                    continue;
                }
                if item.folder.is_some() {
                    if depth < 3 {
                        stack.push((name, depth + 1));
                    }
                } else if name.starts_with(prefix) {
                    objects.push(RemoteObject {
                        name,
                        size: item.size,
                    });
                }
            }
        }
        objects.sort_by(|left, right| left.name.cmp(&right.name));
        Ok(objects)
    }

    async fn get(&self, name: &str) -> Result<Option<Vec<u8>>, RemoteError> {
        let url = format!("{}:/content", self.item_url(name)?);
        // Graph répond par une redirection vers une URL de téléchargement pré-signée ; le
        // client HTTP la suit en HTTPS seulement et sans transmettre le jeton.
        let response = authorized::send(&self.session, |http| http.get(&url)).await?;
        if response.status().as_u16() == 404 {
            return Ok(None);
        }
        authorized::body(response).await.map(Some)
    }

    async fn put_new(&self, name: &str, bytes: &[u8]) -> Result<PutOutcome, RemoteError> {
        if self.exists(name).await? {
            return Ok(PutOutcome::AlreadyExists);
        }
        let url = format!("{}:/content", self.item_url(name)?);
        let response = authorized::send(&self.session, |http| {
            http.put(&url)
                .query(&[("@microsoft.graph.conflictBehavior", "fail")])
                .header(reqwest::header::CONTENT_TYPE, "application/octet-stream")
                .body(bytes.to_vec())
        })
        .await?;
        match response.status().as_u16() {
            200 | 201 => Ok(PutOutcome::Created),
            409 => Ok(PutOutcome::AlreadyExists),
            status => Err(RemoteError::Provider(format!("envoi refusé ({status})"))),
        }
    }

    async fn delete(&self, name: &str) -> Result<(), RemoteError> {
        let url = self.item_url(name)?;
        authorized::send(&self.session, |http| http.delete(&url)).await?;
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

    /// Émulation minimale du dossier d'application Graph, adressé par chemin.
    fn fake_graph(
        files: Arc<Mutex<BTreeMap<String, Vec<u8>>>>,
        limited: Arc<Mutex<bool>>,
    ) -> Handler {
        Arc::new(move |request: &Request| {
            let target = percent_decode(&request.target);
            let (path, _) = target.split_once('?').unwrap_or((&target, ""));
            if path == "/token" {
                return token_ok();
            }
            if std::mem::take(&mut *limited.lock().unwrap()) {
                return (429, vec![("Retry-After".into(), "7".into())], vec![]);
            }
            if request.header("authorization") != Some("Bearer acces-test") {
                return (401, vec![], vec![]);
            }
            let mut files = files.lock().unwrap();
            let Some(rest) = path.strip_prefix("/approot") else {
                return (400, vec![], vec![]);
            };
            if let Some(folder) = rest
                .strip_prefix(":/")
                .and_then(|p| p.strip_suffix(":/children"))
                .or(if rest == "/children" { Some("") } else { None })
            {
                let prefix = if folder.is_empty() {
                    String::new()
                } else {
                    format!("{folder}/")
                };
                if !folder.is_empty() && !files.keys().any(|k| k.starts_with(&prefix)) {
                    return (404, vec![], vec![]);
                }
                let mut seen = std::collections::BTreeSet::new();
                let items: Vec<_> = files
                    .iter()
                    .filter_map(|(name, bytes)| {
                        let rest = name.strip_prefix(&prefix)?;
                        Some(match rest.split_once('/') {
                            Some((dir, _)) => seen
                                .insert(dir.to_owned())
                                .then(|| serde_json::json!({"name": dir, "folder": {}}))?,
                            None => {
                                serde_json::json!({"name": rest, "size": bytes.len(), "file": {}})
                            }
                        })
                    })
                    .collect();
                return (
                    200,
                    vec![],
                    serde_json::to_vec(&serde_json::json!({"value": items})).unwrap(),
                );
            }
            if let Some(name) = rest
                .strip_prefix(":/")
                .and_then(|p| p.strip_suffix(":/content"))
            {
                return match request.method.as_str() {
                    "GET" => files
                        .get(name)
                        .map_or((404, vec![], vec![]), |b| (200, vec![], b.clone())),
                    "PUT" if files.contains_key(name) => (409, vec![], vec![]),
                    "PUT" => {
                        assert!(request.target.contains("conflictBehavior=fail"));
                        files.insert(name.to_owned(), request.body.clone());
                        (201, vec![], b"{}".to_vec())
                    }
                    _ => (400, vec![], vec![]),
                };
            }
            if let Some(name) = rest.strip_prefix(":/") {
                return match request.method.as_str() {
                    "GET" => {
                        if files.contains_key(name) {
                            (200, vec![], b"{}".to_vec())
                        } else {
                            (404, vec![], vec![])
                        }
                    }
                    "DELETE" => {
                        files.remove(name);
                        (204, vec![], vec![])
                    }
                    _ => (400, vec![], vec![]),
                };
            }
            (400, vec![], vec![])
        })
    }

    fn percent_decode(text: &str) -> String {
        url::form_urlencoded::parse(format!("x={}", text.replace('+', "%2B")).as_bytes())
            .next()
            .map(|(_, value)| value.into_owned())
            .unwrap_or_default()
    }

    async fn store() -> (OneDriveStore, Arc<Mutex<bool>>) {
        let files = Arc::new(Mutex::new(BTreeMap::new()));
        let limited = Arc::new(Mutex::new(false));
        let base = serve(fake_graph(files, Arc::clone(&limited))).await;
        let secrets = Arc::new(MemorySecrets::default());
        secrets
            .store("sync-oauth-one_drive", "renouvellement")
            .unwrap();
        let client = OAuthClient {
            provider: "one_drive",
            client_id: "test".into(),
            client_secret: None,
            authorize_url: String::new(),
            token_url: format!("{base}/token"),
            revoke_url: None,
            scopes: SCOPES.into(),
            extra_authorize: vec![],
            scope_on_token: true,
        };
        let session = Arc::new(OAuthSession::new(client, reqwest::Client::new(), secrets));
        (
            OneDriveStore::with_root(session, &format!("{base}/approot")),
            limited,
        )
    }

    #[tokio::test]
    async fn un_objet_fait_l_aller_retour_dans_le_dossier_d_application() {
        let (drive, _) = store().await;
        let name = "changes/11111111-1111-4111-8111-111111111111/000000000001.batch";
        assert_eq!(
            drive.put_new(name, b"chiffre").await.unwrap(),
            PutOutcome::Created
        );
        assert_eq!(
            drive.put_new(name, b"autre").await.unwrap(),
            PutOutcome::AlreadyExists
        );
        assert_eq!(drive.get(name).await.unwrap().unwrap(), b"chiffre");
        let listed = drive.list("changes/").await.unwrap();
        assert_eq!(
            listed,
            vec![RemoteObject {
                name: name.into(),
                size: 7
            }]
        );
        drive.delete(name).await.unwrap();
        assert_eq!(drive.get(name).await.unwrap(), None);
        assert!(drive.list("changes/").await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn une_limite_de_debit_rend_le_delai_demande() {
        let (drive, limited) = store().await;
        *limited.lock().unwrap() = true;
        let error = drive.get("space.json").await.unwrap_err();
        assert_eq!(
            error,
            RemoteError::RateLimited(Some(std::time::Duration::from_secs(7)))
        );
    }

    #[tokio::test]
    async fn un_nom_hors_du_dossier_est_refuse() {
        let (drive, _) = store().await;
        assert!(drive.get("../../Documents/cv.pdf").await.is_err());
        assert!(drive.put_new("a/../b", b"").await.is_err());
    }
}
