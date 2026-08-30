import { useQuery } from "@tanstack/react-query";
import { applicationService } from "@/features/applications/services/applicationService";
import { FILTER_VIDE } from "@/features/applications/model/schemas/application-filter.schema";
import { EntityPicker } from "./EntityPicker";

/**
 * Sélecteur de candidature, partagé par les formulaires d'entretien et de relance.
 *
 * Vit dans `shared/ui` bien qu'il connaisse une feature : deux features distinctes en
 * dépendent, et le loger dans l'une d'elles ferait dépendre l'autre de sa voisine. Le
 * couplage reste dirigé vers `candidatures`, qui n'en sait rien.
 */
export function ApplicationPicker({
  id,
  describedBy,
  invalid,
  value,
  onChange,
}: {
  id?: string;
  describedBy?: string | undefined;
  invalid?: boolean;
  value: string | null;
  onChange: (id: string | null) => void;
}) {
  // Le libellé de la sélection est chargé à part : le sélecteur ne connaît que la page
  // courante de résultats, où la candidature déjà choisie peut ne pas figurer.
  const selection = useQuery({
    queryKey: ["candidatures", "detail", value],
    queryFn: () => applicationService.get(value as string),
    enabled: value !== null,
  });

  return (
    <EntityPicker
      {...(id !== undefined ? { id } : {})}
      describedBy={describedBy}
      invalid={invalid ?? false}
      value={value}
      selectedLabel={
        selection.data
          ? `${selection.data.job_title} — ${selection.data.company_name ?? ""}`.trim()
          : null
      }
      placeholder="Rechercher une candidature…"
      emptyHelp="Aucun résultat. Créez la candidature depuis l'écran Suivi."
      queryKey={["candidatures"]}
      onChange={onChange}
      fetchPage={async ({ page, page_size, search }) => {
        const resultat = await applicationService.listPage({
          page,
          page_size,
          filter: { ...FILTER_VIDE, search, sort: "date", descending: true, ids: [] },
        });
        return {
          ...resultat,
          items: resultat.items.map((application) => ({
            id: application.id,
            label: application.job_title,
            meta: application.company_name ?? undefined,
          })),
        };
      }}
    />
  );
}
