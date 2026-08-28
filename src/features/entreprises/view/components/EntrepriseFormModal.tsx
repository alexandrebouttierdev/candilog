import { useEffect } from "react";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import {
  entrepriseFormSchema,
  type EntrepriseFormInput,
  type EntrepriseFormValues,
} from "../../model/schemas/entreprise-form.schema";
import type { Entreprise, NouvelleEntreprise } from "../../services/entreprise.service";
import { useSecteurs } from "@/features/secteurs";
import { FormField, ModalHost, Select, TextArea, TextInput } from "@/shared/ui";

/** Valeurs d'un formulaire vierge. */
const VIDE: EntrepriseFormInput = {
  nom: "",
  secteurId: "",
  type: "",
  siteWeb: "",
  ville: "",
  adresse: "",
  notes: "",
};

/** Préremplit le formulaire depuis une entreprise existante. */
function depuis(entreprise: Entreprise): EntrepriseFormInput {
  return {
    nom: entreprise.nom,
    secteurId: entreprise.secteurId ?? "",
    type: entreprise.type ?? "",
    siteWeb: entreprise.siteWeb ?? "",
    ville: entreprise.ville ?? "",
    adresse: entreprise.adresse ?? "",
    notes: entreprise.notes ?? "",
  };
}

/**
 * Modale de création et de modification d'une entreprise.
 *
 * Structure reprise de `SPECDESIGN/Modales.dc.html` : deux sections, « Identité » et
 * « Localisation », et le rappel en pied que seul le nom est obligatoire.
 */
export function EntrepriseFormModal({
  open,
  entreprise,
  busy,
  onClose,
  onSubmit,
}: {
  open: boolean;
  /** `null` en création. */
  entreprise: Entreprise | null;
  busy: boolean;
  onClose: () => void;
  onSubmit: (valeurs: NouvelleEntreprise) => Promise<unknown>;
}) {
  const secteurs = useSecteurs();

  const form = useForm<EntrepriseFormInput, unknown, EntrepriseFormValues>({
    resolver: zodResolver(entrepriseFormSchema),
    defaultValues: VIDE,
  });

  // La modale n'est démontée qu'à la fermeture : sans réinitialisation à l'ouverture, elle
  // rouvrirait sur les valeurs de l'entité précédemment éditée.
  useEffect(() => {
    if (open) form.reset(entreprise ? depuis(entreprise) : VIDE);
  }, [open, entreprise, form]);

  const enregistrer = form.handleSubmit(async (valeurs) => {
    // Le libellé du secteur est dénormalisé à l'enregistrement : le backend et l'ancienne
    // base s'en servent pour l'affichage et la recherche, l'identifiant seul ne suffit pas.
    const secteur =
      secteurs.data?.find((item) => item.id === valeurs.secteurId)?.nom ?? null;

    await onSubmit({ ...valeurs, secteur });
    onClose();
  });

  return (
    <ModalHost
      open={open}
      icon="apartment"
      title={entreprise ? "Modifier l'entreprise" : "Nouvelle entreprise"}
      subtitle={
        entreprise ? entreprise.nom : "Ajoutez une société à votre répertoire"
      }
      footerNote="Seul le nom est obligatoire."
      submitDisabled={!form.formState.isValid && form.formState.isSubmitted}
      busy={busy}
      onClose={onClose}
      onSubmit={() => void enregistrer()}
      width="600px"
    >
      <form onSubmit={(event) => void enregistrer(event)} className="flex flex-col gap-5">
        <Section titre="Identité" icone="apartment">
          <div className="col-span-2">
            <FormField label="Nom" required error={form.formState.errors.nom?.message}>
              {(props) => (
                <TextInput
                  {...props}
                  {...form.register("nom")}
                  placeholder="Nom de l'entreprise"
                  invalid={Boolean(form.formState.errors.nom)}
                />
              )}
            </FormField>
          </div>

          <FormField label="Secteur d'activité">
            {(props) => (
              <Select {...props} {...form.register("secteurId")}>
                <option value="">Sélectionner…</option>
                {secteurs.data?.map((secteur) => (
                  <option key={secteur.id} value={secteur.id}>
                    {secteur.nom}
                  </option>
                ))}
              </Select>
            )}
          </FormField>

          <FormField label="Type">
            {(props) => (
              <TextInput {...props} {...form.register("type")} placeholder="Éditeur logiciel…" />
            )}
          </FormField>
        </Section>

        <Section titre="Localisation" icone="location_on">
          <FormField label="Site web" error={form.formState.errors.siteWeb?.message}>
            {(props) => (
              <TextInput
                {...props}
                {...form.register("siteWeb")}
                placeholder="https://…"
                invalid={Boolean(form.formState.errors.siteWeb)}
              />
            )}
          </FormField>

          <FormField label="Ville">
            {(props) => <TextInput {...props} {...form.register("ville")} placeholder="Ville" />}
          </FormField>

          <div className="col-span-2">
            <FormField label="Adresse">
              {(props) => (
                <TextInput {...props} {...form.register("adresse")} placeholder="Adresse" />
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
  titre,
  icone,
  children,
}: {
  titre: string;
  icone: string;
  children: React.ReactNode;
}) {
  return (
    <fieldset className="flex flex-col gap-3">
      <legend className="flex items-center gap-1.5 text-eyebrow uppercase text-ink-faint">
        <span className="material-symbols-rounded text-[14px]" aria-hidden="true">
          {icone}
        </span>
        {titre}
      </legend>
      <div className="grid grid-cols-2 gap-4">{children}</div>
    </fieldset>
  );
}
