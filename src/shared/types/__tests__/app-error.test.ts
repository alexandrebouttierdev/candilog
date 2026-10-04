import { describe, expect, it } from "vitest";
import { AppError, toAppError } from "../app-error";

describe("normalisation des erreurs IPC", () => {
  it("reconstitue une AppError depuis le contrat du backend", () => {
    const error = toAppError({ code: "NOT_FOUND", message: "Introuvable : candidature." });

    expect(error).toBeInstanceOf(AppError);
    expect(error.code).toBe("NOT_FOUND");
    expect(error.message).toBe("Introuvable : candidature.");
  });

  it("reste interceptable comme une Error ordinaire", () => {
    // TanStack Query et les error boundaries ne reconnaissent que `Error` : un objet nu
    // porteur d'un code traverserait ces frontières sans être traité comme une erreur.
    expect(toAppError({ code: "CANCELLED", message: "Génération annulée." })).toBeInstanceOf(
      Error,
    );
  });

  it("présente une phrase lisible quand l'IPC échoue avant d'atteindre une commande", () => {
    // Commande inconnue, permission refusée par les capabilities, argument non
    // désérialisable : Tauri rejette alors une chaîne, pas le contrat `{ code, message }`.
    // Cette chaîne est en anglais et nomme des composants internes : elle ne peut pas être
    // le message affiché (`docs/CODE_RULES.md` §1 et §13), mais elle reste utile au
    // diagnostic — d'où le détail à côté du message générique.
    const error = toAppError("command not found");

    expect(error.code).toBe("IPC_ERROR");
    expect(error.message).toBe(
      "Une erreur inattendue est survenue lors de la communication avec Candilog.",
    );
    expect(error.detail).toBe("command not found");
  });

  it("n'invente pas de détail quand l'IPC rejette autre chose qu'une chaîne", () => {
    const error = toAppError({ inattendu: true });

    expect(error.code).toBe("IPC_ERROR");
    expect(error.detail).toBeUndefined();
  });

  it("ne double pas l'enveloppe d'une AppError déjà normalisée", () => {
    const original = new AppError({ code: "VALIDATION_ERROR", message: "Le poste est requis" });

    expect(toAppError(original)).toBe(original);
  });

  it("signale l'annulation pour qu'elle soit ignorée plutôt qu'affichée", () => {
    expect(new AppError({ code: "CANCELLED", message: "Génération annulée." }).isCancelled).toBe(
      true,
    );
    expect(
      new AppError({ code: "DATABASE_ERROR", message: "Fichier illisible." }).isCancelled,
    ).toBe(false);
  });
});
