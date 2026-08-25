import { useEffect } from "react";
import { Controller, useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import {
  entretienFormSchema,
  type EntretienFormInput,
  type EntretienFormValues,
} from "../../model/schemas/entretien-form.schema";
import { TYPES_ENTRETIEN } from "../../model/types";
import type { Entretien, NouvelEntretien } from "../../services/entretien.service";
import { contactService } from "@/features/contacts/services/contact.service";
import { useQuery } from "@tanstack/react-query";
import { dateDepuisHorodatage, heureDepuisHorodatage, versDateAffichee } from "@/shared/lib/dates";
import {
  CandidaturePicker,
  FormField,
  ModalHost,
  Select,
  TextArea,
  TextInput,
} from "@/shared/ui";

function vide(candidatureId: string | null, jour: string | null): EntretienFormInput {
  return {
    candidatureId: candidatureId ?? "",
    contactId: "",
    date: versDateAffichee(jour ?? new Date().toISOString().slice(0, 10)),
    heure: "14:00",
    type: "Visio",
    lieu: "",
    notes: "",
    compteRendu: "",
  };
}

function depuis(entretien: Entretien): EntretienFormInput {
  return {
    candidatureId: entretien.candidatureId,
    contactId: entretien.contactId ?? "",
    date: dateDepuisHorodatage(entretien.dateEntretien),
    heure: heureDepuisHorodatage(entretien.dateEntretien),
    type: entretien.type,
    lieu: entretien.lieu ?? "",
    notes: entretien.notes ?? "",
    compteRendu: entretien.compteRendu ?? "",
  };
}

/**
 * Modale de planification et de modification d'un entretien.
 *
 * Structure reprise de `SPECDESIGN/Modales.dc.html` : sections « Contexte » et
 * « Organisation », date et heure séparées, champ « Lien / lieu » unique.
 *
 * Le compte rendu n'est proposé qu'en modification : il se rédige après l'entretien, et
 * l'offrir à la planification suggérerait qu'il faut l'écrire tout de suite.
 */
export function EntretienFormModal({
  open,
  entretien,
  candidatureId,
  jour,
  busy,
  onClose,
  onSubmit,
}: {
  open: boolean;
  entretien: Entretien | null;
  /** Candidature présélectionnée, quand la modale s'ouvre depuis une fiche. */
  candidatureId?: string | null;
  /** Jour présélectionné, quand la modale s'ouvre depuis une case du calendrier. */
  jour?: string | null;
  busy: boolean;
  onClose: () => void;
  onSubmit: (valeurs: NouvelEntretien) => Promise<unknown>;
}) {
  const contacts = useQuery({
    queryKey: ["contacts", "tous"],
    queryFn: contactService.lister,
    enabled: open,
  });

  const form = useForm<EntretienFormInput, unknown, EntretienFormValues>({
    resolver: zodResolver(entretienFormSchema),
    defaultValues: vide(null, null),
  });

  useEffect(() => {
    if (open) {
      form.reset(entretien ? depuis(entretien) : vide(candidatureId ?? null, jour ?? null));
    }
  }, [open, entretien, candidatureId, jour, form]);

  const enregistrer = form.handleSubmit(async (valeurs) => {
    await onSubmit(valeurs);
    onClose();
  });

  const errors = form.formState.errors;

  return (
    <ModalHost
      open={open}
      icon="event"
      title={entretien ? "Modifier l'entretien" : "Nouvel entretien"}
      subtitle={
        entretien
          ? `${entretien.candidaturePoste ?? ""} — ${entretien.entrepriseNom ?? ""}`
          : "Planifiez un échange et son contexte"
      }
      footerNote="La candidature passera au statut « Entretien »."
      submitLabel={entretien ? "Enregistrer" : "Planifier"}
      busy={busy}
      onClose={onClose}
      onSubmit={() => void enregistrer()}
      width="600px"
    >
      <form onSubmit={(event) => void enregistrer(event)} className="flex flex-col gap-5">
        <fieldset className="flex flex-col gap-3">
          <legend className="text-eyebrow uppercase text-ink-faint">Contexte</legend>
          <div className="flex flex-col gap-4">
            <FormField label="Candidature" required error={errors.candidatureId?.message}>
              {(props) => (
                <Controller
                  control={form.control}
                  name="candidatureId"
                  render={({ field }) => (
                    <CandidaturePicker
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

            <FormField label="Interlocuteur">
              {(props) => (
                <Select {...props} {...form.register("contactId")}>
                  <option value="">Aucun</option>
                  {contacts.data?.map((contact) => (
                    <option key={contact.id} value={contact.id}>
                      {contact.prenom} {contact.nom}
                      {contact.entrepriseNom ? ` — ${contact.entrepriseNom}` : ""}
                    </option>
                  ))}
                </Select>
              )}
            </FormField>
          </div>
        </fieldset>

        <fieldset className="flex flex-col gap-3">
          <legend className="text-eyebrow uppercase text-ink-faint">Organisation</legend>
          <div className="grid grid-cols-3 gap-4">
            <FormField label="Date" required error={errors.date?.message}>
              {(props) => (
                <TextInput
                  {...props}
                  {...form.register("date")}
                  placeholder="JJ-MM-AAAA"
                  inputMode="numeric"
                  invalid={Boolean(errors.date)}
                />
              )}
            </FormField>

            <FormField label="Heure" required error={errors.heure?.message}>
              {(props) => (
                <TextInput
                  {...props}
                  {...form.register("heure")}
                  placeholder="HH:MM"
                  inputMode="numeric"
                  invalid={Boolean(errors.heure)}
                />
              )}
            </FormField>

            <FormField label="Format">
              {(props) => (
                <Select {...props} {...form.register("type")}>
                  {TYPES_ENTRETIEN.map((type) => (
                    <option key={type} value={type}>
                      {type}
                    </option>
                  ))}
                </Select>
              )}
            </FormField>

            <div className="col-span-3">
              <FormField label="Lien ou lieu">
                {(props) => (
                  <TextInput
                    {...props}
                    {...form.register("lieu")}
                    placeholder="https://meet… ou adresse"
                  />
                )}
              </FormField>
            </div>

            <div className="col-span-3">
              <FormField label="Préparation">
                {(props) => (
                  <TextArea
                    {...props}
                    {...form.register("notes")}
                    placeholder="Questions à poser, points à préparer…"
                  />
                )}
              </FormField>
            </div>

            {entretien ? (
              <div className="col-span-3">
                <FormField label="Compte rendu">
                  {(props) => (
                    <TextArea
                      {...props}
                      {...form.register("compteRendu")}
                      rows={4}
                      placeholder="Ce qui s'est dit, impressions, suites à donner…"
                    />
                  )}
                </FormField>
              </div>
            ) : null}
          </div>
        </fieldset>
      </form>
    </ModalHost>
  );
}
