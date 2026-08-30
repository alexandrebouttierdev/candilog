import { zodResolver } from "@hookform/resolvers/zod";
import { useFieldArray, useForm, useWatch } from "react-hook-form";
import { z } from "zod";
import type { Experience } from "@/shared/types/generated/profile";
import { experiencesFormSchema } from "../../../model/profileSchemas";
import { ItemCard, ProfileArea, ProfileField, RepeatList } from "./ProfileSectionFields";
import { emptyExperience, experienceDefaults } from "./profileSectionDefaults";

export function ProfileExperiencesForm({ id, value, onSubmit }: { id: string; value: Experience[]; onSubmit: (value: Experience[]) => Promise<unknown> }) {
  const form = useForm<z.input<typeof experiencesFormSchema>, unknown, z.output<typeof experiencesFormSchema>>({ resolver: zodResolver(experiencesFormSchema), defaultValues: experienceDefaults(value) });
  const rows = useFieldArray({ control: form.control, name: "items" });
  const values = useWatch({ control: form.control, name: "items" });
  return (
    <form id={id} onSubmit={(event) => void form.handleSubmit((data) => onSubmit(data.items))(event)}>
      <RepeatList empty="Aucune expérience ajoutée" addLabel="Ajouter une expérience" onAdd={() => rows.append(emptyExperience())}>
        {rows.fields.map((field, index) => {
          const errors = form.formState.errors.items?.[index];
          const current = form.register(`items.${index}.current`);
          return (
            <ItemCard key={field.id} title={values[index]?.title || `Expérience ${index + 1}`} onRemove={() => rows.remove(index)}>
              <div className="grid gap-4 sm:grid-cols-2">
                <ProfileField required label="Intitulé" registration={form.register(`items.${index}.title`)} error={errors?.title?.message} />
                <ProfileField required label="Entreprise" registration={form.register(`items.${index}.company`)} error={errors?.company?.message} />
                <ProfileField label="Lieu" registration={form.register(`items.${index}.location`)} error={errors?.location?.message} />
                <ProfileField required label="Début" registration={form.register(`items.${index}.start_date`)} error={errors?.start_date?.message} placeholder="AAAA-MM" />
                <ProfileField label="Fin" registration={form.register(`items.${index}.end_date`)} disabled={Boolean(values[index]?.current)} error={errors?.end_date?.message} placeholder="AAAA-MM" />
                <label className="flex min-h-field items-center gap-2 self-end text-body text-ink-muted"><input type="checkbox" {...current} onChange={(event) => { void current.onChange(event); if (event.target.checked) form.setValue(`items.${index}.end_date`, ""); }} /> Poste actuel</label>
                <div className="sm:col-span-2"><ProfileArea label="Description" registration={form.register(`items.${index}.description`)} error={errors?.description?.message} /></div>
              </div>
            </ItemCard>
          );
        })}
      </RepeatList>
    </form>
  );
}
