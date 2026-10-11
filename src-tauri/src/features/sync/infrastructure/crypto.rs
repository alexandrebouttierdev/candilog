//! Chiffrement côté client des objets de synchronisation (`docs/SYNC.md`, « Chiffrement »).
//!
//! Hiérarchie de clés :
//!
//! - une **clé d'espace** aléatoire de 256 bits, tirée à la création de l'espace ;
//! - deux sous-clés dérivées par HKDF-SHA256 (sel : identifiant de l'espace) : l'une chiffre
//!   les objets (ChaCha20-Poly1305), l'autre nomme les fichiers joints (HMAC-SHA256), pour
//!   que leur nom ne révèle pas l'empreinte de leur contenu ;
//! - la clé d'espace est déposée dans le manifeste **chiffrée** par une clé dérivée de la
//!   phrase secrète de l'utilisateur (Argon2id, 64 Mio, 3 passes) ;
//! - la clé de récupération est la clé d'espace elle-même, encodée en base32.
//!
//! Chaque objet porte un nonce aléatoire de 96 bits et des données associées qui le lient à
//! son espace et à son nom : un fichier déplacé ou renommé chez le fournisseur ne se
//! déchiffre plus. Aucun algorithme n'est écrit ici ; tout vient de `ring` et `argon2`.

use crate::core::errors::{AppError, AppResult};
use crate::features::sync::domain::protocol::{
    KdfParams, SpaceManifest, FORMAT_VERSION, MAX_OBJECT_BYTES,
};
use base64::Engine;
use ring::aead::{self, Aad, LessSafeKey, Nonce, UnboundKey, NONCE_LEN};
use ring::rand::{SecureRandom, SystemRandom};
use ring::{hkdf, hmac};
use std::io::{Read, Write};
use zeroize::Zeroizing;

const MAGIC: &[u8; 4] = b"CLS1";
const KEY_LEN: usize = 32;
const SALT_LEN: usize = 16;
/// Longueur minimale de la phrase secrète, en caractères.
pub const MIN_PASSPHRASE_CHARS: usize = 12;
const MAX_PASSPHRASE_BYTES: usize = 1024;
const KEY_CHECK: &[u8] = b"candilog-key-check";

/// Paramètres Argon2id retenus pour un nouvel espace : au-dessus des recommandations OWASP
/// (19 Mio, 2 passes), pour une dérivation d'environ une demi-seconde, une seule fois par
/// ordinateur.
const ARGON2_MEMORY_KIB: u32 = 65_536;
const ARGON2_ITERATIONS: u32 = 3;
const ARGON2_PARALLELISM: u32 = 1;

/// Échec de déchiffrement ou de vérification.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CryptoError {
    /// Phrase secrète ou clé de récupération incorrecte.
    #[error("secret incorrect")]
    WrongSecret,
    /// Objet altéré, tronqué, déplacé ou chiffré par une autre clé.
    #[error("objet illisible ou altéré")]
    Corrupted,
}

/// Clé d'un espace, effacée de la mémoire à sa libération.
#[derive(Clone)]
pub struct SpaceKey(Zeroizing<[u8; KEY_LEN]>);

impl std::fmt::Debug for SpaceKey {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Jamais la valeur : une clé ne doit pas pouvoir finir dans un journal.
        formatter.write_str("SpaceKey(…)")
    }
}

impl SpaceKey {
    fn random() -> AppResult<Self> {
        let mut bytes = Zeroizing::new([0_u8; KEY_LEN]);
        SystemRandom::new()
            .fill(bytes.as_mut())
            .map_err(|_| AppError::Sync("Aléa système indisponible.".into()))?;
        Ok(Self(bytes))
    }

    /// Forme stockée dans le coffre système.
    #[must_use]
    pub fn to_base64(&self) -> Zeroizing<String> {
        Zeroizing::new(base64::engine::general_purpose::STANDARD.encode(self.0.as_ref()))
    }

