import { useEffect } from "react";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import {
  contactFormSchema,
  type ContactFormInput,
  type ContactFormValues,
} from "../../model/schemas/contact-form.schema";
import { Roles } from "../../model/roles";
import type { Contact, NewContact } from "../../services/contactService";
import { companyService } from "@/features/companies/services/companyService";
import { useQuery } from "@tanstack/react-query";
import { FormField, ModalHost, Select, TextArea, TextInput } from "@/shared/ui";

const VIDE: ContactFormInput = {
  first_name: "",
  name: "",
  email: "",
  phone: "",
  company_id: "",
  job_title: "",
  tracking_role: "",
  linkedin: "",
  notes: "",
};

function from(contact: Contact): ContactFormInput {
  return {
    first_name: contact.first_name,
    name: contact.name,
    email: contact.email ?? "",
    phone: contact.phone ?? "",
    company_id: contact.company_id ?? "",
    job_title: contact.job_title ?? "",
    tracking_role: contact.tracking_role ?? "",
    linkedin: contact.linkedin ?? "",
    notes: contact.notes ?? "",
  };
}

/**
 * Modale de création et de modification d'un contact.
 *
 * Structure reprise de `SPECDESIGN/Modales.dc.html` : « Identité » puis « Context
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
  onSubmit: (values: NewContact) => Promise<unknown>;
}) {
  // Le sélecteur d'entreprise charge le répertoire complet, sans pagination : il alimente
  // une liste déroulante, et un `select` natif ne saurait pas demander la page suivante.
  // Un EntityPicker paginé sera introduit avec les candidatures, dont le répertoire
  // d'entreprises est le même mais l'usage plus intensif.
  const companies = useQuery({
    queryKey: ["entreprises", "toutes"],
    queryFn: companyService.list,
    enabled: open,
  });

  const form = useForm<ContactFormInput, unknown, ContactFormValues>({
    resolver: zodResolver(contactFormSchema),
    defaultValues: VIDE,
  });

  useEffect(() => {
    if (open) form.reset(contact ? from(contact) : VIDE);
  }, [open, contact, form]);

  const save = form.handleSubmit(async (values) => {
    await onSubmit(values);
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
          ? `${contact.first_name} ${contact.name}`
          : "Ajoutez un interlocuteur à votre réseau"
      }
      footer_note="Le prénom et le nom sont obligatoires."
      busy={busy}
      onClose={onClose}
      onSubmit={() => void save()}
      width="620px"
    >
      <form onSubmit={(event) => void save(event)} className="flex flex-col gap-5">
        <fieldset className="flex flex-col gap-3">
          <legend className="text-eyebrow uppercase text-ink-faint">Identité</legend>
          <div className="grid grid-cols-2 gap-4">
            <FormField label="Prénom" required error={errors.first_name?.message}>
              {(props) => (
                <TextInput
                  {...props}
                  {...form.register("first_name")}
                  placeholder="Camille"
                  invalid={Boolean(errors.first_name)}
                />
              )}
            </FormField>

            <FormField label="Nom" required error={errors.name?.message}>
              {(props) => (
                <TextInput
                  {...props}
                  {...form.register("name")}
                  placeholder="Rivet"
                  invalid={Boolean(errors.name)}
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
                <TextInput {...props} {...form.register("phone")} placeholder="02 99 14 88 05" />
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
                <Select {...props} {...form.register("company_id")}>
                  <option value="">Aucune</option>
                  {companies.data?.map((company) => (
                    <option key={company.id} value={company.id}>
                      {company.name}
                    </option>
                  ))}
                </Select>
              )}
            </FormField>

            <FormField label="Poste">
              {(props) => (
                <TextInput {...props} {...form.register("job_title")} placeholder="Lead Frontend" />
              )}
            </FormField>

            <FormField label="Rôle dans le suivi">
              {(props) => (
                <Select {...props} {...form.register("tracking_role")}>
                  <option value="">Non renseigné</option>
                  {Roles.map((role) => (
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
