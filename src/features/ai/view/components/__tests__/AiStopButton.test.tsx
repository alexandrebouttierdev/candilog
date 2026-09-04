import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { AiStopButton } from "../AiStopButton";

describe("AiStopButton", () => {
  it("demande l'arrêt avec un bouton danger pleine largeur", async () => {
    const onStop = vi.fn();
    render(<AiStopButton stopping={false} onStop={onStop} />);

    await userEvent.click(screen.getByRole("button", { name: "Arrêter" }));

    expect(onStop).toHaveBeenCalledOnce();
    expect(screen.getByRole("button", { name: "Arrêter" })).toHaveClass("w-full");
  });

  it("désactive le bouton pendant l'arrêt", () => {
    render(<AiStopButton stopping onStop={vi.fn()} />);

    expect(screen.getByRole("button", { name: "Arrêt…" })).toBeDisabled();
    expect(screen.getByText("progress_activity")).toBeInTheDocument();
  });
});
