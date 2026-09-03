import { describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { OnboardingTour } from "../OnboardingTour";
import { ONBOARDING_STEPS } from "../../../model/steps";

describe("OnboardingTour", () => {
  it("ouvre sur la première étape et annonce le dialogue", () => {
    render(<OnboardingTour onFinish={vi.fn()} />);

    const dialog = screen.getByRole("dialog", { name: ONBOARDING_STEPS[0]!.title });
    expect(dialog).toBeInTheDocument();
    expect(screen.getByText(ONBOARDING_STEPS[0]!.description)).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Précédent" })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Fermer" })).not.toBeInTheDocument();
  });

  it("reste ouvert sur Escape, contrairement aux autres modales", () => {
    render(<OnboardingTour onFinish={vi.fn()} />);

    fireEvent.keyDown(document, { key: "Escape" });

    expect(screen.getByRole("dialog")).toBeInTheDocument();
  });

  it("avance étape par étape jusqu'à la dernière, puis termine", async () => {
    const onFinish = vi.fn();
    render(<OnboardingTour onFinish={onFinish} />);

    for (let étape = 0; étape < ONBOARDING_STEPS.length - 1; étape += 1) {
      await userEvent.click(screen.getByRole("button", { name: "Suivant" }));
    }

    expect(
      screen.getByRole("dialog", { name: ONBOARDING_STEPS.at(-1)!.title }),
    ).toBeInTheDocument();
    expect(onFinish).not.toHaveBeenCalled();

    await userEvent.click(screen.getByRole("button", { name: "Commencer" }));
    expect(onFinish).toHaveBeenCalledOnce();
  });

  it("permet de revenir en arrière sans terminer le tour", async () => {
    render(<OnboardingTour onFinish={vi.fn()} />);

    await userEvent.click(screen.getByRole("button", { name: "Suivant" }));
    expect(
      screen.getByRole("dialog", { name: ONBOARDING_STEPS[1]!.title }),
    ).toBeInTheDocument();

    await userEvent.click(screen.getByRole("button", { name: "Précédent" }));
    expect(
      screen.getByRole("dialog", { name: ONBOARDING_STEPS[0]!.title }),
    ).toBeInTheDocument();
  });
});
