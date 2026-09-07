import { describe, expect, it } from "vitest";
import { railIaStatus } from "../railIaStatus";

describe("railIaStatus", () => {
  const base = {
    configured: true,
    busy: false,
    localError: false,
    connectionError: false,
    operationError: false,
  };

  it("signale disponible quand configuré et idle", () => {
    expect(railIaStatus(base)).toEqual({ tone: "success", label: "Disponible" });
  });

  it("signale non configuré", () => {
    expect(railIaStatus({ ...base, configured: false })).toEqual({
      tone: "neutral",
      label: "Non configuré",
    });
  });

  it("signale une erreur connue", () => {
    expect(railIaStatus({ ...base, connectionError: true })).toEqual({
      tone: "danger",
      label: "Erreur",
    });
    expect(railIaStatus({ ...base, localError: true })).toMatchObject({ tone: "danger" });
    expect(railIaStatus({ ...base, operationError: true })).toMatchObject({ tone: "danger" });
  });

  it("priorise l'occupation sur l'erreur", () => {
    expect(
      railIaStatus({ ...base, busy: true, connectionError: true }),
    ).toEqual({ tone: "warning", label: "En cours…" });
  });
});
