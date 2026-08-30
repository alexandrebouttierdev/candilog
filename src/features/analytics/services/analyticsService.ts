import { ipc } from "@/shared/services/ipc";
import type {
  Analytics,
  Period,
  Dashboard,
} from "@/shared/types/generated/analytics";

export type * from "@/shared/types/generated/analytics";

/** Frontière IPC unique du Dashboard et des Analytics. */
export const analyticsService = {
  dashboard: () => ipc<Dashboard>("analytics_dashboard"),
  load: (period: Period) => ipc<Analytics>("analytics_load", { period }),
  exportCsv: (period: Period) =>
    ipc<boolean>("analytics_export_csv", { period }),
};
