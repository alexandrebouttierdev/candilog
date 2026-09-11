import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";
import type { Identity } from "@/shared/types/generated/profile";
import { identitySchema } from "../../../model/profileSchemas";
import { ProfileArea, ProfileField } from "./ProfileSectionFields";
import { identityDefaults } from "./profileSectionDefaults";

export type IdentityFormSection = "identity" | "objective" | "online";

const identityOnlySchema = identitySchema.pick({
  first_name: true,
  name: true,
  email: true,
  phone: true,
  address: true,
  city: true,
  birth_date: true,
  age: true,
});

const objectiveOnlySchema = identitySchema.pick({
  title: true,
  resume: true,
  availability: true,
  desired_contracts: true,
});

const onlineOnlySchema = identitySchema.pick({
  linkedin: true,
  github: true,
  website: true,
});

export function ProfileIdentityForm({
  id,
  section,
  value,
  onSubmit,
}: {
  id: string;
  section: IdentityFormSection;
  value: Identity;
  onSubmit: (value: Identity) => Promise<unknown>;
}) {
  if (section === "identity") {
    return <IdentityCoordsForm id={id} value={value} onSubmit={onSubmit} />;
  }
  if (section === "objective") {
    return <IdentityObjectiveForm id={id} value={value} onSubmit={onSubmit} />;
  }
  return <IdentityOnlineForm id={id} value={value} onSubmit={onSubmit} />;
}

function IdentityCoordsForm({
  id,
  value,
  onSubmit,
}: {
  id: string;
  value: Identity;
  onSubmit: (value: Identity) => Promise<unknown>;
}) {
  const defaults = identityDefaults(value);
  const form = useForm({
    resolver: zodResolver(identityOnlySchema),
    defaultValues: {
      first_name: defaults.first_name,
      name: defaults.name,
      email: defaults.email,
      phone: defaults.phone,
      address: defaults.address,
      city: defaults.city,
      birth_date: defaults.birth_date,
      age: defaults.age,
    },
  });
  const errors = form.formState.errors;
  const message = (field: keyof typeof errors) => errors[field]?.message?.toString();

  return (
    <form
      id={id}
      onSubmit={(event) =>
        void form.handleSubmit(async (data) => {
          await onSubmit({
            ...value,
            first_name: data.first_name,
            name: data.name,
            email: data.email,
            phone: data.phone,
            address: data.address,
            city: data.city,
            birth_date: data.birth_date,
            age: data.age,
          });
        })(event)
      }
      className="flex flex-col gap-4"
    >
      <fieldset className="grid gap-4 sm:grid-cols-2">
        <legend className="sr-only">Coordonnées</legend>
        <ProfileField label="Prénom" registration={form.register("first_name")} error={message("first_name")} />
        <ProfileField label="Nom" registration={form.register("name")} error={message("name")} />
        <ProfileField label="E-mail" type="email" registration={form.register("email")} error={message("email")} />
        <ProfileField label="Téléphone" type="tel" registration={form.register("phone")} error={message("phone")} />
        <div className="sm:col-span-2">
          <ProfileField label="Adresse" registration={form.register("address")} error={message("address")} placeholder="14 rue Saint-Melaine" />
        </div>
        <div className="sm:col-span-2">
          <ProfileField label="Ville" registration={form.register("city")} error={message("city")} />
        </div>
        <ProfileField label="Date de naissance" registration={form.register("birth_date")} error={message("birth_date")} placeholder="14 avril 1992" />
        <ProfileField label="Âge" registration={form.register("age")} error={message("age")} placeholder="34" />
      </fieldset>
    </form>
  );
}

function IdentityObjectiveForm({
  id,
  value,
  onSubmit,
}: {
  id: string;
  value: Identity;
  onSubmit: (value: Identity) => Promise<unknown>;
}) {
  const defaults = identityDefaults(value);
  const form = useForm({
    resolver: zodResolver(objectiveOnlySchema),
    defaultValues: {
      title: defaults.title,
      resume: defaults.resume,
      availability: defaults.availability,
      desired_contracts: defaults.desired_contracts,
    },
  });
  const errors = form.formState.errors;
  const message = (field: keyof typeof errors) => errors[field]?.message?.toString();

  return (
    <form
      id={id}
      onSubmit={(event) =>
        void form.handleSubmit(async (data) => {
          await onSubmit({
            ...value,
            title: data.title,
            resume: data.resume,
            availability: data.availability,
            desired_contracts: data.desired_contracts,
          });
        })(event)
      }
      className="flex flex-col gap-4"
    >
      <fieldset className="grid gap-4">
        <legend className="sr-only">Objectif professionnel</legend>
        <ProfileField label="Titre ou poste visé" registration={form.register("title")} error={message("title")} placeholder="Product designer — mobilité durable" />
        <ProfileArea label="Résumé du profil" rows={5} registration={form.register("resume")} error={message("resume")} help="Facultatif. En quelques phrases : votre expérience, vos forces et ce que vous recherchez." />
        <ProfileField label="Disponibilité" registration={form.register("availability")} error={message("availability")} placeholder="Sous 1 mois" />
        <ProfileField label="Contrats recherchés" registration={form.register("desired_contracts")} error={message("desired_contracts")} placeholder="CDI, CDD, Freelance" />
      </fieldset>
    </form>
  );
}

function IdentityOnlineForm({
  id,
  value,
  onSubmit,
}: {
  id: string;
  value: Identity;
  onSubmit: (value: Identity) => Promise<unknown>;
}) {
  const defaults = identityDefaults(value);
  const form = useForm({
    resolver: zodResolver(onlineOnlySchema),
    defaultValues: {
      linkedin: defaults.linkedin,
      github: defaults.github,
      website: defaults.website,
    },
  });
  const errors = form.formState.errors;
  const message = (field: keyof typeof errors) => errors[field]?.message?.toString();

  return (
    <form
      id={id}
      onSubmit={(event) =>
        void form.handleSubmit(async (data) => {
          await onSubmit({
            ...value,
            linkedin: data.linkedin,
            github: data.github,
            website: data.website,
          });
        })(event)
      }
      className="flex flex-col gap-4"
    >
      <fieldset className="grid gap-4 sm:grid-cols-2">
        <legend className="sr-only">Présence en ligne</legend>
        <ProfileField label="LinkedIn" type="url" registration={form.register("linkedin")} error={message("linkedin")} placeholder="https://linkedin.com/in/…" />
        <ProfileField label="GitHub" type="url" registration={form.register("github")} error={message("github")} placeholder="https://github.com/…" />
        <div className="sm:col-span-2">
          <ProfileField label="Site web" type="url" registration={form.register("website")} error={message("website")} placeholder="https://…" />
        </div>
      </fieldset>
    </form>
  );
}
