//! Photo de profil : formats acceptés, validation et normalisation.
//!
//! Le domaine ne connaît ni le disque ni la base : il transforme des octets en octets. Le
//! service décide seul de l'endroit où le résultat est écrit.

use crate::core::errors::{AppError, AppResult};
use image::ImageFormat;
use std::io::Cursor;

/// Taille maximale du fichier choisi par l'utilisateur.
///
/// Huit mégaoctets couvrent largement une photo d'identité issue d'un téléphone, tout en
/// bornant ce que le décodeur doit ingérer.
pub const MAX_SOURCE_BYTES: usize = 8 * 1024 * 1024;

/// Côté maximal de l'image conservée, en pixels.
///
/// Une photo de CV occupe quelques centimètres carrés : au-delà de 512 px, chaque pixel
/// supplémentaire alourdit le PDF sans rien ajouter à l'impression.
pub const MAX_SIDE: u32 = 512;

/// Extensions proposées par le dialogue natif, dans l'ordre d'usage.
pub const ACCEPTED_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "webp"];

/// Nom de fichier d'une nouvelle photo, unique à chaque enregistrement.
///
/// Un nom neuf plutôt qu'un nom fixe : la webview met les images en cache, et réécrire
/// `photo.png` afficherait l'ancienne jusqu'au redémarrage.
#[must_use]
pub fn nouveau_nom_fichier() -> String {
    format!("photo-{}.png", uuid::Uuid::new_v4())
}

/// Valide une image choisie par l'utilisateur et la normalise en PNG borné.
///
/// Le rapport largeur / hauteur est préservé : la photo est inscrite dans un carré de
/// `MAX_SIDE`, jamais étirée pour le remplir.
///
/// # Errors
/// `AppError::Validation` si le fichier est vide, trop volumineux, d'un format non accepté,
/// illisible ou impossible à réencoder.
pub fn normaliser(bytes: &[u8]) -> AppResult<Vec<u8>> {
    if bytes.is_empty() {
        return Err(AppError::Validation(
            "Le fichier sélectionné est vide.".into(),
        ));
    }
    if bytes.len() > MAX_SOURCE_BYTES {
        return Err(AppError::Validation(format!(
            "L'image ne doit pas dépasser {} Mo.",
            MAX_SOURCE_BYTES / (1024 * 1024)
        )));
    }

    let format = image::guess_format(bytes).map_err(|_| format_refuse())?;
    if !matches!(
        format,
        ImageFormat::Jpeg | ImageFormat::Png | ImageFormat::WebP
    ) {
        return Err(format_refuse());
    }

    // Le format déclaré par l'extension n'est jamais cru sur parole : le décodage part de la
    // signature réelle des octets, seule à distinguer une image d'un fichier renommé.
    //
    // Les bornes d'allocation ne sont pas redondantes avec `MAX_SOURCE_BYTES` : la taille du
    // fichier **compressé** ne borne pas celle de l'image décodée. Un PNG de quelques dizaines
    // de kilo-octets peut déclarer 65 535 × 65 535 pixels, soit ~17 Gio en RGBA — et le
    // redimensionnement à `MAX_SIDE` n'intervient qu'après le décodage complet. Sans ces
    // limites, choisir une telle image suffisait à faire tuer le processus par le noyau.
    let mut lecteur = image::ImageReader::new(Cursor::new(bytes));
    lecteur.set_format(format);
    lecteur.limits(decodeur_borne());
    let image = lecteur
        .decode()
        .map_err(|_| AppError::Validation("L'image est illisible ou endommagée.".into()))?;

    let image = if image.width() > MAX_SIDE || image.height() > MAX_SIDE {
        image.resize(MAX_SIDE, MAX_SIDE, image::imageops::FilterType::Lanczos3)
    } else {
        image
    };

    let mut sortie = Cursor::new(Vec::new());
    image
        .write_to(&mut sortie, ImageFormat::Png)
        .map_err(|error| {
            tracing::error!(%error, "photo de profil non réencodée");
            AppError::Validation("L'image n'a pas pu être convertie.".into())
        })?;
    Ok(sortie.into_inner())
}

/// Côté maximal accepté **à l'entrée**, avant redimensionnement.
///
/// Très au-delà de n'importe quelle photo d'appareil (8 000 px couvrent un capteur de
/// 64 Mpx), mais assez bas pour qu'une image forgée ne puisse pas demander une allocation
/// démesurée.
pub const MAX_SIDE_SOURCE: u32 = 8_000;

/// Plafond d'allocation du décodeur (256 Mio), au-delà du pire cas légitime.
///
/// `MAX_SIDE_SOURCE` au carré en RGBA tient dans cette borne ; une image qui la dépasse est
/// refusée par le décodeur au lieu d'être allouée.
const MAX_DECODE_BYTES: u64 = 256 * 1024 * 1024;

/// Bornes imposées au décodeur d'images.
fn decodeur_borne() -> image::Limits {
    let mut limites = image::Limits::no_limits();
    limites.max_alloc = Some(MAX_DECODE_BYTES);
    limites.max_image_width = Some(MAX_SIDE_SOURCE);
    limites.max_image_height = Some(MAX_SIDE_SOURCE);
    limites
}

fn format_refuse() -> AppError {
    AppError::Validation(
        "Le format de l'image n'est pas accepté. Utilisez un fichier JPEG, PNG ou WebP.".into(),
    )
}
