import { afterEach, describe, expect, it, vi } from "vitest";
import { ipc } from "@/shared/services/ipc";
import { openExternal } from "../external-link";
import { useUiStore } from "@/shared/lib/ui-store";
import { AppError } from "@/shared/types/app-error";

vi.mock("@/shared/services/ipc", () => ({ ipc: vi.fn() }));

describe("openExternal", () => {
  afterEach(() => {
    vi.clearAllMocks();
    useUiStore.setState({ toasts: [] });
  });

  it("délègue à la commande Rust plutôt qu'à window.open", async () => {
    vi.mocked(ipc).mockResolvedValue(undefined);

    await openExternal("https://exemple.test/offre");

    expect(ipc).toHaveBeenCalledWith("open_external_url", { url: "https://exemple.test/offre" });
  });

  it("annonce l'échec plutôt que de rester muette", async () => {
    vi.mocked(ipc).mockRejectedValue(
      new AppError({ code: "VALIDATION_ERROR", message: "Lien illisible." }),
    );

    await openExternal("pas une url");

    const [toast] = useUiStore.getState().toasts;
    expect(toast).toMatchObject({ tone: "error", title: "Lien impossible à ouvrir" });
  });
});
