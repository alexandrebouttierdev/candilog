// Les composants de page ne sont pas réexportés ici : `AppRouter` les charge en
// `lazy()` par leur chemin complet, et un baril les réexportant les ramenait dans le
// chunk d'entrée dès qu'un module de la coque importait ce baril (`useNavCounts`).
export { useProfilePhoto, PROFILE_KEY, PROFILE_PHOTO_KEY } from "./viewmodel/useProfileViewModel";
export { profileService } from "./services/profileService";
