import { zodResolver } from "@hookform/resolvers/zod";
import { useFieldArray, useForm, useWatch } from "react-hook-form";
import { z } from "zod";
import type { Certification } from "@/shared/types/generated/profile";
import { certificationsFormSchema } from "../../../model/profileSchemas";
import { ItemCard, ProfileField, RepeatList } from "./ProfileSectionFields";
import { certificationDefaults, emptyCertification } from "./profileSectionDefaults";

export function ProfileCertificationsForm({ id, value, onSubmit }: { id: string; value: Certification[]; onSubmit: (value: Certification[]) => Promise<unknown> }) {
  const form = useForm<z.input<typeof certificationsFormSchema>, unknown, z.output<typeof certificationsFormSchema>>({ resolver: zodResolver(certificationsFormSchema), defaultValues: certificationDefaults(value) });
  const rows = useFieldArray({ control: form.control, name: "items" });
  const values = useWatch({ control: form.control, name: "items" });
  return <form id={id} onSubmit={(event) => void form.handleSubmit((data) => onSubmit(data.items))(event)}><RepeatList empty="Aucune certification ajoutée" addLabel="Ajouter une certification" onAdd={() => rows.append(emptyCertification())}>{rows.fields.map((field, index) => { const errors = form.formState.errors.items?.[index]; return <ItemCard key={field.id} title={values[index]?.name || `Certification ${index + 1}`} onRemove={() => rows.remove(index)}><div className="grid gap-4 sm:grid-cols-2"><ProfileField required label="Nom" registration={form.register(`items.${index}.name`)} error={errors?.name?.message} /><ProfileField label="Organisme" registration={form.register(`items.${index}.issuer`)} /><ProfileField label="Date" registration={form.register(`items.${index}.date`)} placeholder="AAAA-MM" /><ProfileField label="Lien" type="url" registration={form.register(`items.${index}.url`)} error={errors?.url?.message} /></div></ItemCard>; })}</RepeatList></form>;
}
