import { zodResolver } from "@hookform/resolvers/zod";
import { useFieldArray, useForm, useWatch } from "react-hook-form";
import { z } from "zod";
import type { Language } from "@/shared/types/generated/profile";
import { languagesFormSchema } from "../../../model/profileSchemas";
import { ItemCard, ProfileField, ProfileSelect, RepeatList } from "./ProfileSectionFields";
import { emptyLanguage, languageDefaults } from "./profileSectionDefaults";

export function ProfileLanguagesForm({ id, value, onSubmit }: { id: string; value: Language[]; onSubmit: (value: Language[]) => Promise<unknown> }) {
  const form = useForm<z.input<typeof languagesFormSchema>, unknown, z.output<typeof languagesFormSchema>>({ resolver: zodResolver(languagesFormSchema), defaultValues: languageDefaults(value) });
  const rows = useFieldArray({ control: form.control, name: "items" });
  const values = useWatch({ control: form.control, name: "items" });
  return <form id={id} onSubmit={(event) => void form.handleSubmit((data) => onSubmit(data.items))(event)}><RepeatList empty="Aucune langue ajoutée" addLabel="Ajouter une langue" onAdd={() => rows.append(emptyLanguage())}>{rows.fields.map((field, index) => { const errors = form.formState.errors.items?.[index]; return <ItemCard key={field.id} title={values[index]?.name || `Langue ${index + 1}`} onRemove={() => rows.remove(index)}><div className="grid gap-4 sm:grid-cols-2"><ProfileField required label="Langue" registration={form.register(`items.${index}.name`)} error={errors?.name?.message} /><ProfileSelect label="Niveau" registration={form.register(`items.${index}.level`)} error={errors?.level?.message}><option value="">Choisir…</option><option>Débutant</option><option>Intermédiaire</option><option>Professionnel</option><option>Courant</option><option>Langue maternelle</option></ProfileSelect></div></ItemCard>; })}</RepeatList></form>;
}
