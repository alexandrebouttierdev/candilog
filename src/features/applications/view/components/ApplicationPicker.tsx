import { useQuery } from "@tanstack/react-query";
import { EntityPicker } from "@/shared/ui";
import {
  fetchApplicationPickerDetail,
  fetchApplicationPickerPage,
} from "../../viewmodel/fetchApplicationPickerPage";

/**
 * Sélecteur de candidature, partagé par les formulaires d'entretien et de relance.
 *
 * Vit dans la feature `applications` : le couplage IPC reste derrière le viewmodel.
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
  const selection = useQuery({
    queryKey: ["candidatures", "detail", value],
    queryFn: () => fetchApplicationPickerDetail(value as string),
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
      fetchPage={fetchApplicationPickerPage}
    />
  );
}
