import { useEffect } from "react";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import {
  candidatureFilterSchema,
  FILTRE_VIDE,
  type CandidatureFilterInput,
  type CandidatureFilterValues,
} from "../../model/schemas/candidature-filter.schema";
import { CONTRATS, STATUTS, contratLabel } from "../../model/statuts";
import { versDateAffichee } from "@/shared/lib/dates";
import { Button, FormField, ModalHost, Select, TextInput } from "@/shared/ui";

/** Reconstitue les valeurs saisies depuis les filtres appliqués. */
function depuis(filtres: CandidatureFilterValues): CandidatureFilterInput {
  return {
    statut: filtres.statut,
    contrat: filtres.contrat,
    entrepriseId: filtres.entrepriseId,
    ville: filtres.ville,
    poste: filtres.poste,
    dateDebut: filtres.dateDebut ? versDateAffichee(filtres.dateDebut) : "",
    dateFin: filtres.dateFin ? versDateAffichee(filtres.dateFin) : "",
  };
}

/**
 * Filtres cumulables du suivi.
 *
 * En modale plutôt qu'en barre dépliante : sept critères tiennent mal dans un bandeau, et
 * les maquettes montrent une barre de pastilles résumant l'état plutôt que les champs eux-mêmes.
 */
export function CandidatureFilters({
  open,
  filtres,
  onClose,
  onApply,
  onReset,
}: {
  open: boolean;
  filtres: CandidatureFilterValues;
  onClose: () => void;
  onApply: (valeurs: CandidatureFilterValues) => void;
  onReset: () => void;
}) {
  const form = useForm<CandidatureFilterInput, unknown, CandidatureFilterValues>({
    resolver: zodResolver(candidatureFilterSchema),
    defaultValues: FILTRE_VIDE,
  });

  useEffect(() => {
    if (open) form.reset(depuis(filtres));
  }, [open, filtres, form]);

  const appliquer = form.handleSubmit((valeurs) => {
    onApply(valeurs);
    onClose();
  });

  const errors = form.formState.errors;

  return (
    <ModalHost
      open={open}
      icon="filter_alt"
      title="Filtrer les candidatures"
      subtitle="Les critères se cumulent"
      footerNote="Les dates sont saisies au format JJ-MM-AAAA."
      submitLabel="Appliquer"
      submitIcon="filter_alt"
      onClose={onClose}
      onSubmit={() => void appliquer()}
      width="560px"
    >
      <form onSubmit={(event) => void appliquer(event)} className="grid grid-cols-2 gap-4">
        <FormField label="Statut">
          {(props) => (
            <Select {...props} {...form.register("statut")}>
              <option value="">Tous</option>
              {STATUTS.map((statut) => (
                <option key={statut.valeur} value={statut.valeur}>
                  {statut.label}
                </option>
              ))}
            </Select>
          )}
        </FormField>

        <FormField label="Contrat">
          {(props) => (
            <Select {...props} {...form.register("contrat")}>
              <option value="">Tous</option>
              {CONTRATS.map((contrat) => (
                <option key={contrat} value={contrat}>
                  {contratLabel(contrat)}
                </option>
              ))}
            </Select>
          )}
        </FormField>

        <FormField label="Poste">
          {(props) => (
            <TextInput {...props} {...form.register("poste")} placeholder="Développeur…" />
          )}
        </FormField>

        <FormField label="Ville">
          {(props) => <TextInput {...props} {...form.register("ville")} placeholder="Rennes…" />}
        </FormField>

        <FormField label="Envoyée à partir du" error={errors.dateDebut?.message}>
          {(props) => (
            <TextInput
              {...props}
              {...form.register("dateDebut")}
              placeholder="JJ-MM-AAAA"
              inputMode="numeric"
              invalid={Boolean(errors.dateDebut)}
            />
          )}
        </FormField>

        <FormField label="Jusqu'au" error={errors.dateFin?.message}>
          {(props) => (
            <TextInput
              {...props}
              {...form.register("dateFin")}
              placeholder="JJ-MM-AAAA"
              inputMode="numeric"
              invalid={Boolean(errors.dateFin)}
            />
          )}
        </FormField>

        <div className="col-span-2">
          <Button
            variant="ghost"
            icon="filter_alt_off"
            onClick={() => {
              onReset();
              onClose();
            }}
          >
            Réinitialiser tous les filtres
          </Button>
        </div>
      </form>
    </ModalHost>
  );
}
