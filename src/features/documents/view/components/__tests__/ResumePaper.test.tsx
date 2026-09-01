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
});
