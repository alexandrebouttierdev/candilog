// Les composants de page ne sont pas réexportés ici : `AppRouter` les charge en
// `lazy()` par leur chemin complet, et un baril les réexportant les ramenait dans le
// chunk d'entrée dès qu'un module de la coque importait ce baril (`useNavCounts`).
export { documentsService } from "./services/documentsService";
export { RESUME_KEY, COVER_LETTERS_KEY, BASE_RESUME_KEY } from "./viewmodel/documentKeys";
