import { useState } from "react";
import { EntityPicker } from "@/shared/ui";
import { useCompany, useCompanySearch, useCreateCompany } from "../../viewmodel/useCompany";
import { COMPANIES_KEY } from "../../viewmodel/companyKeys";
import { CompanyFormModal } from "./CompanyFormModal";

/**
 * Sélecteur d'entreprise du répertoire, avec création sur place.
 *
 * Appartient à la feature `companies` : c'est elle qui connaît la commande, le schéma et la
 * modale de création. Les autres écrans le consomment par le barrel, sans jamais toucher au
 * service des entreprises.
 *
 * La création ne ferme pas le formulaire appelant, qui reste monté derrière : sa saisie en
 * cours est donc intégralement conservée, et la nouvelle entreprise y est sélectionnée dès
 * l'enregistrement.
 */
export function CompanyPicker({
  id,
  describedBy,
  invalid = false,
  value,
  onChange,
}: {
  id?: string;
  describedBy?: string | undefined;
  invalid?: boolean;
  /** Identifiant sélectionné, ou `null`. */
  value: string | null;
  onChange: (id: string | null) => void;
}) {
  const [aCreer, setACreer] = useState<string | null>(null);
  const selected = useCompany(value).data ?? null;
  const searchCompanies = useCompanySearch();
  const creation = useCreateCompany();

  return (
    <>
      <EntityPicker
        {...(id === undefined ? {} : { id })}
        describedBy={describedBy}
        invalid={invalid}
        value={value}
        selectedLabel={selected?.name ?? null}
        placeholder="Rechercher une entreprise…"
        emptyHelp="Aucun résultat."
        queryKey={COMPANIES_KEY}
        onChange={onChange}
        onCreate={setACreer}
        createLabel="Créer"
        fetchPage={searchCompanies}
      />

      <CompanyFormModal
        open={aCreer !== null}
        company={null}
        defaultName={aCreer ?? ""}
        busy={creation.isPending}
        onClose={() => setACreer(null)}
        onSubmit={async (values) => {
          const company = await creation.mutateAsync(values);
          onChange(company.id);
        }}
      />
    </>
  );
}