    /// # Errors
    /// [`CryptoError::Corrupted`] si la valeur n'est pas une clé.
    pub fn from_base64(text: &str) -> Result<Self, CryptoError> {
        let bytes = Zeroizing::new(
            base64::engine::general_purpose::STANDARD
                .decode(text.trim())
                .map_err(|_| CryptoError::Corrupted)?,
        );
        Self::from_slice(&bytes)
    }

    fn from_slice(bytes: &[u8]) -> Result<Self, CryptoError> {
        let array: [u8; KEY_LEN] = bytes.try_into().map_err(|_| CryptoError::Corrupted)?;
        Ok(Self(Zeroizing::new(array)))
    }
}

fn random_bytes<const N: usize>() -> AppResult<[u8; N]> {
    let mut bytes = [0_u8; N];
    SystemRandom::new()
        .fill(&mut bytes)
        .map_err(|_| AppError::Sync("Aléa système indisponible.".into()))?;
    Ok(bytes)
}

fn aead_key(bytes: &[u8]) -> Result<LessSafeKey, CryptoError> {
    UnboundKey::new(&aead::CHACHA20_POLY1305, bytes)
        .map(LessSafeKey::new)
        .map_err(|_| CryptoError::Corrupted)
}

fn seal_with(key: &LessSafeKey, aad: &[u8], plaintext: &[u8]) -> AppResult<Vec<u8>> {
    let nonce_bytes = random_bytes::<NONCE_LEN>()?;
    let mut in_out = plaintext.to_vec();
    key.seal_in_place_append_tag(
        Nonce::assume_unique_for_key(nonce_bytes),
        Aad::from(aad),
        &mut in_out,
    )
    .map_err(|_| AppError::Sync("Chiffrement impossible.".into()))?;
    let mut output = Vec::with_capacity(MAGIC.len() + NONCE_LEN + in_out.len());
    output.extend_from_slice(MAGIC);
    output.extend_from_slice(&nonce_bytes);
    output.extend_from_slice(&in_out);
    Ok(output)
}

fn open_with(key: &LessSafeKey, aad: &[u8], data: &[u8]) -> Result<Vec<u8>, CryptoError> {
    let rest = data
        .strip_prefix(MAGIC.as_slice())
        .ok_or(CryptoError::Corrupted)?;
    if rest.len() < NONCE_LEN + aead::CHACHA20_POLY1305.tag_len() {
        return Err(CryptoError::Corrupted);
    }
    let (nonce_bytes, ciphertext) = rest.split_at(NONCE_LEN);
    let nonce =
        Nonce::try_assume_unique_for_key(nonce_bytes).map_err(|_| CryptoError::Corrupted)?;
    let mut in_out = ciphertext.to_vec();
    let plaintext = key
        .open_in_place(nonce, Aad::from(aad), &mut in_out)
        .map_err(|_| CryptoError::Corrupted)?;
    Ok(plaintext.to_vec())
}

fn object_aad(space_id: &str, name: &str) -> Vec<u8> {
    format!("candilog-sync/v{FORMAT_VERSION}\0{space_id}\0{name}").into_bytes()
}

/// Chiffreur des objets d'un espace.
pub struct SpaceCipher {
    space_id: String,
    encryption: LessSafeKey,
    naming: hmac::Key,
}

impl std::fmt::Debug for SpaceCipher {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SpaceCipher")
            .field("space_id", &self.space_id)
            .finish_non_exhaustive()
    }
}

impl SpaceCipher {
    /// # Errors
    /// `Sync` si la dérivation échoue (ne se produit pas avec une clé de 256 bits).
    pub fn new(space_id: &str, key: &SpaceKey) -> AppResult<Self> {
        let failure = || AppError::Sync("Dérivation de clé impossible.".into());
        let prk = hkdf::Salt::new(hkdf::HKDF_SHA256, space_id.as_bytes()).extract(key.0.as_ref());
        let encryption: UnboundKey = prk
            .expand(&[b"candilog-sync encryption v1"], &aead::CHACHA20_POLY1305)
            .map_err(|_| failure())?
            .into();
        let naming: hmac::Key = prk
            .expand(&[b"candilog-sync naming v1"], hmac::HMAC_SHA256)
            .map_err(|_| failure())?
            .into();
        Ok(Self {
            space_id: space_id.to_owned(),
            encryption: LessSafeKey::new(encryption),
            naming,
        })
    }

