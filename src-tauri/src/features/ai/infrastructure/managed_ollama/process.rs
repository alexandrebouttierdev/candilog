//! Cycle de vie du processus Ollama privé Candilog.

use crate::core::errors::{AppError, AppResult};
use crate::features::ai::domain::MANAGED_OLLAMA_PREFERRED_PORT;
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::Duration;

pub struct ManagedOllamaProcess {
    child: Mutex<Option<Child>>,
    port: Mutex<Option<u16>>,
    executable: PathBuf,
    models_dir: PathBuf,
}

impl ManagedOllamaProcess {
    pub fn new(executable: PathBuf, models_dir: PathBuf) -> Self {
        Self {
            child: Mutex::new(None),
            port: Mutex::new(None),
            executable,
            models_dir,
        }
    }

    pub fn ensure_running(&self) -> AppResult<u16> {
        if let Some(port) = *self.port.lock().map_err(lock_err)? {
            if self.health_check(port) {
                return Ok(port);
            }
        }
        self.stop()?;
        let port = pick_local_port()?;
        let host = format!("127.0.0.1:{port}");
        let child = Command::new(&self.executable)
            .arg("serve")
            .env("OLLAMA_HOST", &host)
            .env("OLLAMA_MODELS", &self.models_dir)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|error| {
                AppError::Provider(format!("Le moteur local n'a pas pu démarrer : {error}"))
            })?;
        *self.child.lock().map_err(lock_err)? = Some(child);
        *self.port.lock().map_err(lock_err)? = Some(port);
        for _ in 0..60 {
            if self.health_check(port) {
                return Ok(port);
            }
            std::thread::sleep(Duration::from_millis(250));
        }
        self.stop()?;
        Err(AppError::Provider(
            "Le moteur local n'a pas répondu à temps.".into(),
        ))
    }

    pub fn stop(&self) -> AppResult<()> {
        let mut guard = self.child.lock().map_err(lock_err)?;
        if let Some(mut child) = guard.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        *self.port.lock().map_err(lock_err)? = None;
        Ok(())
    }

    pub fn health_check(&self, port: u16) -> bool {
        let url = format!("http://127.0.0.1:{port}/api/tags");
        reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(2))
            .build()
            .and_then(|client| client.get(&url).send())
            .map(|response| response.status().is_success())
            .unwrap_or(false)
    }

    #[must_use]
    pub fn base_url(&self) -> Option<String> {
        self.port
            .lock()
            .ok()
            .and_then(|port| port.map(|value| format!("http://127.0.0.1:{value}")))
    }
}

fn pick_local_port() -> AppResult<u16> {
    for port in MANAGED_OLLAMA_PREFERRED_PORT..MANAGED_OLLAMA_PREFERRED_PORT + 50 {
        if TcpListener::bind(("127.0.0.1", port)).is_ok() {
            return Ok(port);
        }
    }
    Err(AppError::Provider(
        "Aucun port local disponible pour l'IA locale.".into(),
    ))
}

fn lock_err<T>(_: std::sync::PoisonError<T>) -> AppError {
    AppError::Provider("État interne du moteur local corrompu.".into())
}
