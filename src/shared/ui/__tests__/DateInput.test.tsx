import { useState } from "react";
import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { DateInput, TimeInput } from "../DateInput";
import { FormField } from "../FormField";
import { labelDay } from "@/features/calendar/model/month";

function DateHarness({ initial = "" }: { initial?: string }) {
  const [value, setValue] = useState(initial);
  return (
    <DateInput
      aria-label="Date d'envoi"
      value={value}
      onChange={(event) => setValue(event.target.value)}
    />
  );
}

function TimeHarness({ initial = "" }: { initial?: string }) {
  const [value, setValue] = useState(initial);
  return (
    <TimeInput
      aria-label="Heure"
      value={value}
      onChange={(event) => setValue(event.target.value)}
    />
  );
}

describe("DateInput", () => {
  it("associe le libellé au champ texte", () => {
    render(
      <FormField label="Date d'envoi">
        {(props) => <DateInput {...props} />}
      </FormField>,
    );

    const champ = screen.getByLabelText("Date d'envoi");
    expect(champ).toHaveAttribute("type", "text");
    expect(champ).toHaveAttribute("placeholder", "JJ-MM-AAAA");
  });

  it("permet de saisir une date au clavier", async () => {
    render(<DateHarness />);

    await userEvent.type(screen.getByLabelText("Date d'envoi"), "25-08-2026");

    expect(screen.getByLabelText("Date d'envoi")).toHaveValue("25-08-2026");
  });

  it("écrit JJ-MM-AAAA et ferme quand on choisit un jour", async () => {
    render(<DateHarness initial="01-08-2026" />);

    await userEvent.click(screen.getByRole("button", { name: "Choisir une date" }));
    await userEvent.click(screen.getByRole("button", { name: labelDay("2026-08-25") }));

    expect(screen.getByLabelText("Date d'envoi")).toHaveValue("25-08-2026");
    expect(screen.queryByRole("dialog", { name: "Calendrier" })).not.toBeInTheDocument();
  });

  it("ferme le calendrier au clic à l'extérieur", async () => {
    render(
      <div>
        <DateHarness initial="01-08-2026" />
        <button type="button">Dehors</button>
      </div>,
    );

    await userEvent.click(screen.getByRole("button", { name: "Choisir une date" }));
    expect(screen.getByRole("dialog", { name: "Calendrier" })).toBeInTheDocument();

    await userEvent.click(screen.getByRole("button", { name: "Dehors" }));

    expect(screen.queryByRole("dialog", { name: "Calendrier" })).not.toBeInTheDocument();
  });
});

describe("TimeInput", () => {
  it("permet de saisir une heure au clavier", async () => {
    render(<TimeHarness />);

    await userEvent.type(screen.getByLabelText("Heure"), "09:30");

    expect(screen.getByLabelText("Heure")).toHaveValue("09:30");
  });

  it("écrit HH:MM quand on choisit une heure dans le sélecteur", async () => {
    render(<TimeHarness initial="14:00" />);

    await userEvent.click(screen.getByRole("button", { name: "Choisir une heure" }));
    await userEvent.selectOptions(screen.getByLabelText("Heure du jour"), "09");
    await userEvent.selectOptions(screen.getByLabelText("Minutes"), "30");

    expect(screen.getByLabelText("Heure")).toHaveValue("09:30");
  });

  it("ferme l'horloge au clic à l'extérieur", async () => {
    render(
      <div>
        <TimeHarness initial="14:00" />
        <button type="button">Dehors</button>
      </div>,
    );

    await userEvent.click(screen.getByRole("button", { name: "Choisir une heure" }));
    expect(screen.getByRole("dialog", { name: "Horloge" })).toBeInTheDocument();

    await userEvent.click(screen.getByRole("button", { name: "Dehors" }));

    expect(screen.queryByRole("dialog", { name: "Horloge" })).not.toBeInTheDocument();
  });
});
