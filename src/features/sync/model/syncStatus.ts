import type { GlyphTone } from "@/shared/ui";
import type { SyncPhase, SyncProviderKind, SyncStatus } from "@/shared/types/generated/sync";

/** Libellé court de chaque état, tel qu'affiché à côté du glyphe. */
export const PHASE_LABELS: Readonly<Record<SyncPhase, string>> = {
  not_configured: "Stockage local uniquement",
  disabled: "Synchronisation suspendue",
  up_to_date: "Synchronisé",
  syncing: "Synchronisation en cours",
  pending: "Modifications en attente",
  offline: "Hors ligne",
  auth_required: "Reconnexion nécessaire",
  key_required: "Phrase secrète nécessaire",
  temporary_error: "Erreur temporaire",
  conflict: "Conflit à arbitrer",
  restoring: "Restauration en cours",
  incompatible: "Mise à jour nécessaire",
};

/** Forme du glyphe : la couleur ne porte jamais seule l'état (`docs/DESIGN.md` §2). */
export function phaseTone(phase: SyncPhase): GlyphTone {
  switch (phase) {
    case "up_to_date":
      return "g";
    case "auth_required":
    case "key_required":
    case "incompatible":
      return "c";
    case "offline":
    case "temporary_error":
    case "conflict":
    case "pending":
      return "a";
    default:
      return "n";
  }
}

export const PROVIDER_LABELS: Readonly<Record<SyncProviderKind, string>> = {
  local: "Stockage local uniquement",
  google_drive: "Google Drive",
  one_drive: "Microsoft OneDrive",
  folder: "Dossier personnalisé",
};

/** L'état demande-t-il une action de l'utilisateur pour que la synchronisation reprenne ? */
export function needsAttention(status: SyncStatus): boolean {
  return ["auth_required", "key_required", "incompatible", "conflict"].includes(status.phase);
}

/** Décompte lisible des modifications en attente. */
export function pendingLabel(count: number): string {
  if (count === 0) return "Aucune";
  return count === 1 ? "1 modification" : `${count} modifications`;
}

/** Instant d'une synchronisation, en heure locale : « 10 octobre à 14:32 ». */
export function formatMoment(iso: string | null): string {
  if (!iso) return "Jamais";
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return iso;
  const day = new Intl.DateTimeFormat("fr-FR", { day: "2-digit", month: "long" }).format(date);
  const time = new Intl.DateTimeFormat("fr-FR", { hour: "2-digit", minute: "2-digit" }).format(date);
  return `${day} à ${time}`;
}
