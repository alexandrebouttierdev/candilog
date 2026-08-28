import { useEffect } from "react";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import {
  applicationFilterSchema,
  FILTER_VIDE,
  type ApplicationFilterInput,
  type ApplicationFilterValues,
} from "../../model/schemas/application-filter.schema";
import { Contracts, Statuses, contract_label } from "../../model/statuses";
import { versDateAffichee } from "@/shared/lib/dates";
import { Button, FormField, ModalHost, Select, TextInput } from "@/shared/ui";

/** Reconstitue les valeurs saisies depuis les filtres appliqués. */
function from(filters: ApplicationFilterValues): ApplicationFilterInput {
  return {
    status: filters.status,
    contract: filters.contract,
    company_id: filters.company_id,
    city: filters.city,
    job_title: filters.job_title,
    start_date: filters.start_date ? versDateAffichee(filters.start_date) : "",
    end_date: filters.end_date ? versDateAffichee(filters.end_date) : "",
  };
}

/**
 * Filters cumulables du suivi.
 *
 * En modale plutôt qu'en barre dépliante : sept critères tiennent mal dans un bandeau, et
 * les maquettes montrent une barre de pastilles résumant l'état plutôt que les champs eux-mêmes.
 */
export function ApplicationFilters({
  open,
  filters,
  onClose,
  onApply,
  onReset,
}: {
  open: boolean;
  filters: ApplicationFilterValues;
  onClose: () => void;
  onApply: (values: ApplicationFilterValues) => void;
  onReset: () => void;
}) {
  const form = useForm<ApplicationFilterInput, unknown, ApplicationFilterValues>({
    resolver: zodResolver(applicationFilterSchema),
    defaultValues: FILTER_VIDE,
  });

  useEffect(() => {
    if (open) form.reset(from(filters));
  }, [open, filters, form]);

  const appliquer = form.handleSubmit((values) => {
    onApply(values);
    onClose();
  });

  const errors = form.formState.errors;

  return (
    <ModalHost
      open={open}
      icon="filter_alt"
      title="Filtrer les candidatures"
      subtitle="Les critères se cumulent"
      footer_note="Les dates sont saisies au format JJ-MM-AAAA."
      submitLabel="Appliquer"
      submitIcon="filter_alt"
      onClose={onClose}
      onSubmit={() => void appliquer()}
      width="560px"
    >
      <form onSubmit={(event) => void appliquer(event)} className="grid grid-cols-2 gap-4">
        <FormField label="Statut">
          {(props) => (
            <Select {...props} {...form.register("status")}>
              <option value="">Tous</option>
              {Statuses.map((status) => (
                <option key={status.value} value={status.value}>
                  {status.label}
                </option>
              ))}
            </Select>
          )}
        </FormField>

        <FormField label="Contrat">
          {(props) => (
            <Select {...props} {...form.register("contract")}>
              <option value="">Tous</option>
              {Contracts.map((contract) => (
                <option key={contract} value={contract}>
                  {contract_label(contract)}
                </option>
              ))}
            </Select>
          )}
        </FormField>

        <FormField label="Poste">
          {(props) => (
            <TextInput {...props} {...form.register("job_title")} placeholder="Développeur…" />
          )}
        </FormField>

        <FormField label="Ville">
          {(props) => <TextInput {...props} {...form.register("city")} placeholder="Rennes…" />}
        </FormField>

        <FormField label="Envoyée à partir du" error={errors.start_date?.message}>
          {(props) => (
            <TextInput
              {...props}
              {...form.register("start_date")}
              placeholder="JJ-MM-AAAA"
              inputMode="numeric"
              invalid={Boolean(errors.start_date)}
            />
          )}
        </FormField>

        <FormField label="Jusqu'au" error={errors.end_date?.message}>
          {(props) => (
            <TextInput
              {...props}
              {...form.register("end_date")}
              placeholder="JJ-MM-AAAA"
              inputMode="numeric"
              invalid={Boolean(errors.end_date)}
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
