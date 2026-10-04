// Les composants de page ne sont pas réexportés ici : `AppRouter` les charge en
// `lazy()` par leur chemin complet, et un baril les réexportant les ramenait dans le
// chunk d'entrée dès qu'un module de la coque importait ce baril (`useNavCounts`).
export type { ActiveSavedView } from "./view/pages/ApplicationsPage";
export { useApplicationsViewModel, APPLICATIONS_KEY } from "./viewmodel/useApplicationsViewModel";
export type { Application, NewApplication } from "./services/applicationService";
export type { ApplicationFilter, ApplicationStatus } from "./services/applicationService";
export { applicationService } from "./services/applicationService";
export { EMPTY_FILTER } from "./model/schemas/application-filter.schema";
export { Statuses, status_meta } from "./model/statuses";
export { ApplicationPicker } from "./view/components/ApplicationPicker";
export { Channels, channelLabel, formatReference } from "./model/presentation";
export { useScheduleFollowUp } from "./viewmodel/useScheduleFollowUp";
