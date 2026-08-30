import { zodResolver } from "@hookform/resolvers/zod";
import { useFieldArray, useForm } from "react-hook-form";
import { z } from "zod";
import type { Skill } from "@/shared/types/generated/profile";
import { skillsFormSchema } from "../../../model/profileSchemas";
import { ItemCard, ProfileField, RepeatList } from "./ProfileSectionFields";
import { emptySkill, skillDefaults } from "./profileSectionDefaults";

export function ProfileSkillsForm({ id, value, onSubmit }: { id: string; value: Skill[]; onSubmit: (value: Skill[]) => Promise<unknown> }) {
  const form = useForm<z.input<typeof skillsFormSchema>, unknown, z.output<typeof skillsFormSchema>>({ resolver: zodResolver(skillsFormSchema), defaultValues: skillDefaults(value) });
  const rows = useFieldArray({ control: form.control, name: "items" });
  return <form id={id} onSubmit={(event) => void form.handleSubmit((data) => onSubmit(data.items))(event)}><RepeatList empty="Aucune compétence ajoutée" addLabel="Ajouter une compétence" onAdd={() => rows.append(emptySkill())}>{rows.fields.map((field, index) => <ItemCard key={field.id} title={`Compétence ${index + 1}`} onRemove={() => rows.remove(index)}><ProfileField required label="Nom" registration={form.register(`items.${index}.name`)} error={form.formState.errors.items?.[index]?.name?.message} /></ItemCard>)}</RepeatList></form>;
}
