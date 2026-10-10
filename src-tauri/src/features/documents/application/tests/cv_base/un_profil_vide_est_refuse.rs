//! Un profil sans aucune matière (ni nom, ni expérience, ni formation, ni compétence) est
//! refusé nommément plutôt que de produire un CV vide.

use crate::core::errors::AppError;
use crate::features::documents::application::resume_workspace::compose_base_resume;
use crate::features::profile::domain::Profile;

#[test]
fn un_profil_vide_est_refuse() {
    let erreur = compose_base_resume(&Profile::default(), &[]).unwrap_err();
    assert!(matches!(erreur, AppError::Validation(message)
        if message.contains("Complétez votre profil")));
}
