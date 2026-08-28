import { useEffect } from "react";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import {
  contactFormSchema,
  type ContactFormInput,
  type ContactFormValues,
} from "../../model/schemas/contact-form.schema";
import { ROLES } from "../../model/roles";
import type { Contact, NouveauContact } from "../../services/contact.service";
import { entrepriseService } from "@/features/entreprises/services/entreprise.service";
import { useQuery } from "@tanstack/react-query";
import { FormField, ModalHost, Select, TextArea, TextInput } from "@/shared/ui";

const VIDE: ContactFormInput = {
  prenom: "",
  nom: "",
  email: "",
  telephone: "",
  entrepriseId: "",
  poste: "",
  roleSuivi: "",
  linkedin: "",
  notes: "",
};

function depuis(contact: Contact): ContactFormInput {
  return {
    prenom: contact.prenom,
    nom: contact.nom,
    email: contact.email ?? "",
    telephone: contact.telephone ?? "",
    entrepriseId: contact.entrepriseId ?? "",
    poste: contact.poste ?? "",
    roleSuivi: contact.roleSuivi ?? "",
    linkedin: contact.linkedin ?? "",
    notes: contact.notes ?? "",
  };
}

/**
 * Modale de création et de modification d'un contact.
 *
 * Structure reprise de `SPECDESIGN/Modales.dc.html` : « Identité » puis « Contexte
 * professionnel ».
 */
export function ContactFormModal({
  open,
  contact,
  busy,
  onClose,
  onSubmit,
}: {
  open: boolean;
  contact: Contact | null;
  busy: boolean;
  onClose: () => void;
  onSubmit: (valeurs: NouveauContact) => Promise<unknown>;
}) {
  // Le sélecteur d'entreprise charge le répertoire complet, sans pagination : il alimente
  // une liste déroulante, et un `select` natif ne saurait pas demander la page suivante.
  // Un EntityPicker paginé sera introduit avec les candidatures, dont le répertoire
  // d'entreprises est le même mais l'usage plus intensif.
  const entreprises = useQuery({
    queryKey: ["entreprises", "toutes"],
    queryFn: entrepriseService.lister,
    enabled: open,
  });

  const form = useForm<ContactFormInput, unknown, ContactFormValues>({
    resolver: zodResolver(contactFormSchema),
    defaultValues: VIDE,
  });

  useEffect(() => {
    if (open) form.reset(contact ? depuis(contact) : VIDE);
  }, [open, contact, form]);

  const enregistrer = form.handleSubmit(async (valeurs) => {
    await onSubmit(valeurs);
    onClose();
  });

  const errors = form.formState.errors;

  return (
    <ModalHost
      open={open}
      icon="person_add"
      title={contact ? "Modifier le contact" : "Nouveau contact"}
      subtitle={
        contact
          ? `${contact.prenom} ${contact.nom}`
          : "Ajoutez un interlocuteur à votre réseau"
      }
      footerNote="Le prénom et le nom sont obligatoires."
      busy={busy}
      onClose={onClose}
      onSubmit={() => void enregistrer()}
      width="620px"
    >
      <form onSubmit={(event) => void enregistrer(event)} className="flex flex-col gap-5">
        <fieldset className="flex flex-col gap-3">
          <legend className="text-eyebrow uppercase text-ink-faint">Identité</legend>
          <div className="grid grid-cols-2 gap-4">
            <FormField label="Prénom" required error={errors.prenom?.message}>
              {(props) => (
                <TextInput
                  {...props}
                  {...form.register("prenom")}
                  placeholder="Camille"
                  invalid={Boolean(errors.prenom)}
                />
              )}
            </FormField>

            <FormField label="Nom" required error={errors.nom?.message}>
              {(props) => (
                <TextInput
                  {...props}
                  {...form.register("nom")}
                  placeholder="Rivet"
                  invalid={Boolean(errors.nom)}
                />
              )}
            </FormField>

            <FormField label="E-mail" error={errors.email?.message}>
              {(props) => (
                <TextInput
                  {...props}
                  {...form.register("email")}
                  type="email"
                  placeholder="camille.rivet@exemple.fr"
                  invalid={Boolean(errors.email)}
                />
              )}
            </FormField>

            <FormField label="Téléphone">
              {(props) => (
                <TextInput {...props} {...form.register("telephone")} placeholder="02 99 14 88 05" />
              )}
            </FormField>
          </div>
        </fieldset>

        <fieldset className="flex flex-col gap-3">
          <legend className="text-eyebrow uppercase text-ink-faint">
            Contexte professionnel
          </legend>
          <div className="grid grid-cols-2 gap-4">
            <FormField label="Entreprise">
              {(props) => (
                <Select {...props} {...form.register("entrepriseId")}>
                  <option value="">Aucune</option>
                  {entreprises.data?.map((entreprise) => (
                    <option key={entreprise.id} value={entreprise.id}>
                      {entreprise.nom}
                    </option>
                  ))}
                </Select>
              )}
            </FormField>

            <FormField label="Poste">
              {(props) => (
                <TextInput {...props} {...form.register("poste")} placeholder="Lead Frontend" />
              )}
            </FormField>

            <FormField label="Rôle dans le suivi">
              {(props) => (
                <Select {...props} {...form.register("roleSuivi")}>
                  <option value="">Non renseigné</option>
                  {ROLES.map((role) => (
                    <option key={role} value={role}>
                      {role}
                    </option>
                  ))}
                </Select>
              )}
            </FormField>

            <FormField label="LinkedIn" error={errors.linkedin?.message}>
              {(props) => (
                <TextInput
                  {...props}
                  {...form.register("linkedin")}
                  placeholder="https://linkedin.com/in/…"
                  invalid={Boolean(errors.linkedin)}
                />
              )}
            </FormField>

            <div className="col-span-2">
              <FormField label="Notes">
                {(props) => (
                  <TextArea
                    {...props}
                    {...form.register("notes")}
                    placeholder="Contexte, sujets abordés, points à retenir…"
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
