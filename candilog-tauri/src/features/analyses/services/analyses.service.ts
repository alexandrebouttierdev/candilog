import { ipc } from "@/shared/services/ipc";
import type {
  Analyses,
  Periode,
  TableauDeBord,
} from "@/shared/types/generated/analyses";

export type * from "@/shared/types/generated/analyses";

/** Frontière IPC unique du Dashboard et des Analyses. */
export const analysesService = {
  tableauDeBord: () => ipc<TableauDeBord>("analyses_tableau_de_bord"),
  charger: (periode: Periode) => ipc<Analyses>("analyses_charger", { periode }),
  exporterCsv: (periode: Periode, chemin: string) =>
    ipc<void>("analyses_exporter_csv", { periode, chemin }),
};
