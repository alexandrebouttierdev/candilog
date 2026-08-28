import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";
import { FormField } from "../FormField";
import { TextInput } from "../Field";

describe("FormField", () => {
  it("associe le libellé au champ", () => {
    render(
      <FormField label="Poste">{(props) => <TextInput {...props} />}</FormField>,
    );
    expect(screen.getByLabelText("Poste")).toBeInTheDocument();
  });

  it("rend l'erreur sous le champ et la lie par aria-describedby", () => {
    // Le guide interdit de n'exposer l'erreur qu'en infobulle : invisible au clavier comme
    // au lecteur d'écran.
    render(
      <FormField label="Date d'envoi" error="Date invalide — format attendu JJ-MM-AAAA.">
        {(props) => <TextInput {...props} />}
      </FormField>,
    );

    const champ = screen.getByLabelText(/Date d'envoi/);
    expect(champ).toHaveAttribute("aria-invalid", "true");
    expect(champ).toHaveAccessibleDescription("Date invalide — format attendu JJ-MM-AAAA.");
  });

  it("laisse l'erreur remplacer l'aide plutôt que s'y ajouter", () => {
    render(
      <FormField label="Lien" help="Facultatif." error="URL invalide.">
        {(props) => <TextInput {...props} />}
      </FormField>,
    );

    expect(screen.queryByText("Facultatif.")).not.toBeInTheDocument();
    expect(screen.getByText("URL invalide.")).toBeInTheDocument();
  });

  it("ne marque pas invalide un champ sans erreur", () => {
    render(<FormField label="Ville">{(props) => <TextInput {...props} />}</FormField>);
    expect(screen.getByLabelText("Ville")).toHaveAttribute("aria-invalid", "false");
  });
});