    /// Chiffre un objet destiné au nom `name`.
    ///
    /// # Errors
    /// `Sync` si l'aléa système est indisponible.
    pub fn seal(&self, name: &str, plaintext: &[u8]) -> AppResult<Vec<u8>> {
        seal_with(
            &self.encryption,
            &object_aad(&self.space_id, name),
            plaintext,
        )
    }

    /// Déchiffre un objet lu sous le nom `name`.
    ///
    /// # Errors
    /// [`CryptoError::Corrupted`] si l'objet est altéré, tronqué, renommé ou d'une autre clé.
    pub fn open(&self, name: &str, data: &[u8]) -> Result<Vec<u8>, CryptoError> {
        open_with(&self.encryption, &object_aad(&self.space_id, name), data)
    }

    /// Nom distant d'un fichier joint : HMAC de son contenu par une sous-clé de l'espace.
    #[must_use]
    pub fn blob_id(&self, content: &[u8]) -> String {
        crate::features::sync::infrastructure::store::hex(
            hmac::sign(&self.naming, content).as_ref(),
        )
    }

    /// Sérialise, compresse puis chiffre une valeur.
    ///
    /// # Errors
    /// `Serialization` ou `Sync`.
    pub fn encode_json<T: serde::Serialize>(&self, name: &str, value: &T) -> AppResult<Vec<u8>> {
        let json = serde_json::to_vec(value)?;
        let mut encoder =
            flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::default());
        encoder
            .write_all(&json)
            .and_then(|()| encoder.finish())
            .map_err(|error| AppError::Sync(format!("Compression impossible : {error}")))
            .and_then(|compressed| self.seal(name, &compressed))
    }

    /// Déchiffre, décompresse (taille bornée) puis désérialise une valeur.
    ///
    /// # Errors
    /// [`CryptoError::Corrupted`] pour tout objet illisible.
    pub fn decode_json<T: serde::de::DeserializeOwned>(
        &self,
        name: &str,
        data: &[u8],
    ) -> Result<T, CryptoError> {
        let compressed = self.open(name, data)?;
        let mut json = Vec::new();
        flate2::read::DeflateDecoder::new(compressed.as_slice())
            .take(u64::try_from(MAX_OBJECT_BYTES).unwrap_or(u64::MAX) + 1)
            .read_to_end(&mut json)
            .map_err(|_| CryptoError::Corrupted)?;
        if json.len() > MAX_OBJECT_BYTES {
            return Err(CryptoError::Corrupted);
        }
        serde_json::from_slice(&json).map_err(|_| CryptoError::Corrupted)
    }

    fn key_check(&self) -> AppResult<Vec<u8>> {
        seal_with(
            &self.encryption,
            &object_aad(&self.space_id, "key-check"),
            KEY_CHECK,
        )
    }

    fn verify_key_check(&self, data: &[u8]) -> Result<(), CryptoError> {
        let plain = open_with(
            &self.encryption,
            &object_aad(&self.space_id, "key-check"),
            data,
        )
        .map_err(|_| CryptoError::WrongSecret)?;
        if plain == KEY_CHECK {
            Ok(())
        } else {
            Err(CryptoError::WrongSecret)
        }
    }
}

/// Valide une phrase secrète choisie par l'utilisateur.
///
/// # Errors
/// `Validation` si elle est trop courte ou trop longue.
pub fn validate_passphrase(passphrase: &str) -> AppResult<()> {
    if passphrase.chars().count() < MIN_PASSPHRASE_CHARS {
        return Err(AppError::Validation(format!(
            "La phrase secrète doit compter au moins {MIN_PASSPHRASE_CHARS} caractères."
        )));
    }
    if passphrase.len() > MAX_PASSPHRASE_BYTES {
        return Err(AppError::Validation(
            "La phrase secrète est trop longue.".into(),
        ));
    }
    Ok(())
}

