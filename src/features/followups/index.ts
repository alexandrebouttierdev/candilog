export { FollowUpFormModal } from "./view/components/FollowUpFormModal";
export { followUpService } from "./services/followUpService";
export type { FollowUp, NewFollowUp } from "./services/followUpService";
export { followUpIcon } from "./model/types";

/** Racine de cache partagée par les écrans qui affichent ou créent des relances. */
export const FOLLOW_UPS_KEY = ["relances"] as const;
