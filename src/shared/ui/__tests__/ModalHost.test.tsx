import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { ModalHost } from "../ModalHost";
import { TextInput } from "../Field";

function ouvrir(props: Partial<Parameters<typeof ModalHost>[0]> = {}) {
  const onClose = vi.fn();
  const onSubmit = vi.fn();
  render(
    <ModalHost open icon="work" title="Nouvelle candidature" onClose={onClose} onSubmit={onSubmit} {...props}>
      <TextInput aria-label="Poste" />
    </ModalHost>,
  );
  return { onClose, onSubmit };
}

describe("ModalHost", () => {
  it("ne rend rien tant qu'elle est fermée", () => {
    const onClose = vi.fn();
    render(
      <ModalHost open={false} icon="work" title="Nouvelle candidature" onClose={onClose}>
        <TextInput aria-label="Poste" />
      </ModalHost>,
    );
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
  });

  it("place le focus dans la modale à l'ouverture", () => {
    // Sans cela, la tabulation continue de parcourir l'arrière-plan atténué : invisible,
    // mais toujours atteignable au clavier.
    ouvrir();
    expect(screen.getByLabelText("Poste")).toHaveFocus();
  });

  it("ferme sur Échap", async () => {
    const { onClose } = ouvrir();
    await userEvent.keyboard("{Escape}");
    expect(onClose).toHaveBeenCalledOnce();
  });

  it("valide sur Ctrl+Entrée", async () => {
    const { onSubmit } = ouvrir();
    await userEvent.keyboard("{Control>}{Enter}{/Control}");
    expect(onSubmit).toHaveBeenCalledOnce();
  });

  it("permet de renommer l'action secondaire", () => {
    render(
      <ModalHost open icon="check" title="Profil importé" cancelLabel="Fermer" onClose={vi.fn()}>
        <p>Terminé</p>
      </ModalHost>,
    );
    expect(screen.getAllByRole("button", { name: "Fermer" }).length).toBeGreaterThanOrEqual(1);
    expect(screen.queryByRole("button", { name: "Annuler" })).not.toBeInTheDocument();
  });

  it("n'affiche pas d'action primaire quand il n'y a rien à soumettre", () => {
    render(
      <ModalHost open icon="visibility" title="Détail" onClose={vi.fn()}>
        <p>Lecture seule</p>
      </ModalHost>,
    );
    expect(screen.getByRole("button", { name: "Annuler" })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /Enregistrer/ })).not.toBeInTheDocument();
  });

  it("désactive l'action primaire pendant l'enregistrement", () => {
    ouvrir({ busy: true });
    expect(screen.getByRole("button", { name: /Enregistrer/ })).toBeDisabled();
  });
});
