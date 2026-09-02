import type { Profile } from "@/shared/types/generated/profile";
import { ModalHost } from "@/shared/ui";
import { ProfileCertificationsForm } from "./profile-sections/ProfileCertificationsForm";
import { ProfileEducationForm } from "./profile-sections/ProfileEducationForm";
import { ProfileExperiencesForm } from "./profile-sections/ProfileExperiencesForm";
import { ProfileIdentityForm } from "./profile-sections/ProfileIdentityForm";
import { ProfileLanguagesForm } from "./profile-sections/ProfileLanguagesForm";
import { ProfileProjectsForm } from "./profile-sections/ProfileProjectsForm";
import { ProfileSkillsForm } from "./profile-sections/ProfileSkillsForm";
import type { IconName } from "@/shared/ui/icon-names";

export type ProfileSection =
  | "identity"
  | "experiences"
  | "skills"
  | "education"
  | "languages"
  | "projects"
  | "certifications";

const META: Record<ProfileSection, { icon: IconName; title: string; subtitle: string }> = {
  identity: { icon: "person", title: "Identité et objectif", subtitle: "Présentez votre projet professionnel" },
  experiences: { icon: "work_history", title: "Expériences", subtitle: "Décrivez les étapes utiles de votre parcours" },
  skills: { icon: "psychology", title: "Compétences", subtitle: "Ajoutez vos savoir-faire principaux" },
  education: { icon: "school", title: "Formations", subtitle: "Diplômes et parcours de formation" },
  languages: { icon: "translate", title: "Langues", subtitle: "Indiquez votre niveau de pratique" },
  projects: { icon: "rocket_launch", title: "Projets", subtitle: "Valorisez vos réalisations personnelles" },
  certifications: { icon: "workspace_premium", title: "Certifications", subtitle: "Ajoutez vos qualifications reconnues" },
};

/** Coquille de navigation : chaque section possède son propre formulaire RHF + Zod. */
export function ProfileSectionModal({
  section,
  profile,
  busy,
  onClose,
  onSubmit,
}: {
  section: ProfileSection;
  profile: Profile;
  busy: boolean;
  onClose: () => void;
  onSubmit: (profile: Profile) => Promise<unknown>;
}) {
  const meta = META[section];
  const formId = `profile-${section}-form`;
  const save = async <K extends ProfileSection>(key: K, value: Profile[K]) => {
    await onSubmit({ ...profile, [key]: value });
    onClose();
  };

  return (
    <ModalHost
      open
      icon={meta.icon}
      title={meta.title}
      subtitle={meta.subtitle}
      footer_note="Les informations sont utilisées dans votre CV."
      busy={busy}
      onClose={onClose}
      onSubmit={() => {
        const form = document.getElementById(formId);
        if (form instanceof HTMLFormElement) form.requestSubmit();
      }}
      width={section === "identity" ? "720px" : "760px"}
    >
      {section === "identity" ? <ProfileIdentityForm id={formId} value={profile.identity} onSubmit={(value) => save("identity", value)} /> : null}
      {section === "experiences" ? <ProfileExperiencesForm id={formId} value={profile.experiences} onSubmit={(value) => save("experiences", value)} /> : null}
      {section === "skills" ? <ProfileSkillsForm id={formId} value={profile.skills} onSubmit={(value) => save("skills", value)} /> : null}
      {section === "education" ? <ProfileEducationForm id={formId} value={profile.education} onSubmit={(value) => save("education", value)} /> : null}
      {section === "languages" ? <ProfileLanguagesForm id={formId} value={profile.languages} onSubmit={(value) => save("languages", value)} /> : null}
      {section === "projects" ? <ProfileProjectsForm id={formId} value={profile.projects} onSubmit={(value) => save("projects", value)} /> : null}
      {section === "certifications" ? <ProfileCertificationsForm id={formId} value={profile.certifications} onSubmit={(value) => save("certifications", value)} /> : null}
    </ModalHost>
  );
}
