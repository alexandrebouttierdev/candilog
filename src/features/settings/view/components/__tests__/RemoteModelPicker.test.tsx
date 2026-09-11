import { describe, expect, it, vi } from "vitest";
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { RemoteModelPicker } from "../RemoteModelPicker";

describe("RemoteModelPicker", () => {
  const models = ["gpt-4o", "gpt-4o-mini", "o3-mini"];

  it("présente les modèles comme un groupe de boutons radio avec le logo fournisseur", () => {
    render(
      <RemoteModelPicker
        models={models}
        value="gpt-4o"
        onChange={() => undefined}
        providerLabel="OpenAI"
        providerId="openai"
      />,
    );

    expect(screen.getByRole("radiogroup", { name: "Modèles OpenAI" })).toBeInTheDocument();
    expect(screen.getAllByRole("radio")).toHaveLength(3);
    const selected = screen.getByRole("radio", { name: "gpt-4o" });
    expect(selected).toHaveAttribute("aria-checked", "true");
    expect(within(selected).getByText("check_circle")).toBeInTheDocument();
    expect(selected.querySelector("img")).not.toBeNull();
  });

  it("notifie le changement de modèle", async () => {
    const onChange = vi.fn();
    render(
      <RemoteModelPicker
        models={models}
        value="gpt-4o"
        onChange={onChange}
        providerLabel="OpenAI"
        providerId="openai"
      />,
    );

    await userEvent.click(screen.getByRole("radio", { name: "o3-mini" }));
    expect(onChange).toHaveBeenCalledWith("o3-mini");
  });
});
