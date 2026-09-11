import { zodResolver } from "@hookform/resolvers/zod";
import { useFieldArray, useForm } from "react-hook-form";
import { z } from "zod";
import type { Interest } from "@/shared/types/generated/profile";
import { interestsFormSchema } from "../../../model/profileSchemas";
import { ItemCard, ProfileField, RepeatList } from "./ProfileSectionFields";
import { emptyInterest, interestDefaults } from "./profileSectionDefaults";

export function ProfileInterestsForm({
  id,
  value,
  onSubmit,
}: {
  id: string;
  value: Interest[];
  onSubmit: (value: Interest[]) => Promise<unknown>;
}) {
  const form = useForm<
    z.input<typeof interestsFormSchema>,
    unknown,
    z.output<typeof interestsFormSchema>
  >({
    resolver: zodResolver(interestsFormSchema),
    defaultValues: interestDefaults(value),
  });
  const rows = useFieldArray({ control: form.control, name: "items" });
  return (
    <form
      id={id}
      onSubmit={(event) => void form.handleSubmit((data) => onSubmit(data.items))(event)}
    >
      <RepeatList
        empty="Aucun centre d'intérêt ajouté"
        addLabel="Ajouter un centre d'intérêt"
        onAdd={() => rows.append(emptyInterest())}
      >
        {rows.fields.map((field, index) => (
          <ItemCard
            key={field.id}
            title={`Centre d'intérêt ${index + 1}`}
            onRemove={() => rows.remove(index)}
          >
            <ProfileField
              required
              label="Libellé"
              registration={form.register(`items.${index}.name`)}
              error={form.formState.errors.items?.[index]?.name?.message}
            />
          </ItemCard>
        ))}
      </RepeatList>
    </form>
  );
}
