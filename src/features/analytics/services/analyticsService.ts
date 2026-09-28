import { ipc } from "@/shared/services/ipc";
import type {
  AgendaItem,
  Analytics,
  Period,
  Dashboard,
} from "@/shared/types/generated/analytics";

export type * from "@/shared/types/generated/analytics";

/** Frontière IPC unique du Dashboard et des Analytics. */
export const analyticsService = {
  dashboard: () => ipc<Dashboard>("analytics_dashboard"),
  /** Échéances d'Aujourd'hui : relances à faire (retards compris) et entretiens à 7 jours. */
  agenda: () => ipc<AgendaItem[]>("analytics_agenda"),
  load: (period: Period) => ipc<Analytics>("analytics_load", { period }),
  exportCsv: (period: Period) =>
    ipc<boolean>("analytics_export_csv", { period }),
};
