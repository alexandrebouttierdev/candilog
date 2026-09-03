import { fireEvent, render, screen } from "@testing-library/react";
import { vi } from "vitest";
import { workspaceFixture } from "../../../model/resumeWorkspace";
import { ResumePaper } from "../ResumePaper";

describe("ResumePaper", () => {
  it("rend les sections du template et omet celles qui sont vides", () => {
    render(<ResumePaper workspace={workspaceFixture({ projects: [] })} editable={false} onChange={vi.fn()} />);
    expect(screen.getByRole("heading", { name: "Expériences professionnelles" })).toBeInTheDocument();
    expect(screen.queryByRole("heading", { name: "Projets" })).not.toBeInTheDocument();
  });

  it("enregistre uniquement le texte d’un collage", () => {
    const onChange = vi.fn();
    render(<ResumePaper workspace={workspaceFixture()} editable onChange={onChange} />);
    const profile = screen.getByRole("textbox", { name: "Profil" });
    fireEvent.paste(profile, { clipboardData: { getData: () => "<b>React</b>" } });
    expect(onChange).toHaveBeenCalledWith({ type: "profile" }, "<b>React</b>");
    expect(profile.querySelector("b")).toBeNull();
  });

  it("omet une section de compétences sans groupe", () => {
    render(<ResumePaper workspace={workspaceFixture({ skill_groups: [] })} editable={false} onChange={vi.fn()} />);
    expect(screen.queryByRole("heading", { name: "Compétences" })).not.toBeInTheDocument();
  });

  it("n'affiche pas un groupe de compétences vidé de tous ses items", () => {
    render(
      <ResumePaper
        workspace={workspaceFixture({ skill_groups: [{ id: "group-1", name: "Compétences", items: [] }] })}
        editable={false}
        onChange={vi.fn()}
      />,
    );
    expect(screen.queryByRole("heading", { name: "Compétences" })).not.toBeInTheDocument();
    expect(screen.queryByText("Compétences", { selector: "h3" })).not.toBeInTheDocument();
  });

  it("n'affiche que les groupes de compétences non vides", () => {
    render(
      <ResumePaper
        workspace={workspaceFixture({
          skill_groups: [
            { id: "group-vide", name: "Langages", items: [] },
            { id: "group-plein", name: "Outils", items: ["Docker"] },
          ],
        })}
        editable={false}
        onChange={vi.fn()}
      />,
    );
    expect(screen.getByRole("heading", { name: "Compétences" })).toBeInTheDocument();
    expect(screen.queryByText("Langages")).not.toBeInTheDocument();
    expect(screen.getByText("Outils")).toBeInTheDocument();
    expect(screen.getByText("Docker")).toBeInTheDocument();
  });

  it("cible la bonne puce d'expérience à l'édition", () => {
    const onChange = vi.fn();
    render(<ResumePaper workspace={workspaceFixture()} editable onChange={onChange} />);
    const bullet = screen.getByRole("textbox", { name: "Réalisation 1.1" });
    fireEvent.paste(bullet, { clipboardData: { getData: () => "Nouvelle réalisation" } });
    expect(onChange).toHaveBeenCalledWith({ type: "experience_bullet", index: 0, item: 0 }, "Nouvelle réalisation");
  });

  it("rend une URL sûre dans un nouvel onglet isolé", () => {
    const workspace = workspaceFixture({
      identity: {
        ...workspaceFixture().document.identity,
        website: "https://example.test/profil",
      },
    });
    render(<ResumePaper workspace={workspace} editable={false} onChange={vi.fn()} />);

    const link = screen.getByRole("link", { name: "https://example.test/profil" });
    expect(link).toHaveAttribute("href", "https://example.test/profil");
    expect(link).toHaveAttribute("target", "_blank");
    expect(link).toHaveAttribute("rel", "noopener noreferrer");
  });

  it("laisse une URL dangereuse visible sans la rendre cliquable", () => {
    const workspace = workspaceFixture({
      identity: {
        ...workspaceFixture().document.identity,
        website: "javascript:alert(1)",
      },
      projects: [
        {
          id: "project-dangerous",
          name: "Projet historique",
          meta: null,
          url: "data:text/html,attaque",
          bullets: [],
        },
      ],
    });
    render(<ResumePaper workspace={workspace} editable={false} onChange={vi.fn()} />);

    expect(screen.getByText("javascript:alert(1)")).toBeInTheDocument();
    expect(screen.getByText("data:text/html,attaque")).toBeInTheDocument();
    expect(screen.queryByRole("link")).not.toBeInTheDocument();
  });
});

describe("ResumePaper — photo du profil", () => {
  const PHOTO =
    "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==";

  it("compose le CV sans réserver d'espace quand aucune photo n'existe", () => {
    render(
      <ResumePaper workspace={workspaceFixture()} editable={false} onChange={vi.fn()} photo={null} />,
    );

    expect(screen.queryByRole("img")).not.toBeInTheDocument();
  });

  it("place la photo dans l'en-tête sans la déformer quand elle existe", () => {
    render(
      <ResumePaper
        workspace={workspaceFixture()}
        editable={false}
        onChange={vi.fn()}
        photo={PHOTO}
      />,
    );

    const image = screen.getByRole("img", { name: "Photo de profil" });
    expect(image).toHaveAttribute("src", PHOTO);
    // `contain` : l'image est inscrite dans son cadre, jamais étirée pour le remplir.
    expect(image.className).toContain("object-contain");
  });
});
