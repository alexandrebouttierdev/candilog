import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";
import { z } from "zod";
import type { Identity } from "@/shared/types/generated/profile";
import { identitySchema } from "../../../model/profileSchemas";
import { ProfileArea, ProfileField } from "./ProfileSectionFields";
import { identityDefaults } from "./profileSectionDefaults";

export function ProfileIdentityForm({
  id,
  value,
  onSubmit,
}: {
  id: string;
  value: Identity;
  onSubmit: (value: Identity) => Promise<unknown>;
}) {
  const form = useForm<z.input<typeof identitySchema>, unknown, z.output<typeof identitySchema>>({
    resolver: zodResolver(identitySchema),
    defaultValues: identityDefaults(value),
  });
  const errors = form.formState.errors;
  const message = (field: keyof typeof errors) => errors[field]?.message?.toString();

  return (
    <form id={id} onSubmit={(event) => void form.handleSubmit(onSubmit)(event)} className="flex flex-col gap-4">
      <fieldset className="grid gap-4 sm:grid-cols-2">
        <legend className="sr-only">Coordonnées</legend>
        <ProfileField label="Prénom" registration={form.register("first_name")} error={message("first_name")} />
        <ProfileField label="Nom" registration={form.register("name")} error={message("name")} />
        <ProfileField label="E-mail" type="email" registration={form.register("email")} error={message("email")} />
        <ProfileField label="Téléphone" type="tel" registration={form.register("phone")} error={message("phone")} />
        <div className="sm:col-span-2"><ProfileField label="Ville" registration={form.register("city")} error={message("city")} /></div>
      </fieldset>
      <fieldset className="grid gap-4 border-t border-line pt-4">
        <legend className="mb-3 text-eyebrow uppercase text-ink-faint">Objectif professionnel</legend>
        <ProfileField label="Titre ou poste visé" registration={form.register("title")} error={message("title")} placeholder="Product designer — mobilité durable" />
        <ProfileArea label="Présentation" rows={5} registration={form.register("resume")} error={message("resume")} help="En quelques phrases : votre expérience, vos forces et ce que vous recherchez." />
      </fieldset>
      <fieldset className="grid gap-4 border-t border-line pt-4 sm:grid-cols-2">
        <legend className="mb-3 text-eyebrow uppercase text-ink-faint sm:col-span-2">Présence en ligne</legend>
        <ProfileField label="LinkedIn" type="url" registration={form.register("linkedin")} error={message("linkedin")} placeholder="https://linkedin.com/in/…" />
        <ProfileField label="GitHub" type="url" registration={form.register("github")} error={message("github")} placeholder="https://github.com/…" />
        <div className="sm:col-span-2"><ProfileField label="Site web" type="url" registration={form.register("website")} error={message("website")} placeholder="https://…" /></div>
      </fieldset>
    </form>
  );
}
