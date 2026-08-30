import { zodResolver } from "@hookform/resolvers/zod";
import { useFieldArray, useForm, useWatch } from "react-hook-form";
import { z } from "zod";
import type { Project } from "@/shared/types/generated/profile";
import { projectsFormSchema } from "../../../model/profileSchemas";
import { ItemCard, ProfileArea, ProfileField, RepeatList } from "./ProfileSectionFields";
import { emptyProject, projectDefaults } from "./profileSectionDefaults";

export function ProfileProjectsForm({ id, value, onSubmit }: { id: string; value: Project[]; onSubmit: (value: Project[]) => Promise<unknown> }) {
  const form = useForm<z.input<typeof projectsFormSchema>, unknown, z.output<typeof projectsFormSchema>>({ resolver: zodResolver(projectsFormSchema), defaultValues: projectDefaults(value) });
  const rows = useFieldArray({ control: form.control, name: "items" });
  const values = useWatch({ control: form.control, name: "items" });
  return <form id={id} onSubmit={(event) => void form.handleSubmit((data) => onSubmit(data.items))(event)}><RepeatList empty="Aucun projet ajouté" addLabel="Ajouter un projet" onAdd={() => rows.append(emptyProject())}>{rows.fields.map((field, index) => { const errors = form.formState.errors.items?.[index]; return <ItemCard key={field.id} title={values[index]?.name || `Projet ${index + 1}`} onRemove={() => rows.remove(index)}><div className="grid gap-4"><ProfileField required label="Nom" registration={form.register(`items.${index}.name`)} error={errors?.name?.message} /><ProfileArea label="Description" registration={form.register(`items.${index}.description`)} /><div className="grid gap-4 sm:grid-cols-2"><ProfileField label="Lien" type="url" registration={form.register(`items.${index}.url`)} error={errors?.url?.message} /><ProfileField label="Technologies" registration={form.register(`items.${index}.technologies`)} /></div></div></ItemCard>; })}</RepeatList></form>;
}
