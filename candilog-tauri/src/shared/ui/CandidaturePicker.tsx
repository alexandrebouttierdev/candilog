import { useQuery } from "@tanstack/react-query";
import { candidatureService } from "@/features/candidatures/services/candidature.service";
import { EntityPicker } from "./EntityPicker";

/**
 * Sélecteur de candidature, partagé par les formulaires d'entretien et de relance.
 *
 * Vit dans `shared/ui` bien qu'il connaisse une feature : deux features distinctes en
 * dépendent, et le loger dans l'une d'elles ferait dépendre l'autre de sa voisine. Le
 * couplage reste dirigé vers `candidatures`, qui n'en sait rien.
 */
export function CandidaturePicker({
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
    queryFn: () => candidatureService.obtenir(value as string),
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
          ? `${selection.data.poste} — ${selection.data.entrepriseNom ?? ""}`.trim()
          : null
      }
      placeholder="Rechercher une candidature…"
      emptyHelp="Aucun résultat. Créez la candidature depuis l'écran Suivi."
      queryKey={["candidatures"]}
      onChange={onChange}
      fetchPage={async ({ page, pageSize, search }) => {
        const resultat = await candidatureService.listerPage({
          page,
          pageSize,
          filtre: {
            search,
            statut: null,
            contrat: null,
            entrepriseId: null,
            ville: "",
            poste: "",
            dateDebut: null,
            dateFin: null,
            tri: "date",
            descendant: true,
          },
        });
        return {
          ...resultat,
          items: resultat.items.map((candidature) => ({
            id: candidature.id,
            label: candidature.poste,
            meta: candidature.entrepriseNom ?? undefined,
          })),
        };
      }}
    />
  );
}