fn derive_kek(passphrase: &str, kdf: &KdfParams) -> AppResult<Zeroizing<[u8; KEY_LEN]>> {
    let salt = base64::engine::general_purpose::STANDARD
        .decode(&kdf.salt)
        .map_err(|_| AppError::Sync("Paramètres de chiffrement illisibles.".into()))?;
    let params = argon2::Params::new(
        kdf.memory_kib,
        kdf.iterations,
        kdf.parallelism,
        Some(KEY_LEN),
    )
    .map_err(|_| AppError::Sync("Paramètres de chiffrement non pris en charge.".into()))?;
    let argon = argon2::Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);
    let mut output = Zeroizing::new([0_u8; KEY_LEN]);
    argon
        .hash_password_into(passphrase.as_bytes(), &salt, output.as_mut())
        .map_err(|_| AppError::Sync("Dérivation de la phrase secrète impossible.".into()))?;
    Ok(output)
}

fn wrap_aad(space_id: &str) -> Vec<u8> {
    format!("candilog-space-key\0{space_id}").into_bytes()
}

/// Espace nouvellement créé : manifeste à publier, clé et clé de récupération.
pub struct NewSpace {
    pub manifest: SpaceManifest,
    pub key: SpaceKey,
    pub recovery_key: Zeroizing<String>,
}

/// Crée un espace : clé aléatoire, chiffrée par la phrase secrète.
///
/// # Errors
/// `Validation` pour une phrase secrète refusée, `Sync` pour un échec cryptographique.
pub fn create_space(space_id: &str, passphrase: &str, created_at: &str) -> AppResult<NewSpace> {
    validate_passphrase(passphrase)?;
    let key = SpaceKey::random()?;
    let salt = random_bytes::<SALT_LEN>()?;
    let kdf = KdfParams {
        algorithm: "argon2id".into(),
        memory_kib: ARGON2_MEMORY_KIB,
        iterations: ARGON2_ITERATIONS,
        parallelism: ARGON2_PARALLELISM,
        salt: base64::engine::general_purpose::STANDARD.encode(salt),
    };
    let kek = derive_kek(passphrase, &kdf)?;
    let wrapping = aead_key(kek.as_ref()).map_err(|_| AppError::Sync("Clé invalide.".into()))?;
    let wrapped = seal_with(&wrapping, &wrap_aad(space_id), key.0.as_ref())?;
    let cipher = SpaceCipher::new(space_id, &key)?;
    let key_check = cipher.key_check()?;
    let engine = base64::engine::general_purpose::STANDARD;
    Ok(NewSpace {
        manifest: SpaceManifest {
            format: FORMAT_VERSION,
            app: "candilog".into(),
            space_id: space_id.to_owned(),
            created_at: created_at.to_owned(),
            kdf,
            wrapped_key: engine.encode(wrapped),
            key_check: engine.encode(key_check),
        },
        recovery_key: encode_recovery_key(&key),
        key,
    })
}

/// Retrouve la clé d'un espace avec la phrase secrète.
///
/// # Errors
/// [`CryptoError::WrongSecret`] si la phrase ne déchiffre pas la clé.
pub fn unlock_with_passphrase(
    manifest: &SpaceManifest,
    passphrase: &str,
) -> AppResult<Result<SpaceKey, CryptoError>> {
    let kek = derive_kek(passphrase, &manifest.kdf)?;
    let Ok(wrapping) = aead_key(kek.as_ref()) else {
        return Ok(Err(CryptoError::WrongSecret));
    };
    let Ok(wrapped) = base64::engine::general_purpose::STANDARD.decode(&manifest.wrapped_key)
    else {
        return Ok(Err(CryptoError::Corrupted));
    };
    let key = match open_with(&wrapping, &wrap_aad(&manifest.space_id), &wrapped) {
        Ok(bytes) => Zeroizing::new(bytes),
        Err(_) => return Ok(Err(CryptoError::WrongSecret)),
    };
    Ok(SpaceKey::from_slice(&key))
}

