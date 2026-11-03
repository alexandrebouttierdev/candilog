import { beforeEach, describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter, Route, Routes, useLocation } from "react-router-dom";
import { AiRequiredModal } from "../AiRequiredModal";
import { useAiRequiredStore } from "../../../viewmodel/ai-required-store";

function Destination() {
  const location = useLocation();
  return <p>{`${location.pathname}${location.search}`}</p>;
}

function monter() {
  render(
    <MemoryRouter initialEntries={["/relations"]}>
      <AiRequiredModal />
      <Routes>
        <Route path="*" element={<Destination />} />
      </Routes>
    </MemoryRouter>,
  );
}

beforeEach(() => {
  useAiRequiredStore.setState({ open: true, llm: null });
});

describe("invitation à configurer l'IA", () => {
  it("ouvre l'onglet IA locale", async () => {
    monter();
    await userEvent.click(screen.getByRole("button", { name: "Installer l’IA locale" }));
    expect(screen.getByText("/ai?onglet=local")).toBeInTheDocument();
    expect(useAiRequiredStore.getState().open).toBe(false);
  });

  it("ouvre l'onglet des fournisseurs externes", async () => {
    monter();
    await userEvent.click(screen.getByRole("button", { name: "Configurer un fournisseur" }));
    expect(screen.getByText("/ai?onglet=fournisseurs")).toBeInTheDocument();
  });
});
