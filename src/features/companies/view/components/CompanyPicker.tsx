import { useState } from "react";
import { EntityPicker } from "@/shared/ui";
import { companyService } from "../../services/companyService";
import { useCompany, useCreateCompany } from "../../viewmodel/useCompany";
import { COMPANIES_KEY } from "../../viewmodel/useCompaniesViewModel";
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
        fetchPage={async ({ page, page_size, search }) => {
          const resultat = await companyService.listPage({
            page,
            page_size,
            filter: {
              search,
              sector_id: null,
              company_type_id: null,
              company_size: null,
            },
          });
          return {
            ...resultat,
            items: resultat.items.map((company) => ({
              id: company.id,
              label: company.name,
              meta:
                [company.sector_name, company.city].filter(Boolean).join(" · ") || undefined,
            })),
          };
        }}
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
