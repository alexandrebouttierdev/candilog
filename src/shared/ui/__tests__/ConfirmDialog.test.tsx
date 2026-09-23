import { fireEvent, render, screen, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { ConfirmDialog } from "../ConfirmDialog";

describe("ConfirmDialog", () => {
  it("énumère ce qui disparaît, en registre de destruction", () => {
    // Le design remplace « êtes-vous sûr ? » par la liste des conséquences, et une
    // destruction se confirme par un bouton en contour rouge, jamais en rouge plein.
    render(
      <ConfirmDialog
        open
        title="Supprimer CAN-142 ?"
        description="La candidature et ses échéances disparaissent. L'entreprise et le contact restent."
        consequences={[
          { label: "Documents rattachés", value: "2" },
          { label: "Relance programmée", value: "1" },
        ]}
        footnote="action définitive"
        onCancel={vi.fn()}
        onConfirm={vi.fn()}
      />,
    );

    const dialogue = screen.getByRole("alertdialog", { name: "Supprimer CAN-142 ?" });
    expect(within(dialogue).getByText("Documents rattachés")).toBeInTheDocument();
    expect(within(dialogue).getByText("action définitive")).toBeInTheDocument();
  });

  it("donne le focus au bouton le moins destructeur", () => {
    // Un ⏎ réflexe à l'ouverture ne doit pas supprimer.
    render(
      <ConfirmDialog open title="Supprimer ?" description="…" onCancel={vi.fn()} onConfirm={vi.fn()} />,
    );
    expect(screen.getByRole("button", { name: "Annuler" })).toHaveFocus();
  });

  it("confirme sur ⏎ quand le focus n'est pas sur un bouton", () => {
    const onConfirm = vi.fn();
    render(
      <ConfirmDialog
        open
        register="information"
        title="Exporter les candidatures"
        description="25 lignes, 11 colonnes."
        confirmLabel="Exporter le fichier"
        onCancel={vi.fn()}
        onConfirm={onConfirm}
      />,
    );
    (document.activeElement as HTMLElement | null)?.blur();
    fireEvent.keyDown(document, { key: "Enter" });
    expect(onConfirm).toHaveBeenCalledOnce();
  });

  it("bloque la confirmation tant que la saisie exigée est incomplète", () => {
    const onConfirm = vi.fn();
    render(
      <ConfirmDialog
        open
        title="Effacer toutes les données ?"
        description="Tapez EFFACER pour confirmer."
        confirmLabel="Effacer définitivement"
        confirmDisabled
        onCancel={vi.fn()}
        onConfirm={onConfirm}
      />,
    );
    expect(screen.getByRole("button", { name: /Effacer définitivement/ })).toBeDisabled();
    (document.activeElement as HTMLElement | null)?.blur();
    fireEvent.keyDown(document, { key: "Enter" });
    expect(onConfirm).not.toHaveBeenCalled();
  });

  it("signale une confirmation occupée et empêche de la relancer", () => {
    render(
      <ConfirmDialog
        open
        title="Quitter cet écran ?"
        description="La génération sera arrêtée."
        busy
        onCancel={vi.fn()}
        onConfirm={vi.fn()}
      />,
    );

    const bouton = screen.getByRole("button", { name: "Supprimer" });
    expect(bouton).toBeDisabled();
    expect(bouton).toHaveAttribute("aria-busy", "true");
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
