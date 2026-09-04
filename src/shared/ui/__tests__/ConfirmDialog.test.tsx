import { fireEvent, render, screen, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { ConfirmDialog } from "../ConfirmDialog";

describe("ConfirmDialog", () => {
  it("affiche l'icône de confirmation demandée", () => {
    render(
      <ConfirmDialog
        open
        title="Quitter cet écran ?"
        description="La génération sera arrêtée."
        confirmLabel="Quitter et arrêter"
        confirmIcon="stop"
        onCancel={vi.fn()}
        onConfirm={vi.fn()}
      />,
    );

    expect(within(screen.getByRole("button", { name: "Quitter et arrêter" })).getByText("stop"))
      .toBeInTheDocument();
  });

  it("conserve l'icône de progression pendant une confirmation occupée", () => {
    render(
      <ConfirmDialog
        open
        title="Quitter cet écran ?"
        description="La génération sera arrêtée."
        confirmIcon="stop"
        busy
        onCancel={vi.fn()}
        onConfirm={vi.fn()}
      />,
    );

    expect(within(screen.getByRole("button", { name: "Supprimer" })).getByText("progress_activity"))
      .toBeInTheDocument();
  });

  it("peut désactiver le bouton et le raccourci d'annulation", () => {
    const onCancel = vi.fn();
    render(
      <ConfirmDialog
        open
        title="Quitter cet écran ?"
        description="La génération sera arrêtée."
        cancelDisabled
        dismissDisabled
        onCancel={onCancel}
        onConfirm={vi.fn()}
      />,
    );

    expect(screen.getByRole("button", { name: "Annuler" })).toBeDisabled();
    fireEvent.keyDown(document, { key: "Escape" });
    expect(onCancel).not.toHaveBeenCalled();
  });
});
