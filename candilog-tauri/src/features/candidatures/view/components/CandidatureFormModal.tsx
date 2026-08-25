import { useEffect } from "react";
import { Controller, useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import {
  candidatureFormSchema,
  versDateAffichee,
  type CandidatureFormInput,
  type CandidatureFormValues,
} from "../../model/schemas/candidature-form.schema";
import { CONTRATS, STATUTS, contratLabel } from "../../model/statuts";
import type { Candidature, NouvelleCandidature } from "../../services/candidature.service";
import { useQuery } from "@tanstack/react-query";
import { entrepriseService } from "@/features/entreprises/services/entreprise.service";
import { EntityPicker, FormField, ModalHost, Select, TextArea, TextInput } from "@/shared/ui";

/** Date du jour au format saisi, valeur par défaut du champ « Date d'envoi ». */
function aujourdhui(): string {
  return versDateAffichee(new Date().toISOString().slice(0, 10));
}

function vide(): CandidatureFormInput {
  return {
    poste: "",
    entrepriseId: "",
    typeContrat: "CDI",
    statut: "EN_ATTENTE",
    dateEnvoi: aujourdhui(),
    lienOffre: "",
    notes: "",
  };
}

function depuis(candidature: Candidature): CandidatureFormInput {
  return {
    poste: candidature.poste,
    entrepriseId: candidature.entrepriseId,
    typeContrat: candidature.typeContrat,
    statut: candidature.statut,
    dateEnvoi: versDateAffichee(candidature.dateEnvoi),
    lienOffre: candidature.lienOffre ?? "",
    notes: candidature.notes ?? "",
  };
}

/**
 * Modale de création et de modification d'une candidature.
 *
 * Structure reprise de `SPECDESIGN/Modales.dc.html` : sections « Poste visé » et « Suivi »,
 * sélecteur d'entreprise paginé, rappel du format de date en pied.
 */
export function CandidatureFormModal({
  open,
  candidature,
  busy,
  onClose,
  onSubmit,
}: {
  open: boolean;
  candidature: Candidature | null;
  busy: boolean;
  onClose: () => void;
  onSubmit: (valeurs: NouvelleCandidature) => Promise<unknown>;
}) {
  const form = useForm<CandidatureFormInput, unknown, CandidatureFormValues>({
    resolver: zodResolver(candidatureFormSchema),
    defaultValues: vide(),
  });

  useEffect(() => {
    if (open) form.reset(candidature ? depuis(candidature) : vide());
  }, [open, candidature, form]);

  const enregistrer = form.handleSubmit(async (valeurs) => {
    await onSubmit(valeurs);
    onClose();
  });

  const errors = form.formState.errors;

  return (
    <ModalHost
      open={open}
      icon="work"
      title={candidature ? "Modifier la candidature" : "Nouvelle candidature"}
      subtitle={
        candidature
          ? `${candidature.poste} — ${candidature.entrepriseNom ?? ""}`
          : "Renseignez le poste et l'entreprise visés"
      }
      footerNote="Les dates sont saisies au format JJ-MM-AAAA."
      busy={busy}
      onClose={onClose}
      onSubmit={() => void enregistrer()}
      width="620px"
    >
      <form onSubmit={(event) => void enregistrer(event)} className="flex flex-col gap-5">
        <fieldset className="flex flex-col gap-3">
          <legend className="text-eyebrow uppercase text-ink-faint">Poste visé</legend>
          <div className="flex flex-col gap-4">
            <FormField label="Poste" required error={errors.poste?.message}>
              {(props) => (
                <TextInput
                  {...props}
                  {...form.register("poste")}
                  placeholder="Développeur Frontend"
                  invalid={Boolean(errors.poste)}
                />
              )}
            </FormField>

            <FormField label="Entreprise" required error={errors.entrepriseId?.message}>
              {(props) => (
                <Controller
                  control={form.control}
                  name="entrepriseId"
                  render={({ field }) => (
                    <EntrepriseFieldPicker
                      id={props.id}
                      describedBy={props["aria-describedby"]}
                      invalid={props["aria-invalid"]}
                      value={field.value || null}
                      onChange={(id) => field.onChange(id ?? "")}
                    />
                  )}
                />
              )}
            </FormField>
          </div>
        </fieldset>

        <fieldset className="flex flex-col gap-3">
          <legend className="text-eyebrow uppercase text-ink-faint">Suivi</legend>
          <div className="grid grid-cols-3 gap-4">
            <FormField label="Contrat">
              {(props) => (
                <Select {...props} {...form.register("typeContrat")}>
                  {CONTRATS.map((contrat) => (
                    <option key={contrat} value={contrat}>
                      {contratLabel(contrat)}
                    </option>
                  ))}
                </Select>
              )}
            </FormField>

            <FormField label="Statut">
              {(props) => (
                <Select {...props} {...form.register("statut")}>
                  {STATUTS.map((statut) => (
                    <option key={statut.valeur} value={statut.valeur}>
                      {statut.label}
                    </option>
                  ))}
                </Select>
              )}
            </FormField>

            <FormField label="Date d'envoi" required error={errors.dateEnvoi?.message}>
              {(props) => (
                <TextInput
                  {...props}
                  {...form.register("dateEnvoi")}
                  placeholder="JJ-MM-AAAA"
                  inputMode="numeric"
                  invalid={Boolean(errors.dateEnvoi)}
                />
              )}
            </FormField>

            <div className="col-span-3">
              <FormField label="Lien de l'offre" error={errors.lienOffre?.message}>
                {(props) => (
                  <TextInput
                    {...props}
                    {...form.register("lienOffre")}
                    placeholder="https://…"
                    invalid={Boolean(errors.lienOffre)}
                  />
                )}
              </FormField>
            </div>

            <div className="col-span-3">
              <FormField label="Notes">
                {(props) => (
                  <TextArea
                    {...props}
                    {...form.register("notes")}
                    placeholder="Contexte, personne rencontrée, éléments à retenir…"
                  />
                )}
              </FormField>
            </div>
          </div>
        </fieldset>
      </form>
    </ModalHost>
  );
}

/**
 * Sélecteur d'entreprise du formulaire, branché sur le répertoire paginé.
 *
 * Le libellé de la sélection est chargé séparément : le sélecteur ne connaît que la page
 * courante, et l'entreprise déjà choisie peut n'y figurer pas.
 */
function EntrepriseFieldPicker({
  id,
  describedBy,
  invalid,
  value,
  onChange,
}: {
  id: string;
  describedBy: string | undefined;
  invalid: boolean;
  value: string | null;
  onChange: (id: string | null) => void;
}) {
  return (
    <EntityPicker
      id={id}
      describedBy={describedBy}
      invalid={invalid}
      value={value}
      selectedLabel={useLibelleEntreprise(value)}
      placeholder="Rechercher une entreprise…"
      emptyHelp="Aucun résultat. Créez l'entreprise depuis l'écran Relations."
      queryKey={["entreprises"]}
      onChange={onChange}
      fetchPage={async ({ page, pageSize, search }) => {
        const resultat = await entrepriseService.listerPage({
          page,
          pageSize,
          search,
          companyType: null,
        });
        return {
          ...resultat,
          items: resultat.items.map((entreprise) => ({
            id: entreprise.id,
            label: entreprise.nom,
            meta:
              [entreprise.secteur, entreprise.ville].filter(Boolean).join(" · ") || undefined,
          })),
        };
      }}
    />
  );
}

/** Libellé de l'entreprise sélectionnée, chargé hors de la page courante du sélecteur. */
function useLibelleEntreprise(id: string | null): string | null {
  const { data } = useQueryEntreprise(id);
  return data?.nom ?? null;
}

function useQueryEntreprise(id: string | null) {
  return useQuery({
    queryKey: ["entreprises", "detail", id],
    queryFn: () => entrepriseService.obtenir(id as string),
    enabled: id !== null,
  });
}
