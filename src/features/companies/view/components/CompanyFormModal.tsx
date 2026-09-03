import { useEffect } from "react";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import {
  companyFormSchema,
  type CompanyFormInput,
  type CompanyFormValues,
} from "../../model/schemas/company-form.schema";
import type { Company, NewCompany } from "../../services/companyService";
import { CompanySizes, useReferentials } from "@/features/referentials";
import { FormField, ModalHost, Select, TextArea, TextInput } from "@/shared/ui";
import type { IconName } from "@/shared/ui/icon-names";

/** Valeurs d'un formulaire vierge. */
const VIDE: CompanyFormInput = {
  name: "",
  sector_id: "",
  company_type_id: "",
  company_size: "UNKNOWN",
  website: "",
  city: "",
  address: "",
  notes: "",
};

/** Préremplit le formulaire depuis une entreprise existante. */
function from(company: Company): CompanyFormInput {
  return {
    name: company.name,
    sector_id: company.sector_id ?? "",
    company_type_id: company.company_type_id ?? "",
    company_size: company.company_size,
    website: company.website ?? "",
    city: company.city ?? "",
    address: company.address ?? "",
    notes: company.notes ?? "",
  };
}

/**
 * Modale de création et de modification d'une entreprise.
 *
 * Trois sections : identité, qualification, localisation. Secteur et type d'entreprise
 * viennent des référentiels de la base ; la taille est un jeu fermé de six valeurs.
 */
export function CompanyFormModal({
  open,
  company,
  defaultName = "",
  busy,
  onClose,
  onSubmit,
}: {
  open: boolean;
  /** `null` en création. */
  company: Company | null;
  /** Nom prérempli en création, repris de la recherche restée sans résultat. */
  defaultName?: string;
  busy: boolean;
  onClose: () => void;
  onSubmit: (values: NewCompany) => Promise<unknown>;
}) {
  const referentials = useReferentials();

  const form = useForm<CompanyFormInput, unknown, CompanyFormValues>({
    resolver: zodResolver(companyFormSchema),
    defaultValues: VIDE,
  });

  // La modale n'est démontée qu'à la fermeture : sans réinitialisation à l'ouverture, elle
  // rouvrirait sur les valeurs de l'entité précédemment éditée.
  useEffect(() => {
    if (open) form.reset(company ? from(company) : { ...VIDE, name: defaultName });
  }, [open, company, defaultName, form]);

  // L'échec d'enregistrement est déjà annoncé par la mutation appelante. Il est intercepté
  // ici pour que la modale reste ouverte sur la saisie en cours, et pour ne pas laisser
  // filer un rejet non traité.
  const save = form.handleSubmit(async (values) => {
    try {
      await onSubmit(values);
    } catch {
      return;
    }
    onClose();
  });

  return (
    <ModalHost
      open={open}
      icon="apartment"
      title={company ? "Modifier l'entreprise" : "Nouvelle entreprise"}
      subtitle={company ? company.name : "Ajoutez une société à votre répertoire"}
      footer_note="Seul le nom est obligatoire."
      submitDisabled={!form.formState.isValid && form.formState.isSubmitted}
      busy={busy}
      onClose={onClose}
      onSubmit={() => void save()}
      width="600px"
    >
      <form onSubmit={(event) => void save(event)} className="flex flex-col gap-5">
        <Section title="Identité" icon="apartment">
          <div className="col-span-2">
            <FormField label="Nom" required error={form.formState.errors.name?.message}>
              {(props) => (
                <TextInput
                  {...props}
                  {...form.register("name")}
                  placeholder="Nom de l'entreprise"
                  invalid={Boolean(form.formState.errors.name)}
                />
              )}
            </FormField>
          </div>
        </Section>

        <Section title="Qualification" icon="category">
          <FormField label="Secteur d'activité">
            {(props) => (
              <Select {...props} {...form.register("sector_id")}>
                <option value="">Sélectionner…</option>
                {referentials.data.sectors.map((sector) => (
                  <option key={sector.id} value={sector.id}>
                    {sector.name}
                  </option>
                ))}
              </Select>
            )}
          </FormField>

          <FormField label="Type d'entreprise">
            {(props) => (
              <Select {...props} {...form.register("company_type_id")}>
                <option value="">Sélectionner…</option>
                {referentials.data.company_types.map((type) => (
                  <option key={type.code} value={type.code}>
                    {type.name}
                  </option>
                ))}
              </Select>
            )}
          </FormField>

          <FormField label="Taille de l'entreprise">
            {(props) => (
              <Select {...props} {...form.register("company_size")}>
                {CompanySizes.map((size) => (
                  <option key={size.value} value={size.value}>
                    {size.label}
                  </option>
                ))}
              </Select>
            )}
          </FormField>
        </Section>

        <Section title="Localisation" icon="location_on">
          <FormField label="Site web" error={form.formState.errors.website?.message}>
            {(props) => (
              <TextInput
                {...props}
                {...form.register("website")}
                placeholder="https://…"
                invalid={Boolean(form.formState.errors.website)}
              />
            )}
          </FormField>

          <FormField label="Ville">
            {(props) => <TextInput {...props} {...form.register("city")} placeholder="Ville" />}
          </FormField>

          <div className="col-span-2">
            <FormField label="Adresse">
              {(props) => (
                <TextInput {...props} {...form.register("address")} placeholder="Adresse" />
              )}
            </FormField>
          </div>

          <div className="col-span-2">
            <FormField label="Notes">
              {(props) => (
                <TextArea
                  {...props}
                  {...form.register("notes")}
                  placeholder="Contexte, culture, informations utiles…"
                />
              )}
            </FormField>
          </div>
        </Section>
      </form>
    </ModalHost>
  );
}

/** Section de formulaire : sur-titre, filet, grille à deux colonnes. */
function Section({
  title,
  icon,
  children,
}: {
  title: string;
  icon: IconName;
  children: React.ReactNode;
}) {
  return (
    <fieldset className="flex flex-col gap-3">
      <legend className="flex items-center gap-1.5 text-eyebrow uppercase text-ink-faint">
        <span className="material-symbols-rounded text-[14px]" aria-hidden="true">
          {icon}
        </span>
        {title}
      </legend>
      <div className="grid grid-cols-2 gap-4">{children}</div>
    </fieldset>
  );
}
