import { zodResolver } from "@hookform/resolvers/zod";
import { useFieldArray, useForm, useWatch } from "react-hook-form";
import { z } from "zod";
import type { Education } from "@/shared/types/generated/profile";
import { educationFormSchema } from "../../../model/profileSchemas";
import { ItemCard, ProfileArea, ProfileField, RepeatList } from "./ProfileSectionFields";
import { educationDefaults, emptyEducation } from "./profileSectionDefaults";

export function ProfileEducationForm({ id, value, onSubmit }: { id: string; value: Education[]; onSubmit: (value: Education[]) => Promise<unknown> }) {
  const form = useForm<z.input<typeof educationFormSchema>, unknown, z.output<typeof educationFormSchema>>({ resolver: zodResolver(educationFormSchema), defaultValues: educationDefaults(value) });
  const rows = useFieldArray({ control: form.control, name: "items" });
  const values = useWatch({ control: form.control, name: "items" });
  return <form id={id} onSubmit={(event) => void form.handleSubmit((data) => onSubmit(data.items))(event)}><RepeatList empty="Aucune formation ajoutée" addLabel="Ajouter une formation" onAdd={() => rows.append(emptyEducation())}>{rows.fields.map((field, index) => { const errors = form.formState.errors.items?.[index]; return <ItemCard key={field.id} title={values[index]?.degree || `Formation ${index + 1}`} onRemove={() => rows.remove(index)}><div className="grid gap-4 sm:grid-cols-2"><ProfileField required label="Diplôme" registration={form.register(`items.${index}.degree`)} error={errors?.degree?.message} /><ProfileField required label="Établissement" registration={form.register(`items.${index}.school`)} error={errors?.school?.message} /><ProfileField label="Lieu" registration={form.register(`items.${index}.location`)} error={errors?.location?.message} /><div className="grid grid-cols-2 gap-3"><ProfileField label="Début" registration={form.register(`items.${index}.start_date`)} /><ProfileField label="Fin" registration={form.register(`items.${index}.end_date`)} /></div><div className="sm:col-span-2"><ProfileArea label="Description" registration={form.register(`items.${index}.description`)} /></div></div></ItemCard>; })}</RepeatList></form>;
}
