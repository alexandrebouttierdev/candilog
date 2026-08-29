import { useState } from "react";
import { describe, expect, it } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { DateInput, TimeInput } from "../DateInput";
import { FormField } from "../FormField";

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
  it("associe le libellé au champ texte, pas au calendrier natif", () => {
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

  it("écrit JJ-MM-AAAA quand on choisit une date dans le calendrier", () => {
    render(<DateHarness />);

    fireEvent.change(screen.getByLabelText("Choisir une date"), {
      target: { value: "2026-08-25" },
    });

    expect(screen.getByLabelText("Date d'envoi")).toHaveValue("25-08-2026");
  });

  it("le calendrier natif n'a pas de name, pour ne pas partir en double à la soumission", () => {
    render(<DateInput name="sent_date" />);

    expect(screen.getByLabelText("Choisir une date")).not.toHaveAttribute("name");
  });
});

describe("TimeInput", () => {
  it("permet de saisir une heure au clavier", async () => {
    render(<TimeHarness />);

    await userEvent.type(screen.getByLabelText("Heure"), "09:30");

    expect(screen.getByLabelText("Heure")).toHaveValue("09:30");
  });

  it("écrit HH:MM quand on choisit une heure dans le sélecteur", () => {
    render(<TimeHarness />);

    fireEvent.change(screen.getByLabelText("Choisir une heure"), {
      target: { value: "14:00:00" },
    });

    expect(screen.getByLabelText("Heure")).toHaveValue("14:00");
  });
});
