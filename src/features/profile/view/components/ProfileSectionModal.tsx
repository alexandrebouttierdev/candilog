import type { Identity, Profile } from "@/shared/types/generated/profile";
import { ModalHost } from "@/shared/ui";
import { ProfileCertificationsForm } from "./profile-sections/ProfileCertificationsForm";
import { ProfileEducationForm } from "./profile-sections/ProfileEducationForm";
import { ProfileExperiencesForm } from "./profile-sections/ProfileExperiencesForm";
import { ProfileIdentityForm } from "./profile-sections/ProfileIdentityForm";
import { ProfileInterestsForm } from "./profile-sections/ProfileInterestsForm";
import { ProfileLanguagesForm } from "./profile-sections/ProfileLanguagesForm";
import { ProfileProjectsForm } from "./profile-sections/ProfileProjectsForm";
import { ProfileSkillsForm } from "./profile-sections/ProfileSkillsForm";

export type ProfileSection =
  | "identity"
  | "objective"
  | "online"
  | "experiences"
  | "skills"
  | "education"
  | "languages"
  | "projects"
  | "certifications"
  | "interests";

const META: Record<ProfileSection, { title: string; subtitle: string }> = {
  identity: { title: "Identité", subtitle: "Coordonnées et informations personnelles" },
  objective: { title: "Objectif professionnel", subtitle: "Titre, résumé et recherche" },
  online: { title: "Présence en ligne", subtitle: "Liens professionnels" },
  experiences: { title: "Expériences", subtitle: "Décrivez les étapes utiles de votre parcours" },
  skills: { title: "Compétences", subtitle: "Ajoutez vos savoir-faire principaux" },
  education: { title: "Formations", subtitle: "Diplômes et parcours de formation" },
  languages: { title: "Langues", subtitle: "Indiquez votre niveau de pratique" },
  projects: { title: "Projets", subtitle: "Valorisez vos réalisations personnelles" },
  certifications: { title: "Certifications", subtitle: "Ajoutez vos qualifications reconnues" },
  interests: { title: "Centres d'intérêts", subtitle: "Facultatif — hobbies et centres d'intérêts" },
};

const IDENTITY_SECTIONS = new Set<ProfileSection>(["identity", "objective", "online"]);

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
  const saveList = async <K extends Exclude<ProfileSection, "identity" | "objective" | "online">>(
    key: K,
    value: Profile[K],
  ) => {
    await onSubmit({ ...profile, [key]: value });
    onClose();
  };
  const saveIdentity = async (value: Identity) => {
    await onSubmit({ ...profile, identity: value });
    onClose();
  };

  return (
    <ModalHost
      open
      title={meta.title}
      subtitle={meta.subtitle}
      footer_note="Les informations sont utilisées dans votre CV."
      busy={busy}
      onClose={onClose}
      onSubmit={() => {
        const form = document.getElementById(formId);
        if (form instanceof HTMLFormElement) form.requestSubmit();
      }}
      width={IDENTITY_SECTIONS.has(section) ? "720px" : "760px"}
    >
      {section === "identity" || section === "objective" || section === "online" ? (
        <ProfileIdentityForm
          id={formId}
          section={section}
          value={profile.identity}
          onSubmit={saveIdentity}
        />
      ) : null}
      {section === "experiences" ? <ProfileExperiencesForm id={formId} value={profile.experiences} onSubmit={(value) => saveList("experiences", value)} /> : null}
      {section === "skills" ? <ProfileSkillsForm id={formId} value={profile.skills} onSubmit={(value) => saveList("skills", value)} /> : null}
      {section === "education" ? <ProfileEducationForm id={formId} value={profile.education} onSubmit={(value) => saveList("education", value)} /> : null}
      {section === "languages" ? <ProfileLanguagesForm id={formId} value={profile.languages} onSubmit={(value) => saveList("languages", value)} /> : null}
      {section === "projects" ? <ProfileProjectsForm id={formId} value={profile.projects} onSubmit={(value) => saveList("projects", value)} /> : null}
      {section === "certifications" ? <ProfileCertificationsForm id={formId} value={profile.certifications} onSubmit={(value) => saveList("certifications", value)} /> : null}
      {section === "interests" ? <ProfileInterestsForm id={formId} value={profile.interests} onSubmit={(value) => saveList("interests", value)} /> : null}
    </ModalHost>
  );
}