/// Retrouve la clé d'un espace avec la clé de récupération.
///
/// # Errors
/// [`CryptoError::WrongSecret`] si la clé ne correspond pas à l'espace.
pub fn unlock_with_recovery(
    manifest: &SpaceManifest,
    recovery: &str,
) -> AppResult<Result<SpaceKey, CryptoError>> {
    let Some(key) = decode_recovery_key(recovery) else {
        return Ok(Err(CryptoError::WrongSecret));
    };
    let cipher = SpaceCipher::new(&manifest.space_id, &key)?;
    let Ok(check) = base64::engine::general_purpose::STANDARD.decode(&manifest.key_check) else {
        return Ok(Err(CryptoError::Corrupted));
    };
    Ok(cipher.verify_key_check(&check).map(|()| key))
}

const BASE32: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

/// Clé de récupération : base32 (RFC 4648) par groupes de quatre caractères.
#[must_use]
pub fn encode_recovery_key(key: &SpaceKey) -> Zeroizing<String> {
    let mut symbols = Zeroizing::new(String::new());
    let mut buffer: u32 = 0;
    let mut bits = 0;
    for byte in key.0.iter() {
        buffer = (buffer << 8) | u32::from(*byte);
        bits += 8;
        while bits >= 5 {
            bits -= 5;
            symbols.push(char::from(BASE32[((buffer >> bits) & 31) as usize]));
        }
    }
    if bits > 0 {
        symbols.push(char::from(BASE32[((buffer << (5 - bits)) & 31) as usize]));
    }
    let mut grouped = Zeroizing::new(String::new());
    for (index, symbol) in symbols.chars().enumerate() {
        if index > 0 && index % 4 == 0 {
            grouped.push('-');
        }
        grouped.push(symbol);
    }
    grouped
}

