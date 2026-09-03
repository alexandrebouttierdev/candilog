import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import type { Profile } from "@/shared/types/generated/profile";
import { ProfileSectionModal } from "../../ProfileSectionModal";
import { ProfileExperiencesForm } from "../ProfileExperiencesForm";
import { ProfileProjectsForm } from "../ProfileProjectsForm";

const profile: Profile = {
  photo: null,
  identity: {
    first_name: "Alex",
    name: "Martin",
    email: "alex@example.test",
    phone: "06 00 00 00 00",
    address: null,
    city: null,
    title: null,
    resume: null,
    linkedin: null,
    github: null,
    website: null,
  },
  experiences: [],
  skills: [],
  education: [],
  languages: [],
  projects: [],
  certifications: [],
};

describe("formulaires de sections Profil", () => {
  it("refuse une expérience sans entreprise", async () => {
    const onSubmit = vi.fn();
    render(
      <ProfileExperiencesForm
        id="experiences"
        value={[{ title: "Développeur", company: "", location: null, start_date: "2025-01", end_date: null, current: true, description: null }]}
        onSubmit={onSubmit}
      />,
    );

    fireEvent.submit(document.getElementById("experiences")!);
    expect(await screen.findByText("L'entreprise est obligatoire")).toBeInTheDocument();
    expect(onSubmit).not.toHaveBeenCalled();
  });

  it("refuse une URL de projet invalide", async () => {
    const onSubmit = vi.fn();
    render(
      <ProfileProjectsForm
        id="projects"
        value={[{ name: "Portfolio", description: null, url: "ftp://example.test", technologies: null }]}
        onSubmit={onSubmit}
      />,
    );

    fireEvent.submit(document.getElementById("projects")!);
    expect(await screen.findByText("Le lien doit commencer par http:// ou https://")).toBeInTheDocument();
    expect(onSubmit).not.toHaveBeenCalled();
  });

  it("ajoute et retire une ligne avec useFieldArray", async () => {
    const user = userEvent.setup();
    render(<ProfileExperiencesForm id="rows" value={[]} onSubmit={vi.fn()} />);

    expect(screen.queryByLabelText(/^Intitulé/)).not.toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Ajouter une expérience" }));
    expect(screen.getByLabelText(/^Intitulé/)).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Supprimer Expérience 1" }));
    expect(screen.queryByLabelText(/^Intitulé/)).not.toBeInTheDocument();
  });

  it("normalise les champs vides en null et soumet le profil complet", async () => {
    const user = userEvent.setup();
    const onSubmit = vi.fn().mockResolvedValue(undefined);
    render(
      <ProfileSectionModal
        section="identity"
        profile={profile}
        busy={false}
        onClose={vi.fn()}
        onSubmit={onSubmit}
      />,
    );

    await user.clear(screen.getByLabelText("Téléphone"));
    await user.click(screen.getByRole("button", { name: "Enregistrer" }));

    await waitFor(() => expect(onSubmit).toHaveBeenCalledTimes(1));
    expect(onSubmit).toHaveBeenCalledWith({
      ...profile,
      identity: { ...profile.identity, phone: null },
    });
  });
});
