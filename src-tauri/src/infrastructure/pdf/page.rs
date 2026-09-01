//! Contrat géométrique partagé par les documents PDF.

use crate::core::errors::{AppError, AppResult};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PageSpec {
    pub width_mm: f32,
    pub height_mm: f32,
    pub width_pt: f32,
    pub height_pt: f32,
}

pub const A4: PageSpec = PageSpec {
    width_mm: 210.0,
    height_mm: 297.0,
    width_pt: 595.28,
    height_pt: 841.89,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Density {
    pub spacing_scale: f32,
    pub font_scale: f32,
}

pub const DENSITY_PROFILES: [Density; 5] = [
    Density {
        spacing_scale: 1.35,
        font_scale: 1.04,
    },
    Density {
        spacing_scale: 1.0,
        font_scale: 1.0,
    },
    Density {
        spacing_scale: 0.82,
        font_scale: 1.0,
    },
    Density {
        spacing_scale: 0.72,
        font_scale: 0.96,
    },
    Density {
        spacing_scale: 0.62,
        font_scale: 0.92,
    },
];

pub const MIN_BODY_FONT_PT: f32 = 8.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Margins {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Margins {
    #[must_use]
    pub const fn uniform(value: f32) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct LayoutBounds {
    pub max_x: f32,
    pub max_y: f32,
}

impl LayoutBounds {
    #[must_use]
    pub fn fits(self, page: PageSpec, margins: Margins) -> bool {
        self.max_x <= page.width_pt - margins.right && self.max_y <= page.height_pt - margins.bottom
    }
}

/// Vérifie les bornes finales avant toute sérialisation du PDF.
///
/// # Errors
/// Retourne une validation actionnable si une opération sortirait de la page.
pub fn ensure_inside(
    bounds: LayoutBounds,
    margins: Margins,
    overflow_message: &str,
) -> AppResult<()> {
    if bounds.fits(A4, margins) {
        Ok(())
    } else {
        Err(AppError::Validation(overflow_message.into()))
    }
}

#[cfg(test)]
#[path = "tests/page/mod.rs"]
mod tests;