/// Décode une clé de récupération saisie (casse, espaces et tirets indifférents).
#[must_use]
pub fn decode_recovery_key(text: &str) -> Option<SpaceKey> {
    let mut bytes = Zeroizing::new(Vec::with_capacity(KEY_LEN));
    let mut buffer: u32 = 0;
    let mut bits = 0;
    for symbol in text.chars().filter(|c| !c.is_whitespace() && *c != '-') {
        let upper = symbol.to_ascii_uppercase();
        let value = BASE32
            .iter()
            .position(|candidate| char::from(*candidate) == upper)?;
        buffer = (buffer << 5) | u32::try_from(value).ok()?;
        bits += 5;
        if bits >= 8 {
            bits -= 8;
            bytes.push(u8::try_from((buffer >> bits) & 0xff).ok()?);
        }
    }
    SpaceKey::from_slice(&bytes).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SPACE: &str = "22222222-2222-4222-8222-222222222222";
    const PASSPHRASE: &str = "cheval agrafe batterie correcte";

    fn space() -> NewSpace {
        create_space(SPACE, PASSPHRASE, "2026-10-10T00:00:00Z").unwrap()
    }

    #[test]
    fn un_objet_chiffre_se_dechiffre_sous_son_nom() {
        let new = space();
        let cipher = SpaceCipher::new(SPACE, &new.key).unwrap();
        let sealed = cipher
            .seal("changes/x/1.batch", b"donnees personnelles")
            .unwrap();
        assert!(
            !sealed.windows(9).any(|w| w == b"personnel"),
            "rien en clair"
        );
        assert_eq!(
            cipher.open("changes/x/1.batch", &sealed).unwrap(),
            b"donnees personnelles"
        );
    }

    #[test]
    fn un_objet_renomme_ou_altere_est_refuse() {
        let new = space();
        let cipher = SpaceCipher::new(SPACE, &new.key).unwrap();
        let mut sealed = cipher.seal("a", b"contenu").unwrap();
        assert_eq!(cipher.open("b", &sealed), Err(CryptoError::Corrupted));
        let last = sealed.len() - 1;
        sealed[last] ^= 1;
        assert_eq!(cipher.open("a", &sealed), Err(CryptoError::Corrupted));
        assert_eq!(cipher.open("a", &sealed[..10]), Err(CryptoError::Corrupted));
    }

    #[test]
    fn deux_chiffrements_du_meme_contenu_different() {
        let new = space();
        let cipher = SpaceCipher::new(SPACE, &new.key).unwrap();
        assert_ne!(
            cipher.seal("a", b"x").unwrap(),
            cipher.seal("a", b"x").unwrap()
        );
    }

    #[test]
    fn la_phrase_secrete_retrouve_la_cle() {
        let new = space();
        let key = unlock_with_passphrase(&new.manifest, PASSPHRASE)
            .unwrap()
            .unwrap();
        let original = SpaceCipher::new(SPACE, &new.key).unwrap();
        let recovered = SpaceCipher::new(SPACE, &key).unwrap();
        let sealed = original.seal("a", b"x").unwrap();
        assert_eq!(recovered.open("a", &sealed).unwrap(), b"x");
    }

    #[test]
    fn une_phrase_secrete_incorrecte_est_refusee() {
        let new = space();
        assert_eq!(
            unlock_with_passphrase(&new.manifest, "une autre phrase longue")
                .unwrap()
                .unwrap_err(),
            CryptoError::WrongSecret
        );
    }

    #[test]
    fn la_cle_de_recuperation_retrouve_la_cle() {
        let new = space();
        let typed = new.recovery_key.to_lowercase().replace('-', " ");
        let key = unlock_with_recovery(&new.manifest, &typed)
            .unwrap()
            .unwrap();
        assert_eq!(key.to_base64().as_str(), new.key.to_base64().as_str());
    }

    #[test]
    fn une_cle_de_recuperation_d_un_autre_espace_est_refusee() {
        let first = space();
        let second = space();
        assert!(unlock_with_recovery(&first.manifest, &second.recovery_key)
            .unwrap()
            .is_err());
        assert!(unlock_with_recovery(&first.manifest, "pas une clé")
            .unwrap()
            .is_err());
    }

    #[test]
    fn une_phrase_trop_courte_est_refusee() {
        assert!(create_space(SPACE, "court", "").is_err());
    }

    #[test]
    fn la_cle_ne_s_affiche_jamais_en_debug() {
        let new = space();
        let debug = format!(
            "{:?} {:?}",
            new.key,
            SpaceCipher::new(SPACE, &new.key).unwrap()
        );
        assert!(!debug.contains(new.key.to_base64().as_str()));
    }

    #[test]
    fn json_compresse_et_chiffre_fait_l_aller_retour() {
        let new = space();
        let cipher = SpaceCipher::new(SPACE, &new.key).unwrap();
        let value = serde_json::json!({"nom": "Acme", "notes": "x".repeat(10_000)});
        let sealed = cipher.encode_json("n", &value).unwrap();
        assert!(
            sealed.len() < 2_000,
            "le contenu est compressé avant chiffrement"
        );
        let back: serde_json::Value = cipher.decode_json("n", &sealed).unwrap();
        assert_eq!(back, value);
    }

    #[test]
    fn le_nom_d_un_fichier_joint_ne_revele_pas_son_empreinte() {
        let new = space();
        let cipher = SpaceCipher::new(SPACE, &new.key).unwrap();
        let content = b"photo";
        let plain = crate::features::sync::infrastructure::store::hex(
            &crate::features::sync::infrastructure::store::sha2_digest(content),
        );
        assert_ne!(cipher.blob_id(content), plain);
        assert_eq!(cipher.blob_id(content).len(), 64);
    }
}
