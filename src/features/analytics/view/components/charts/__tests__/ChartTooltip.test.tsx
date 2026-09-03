import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";
import { ChartTooltip } from "../ChartTooltip";

describe("ChartTooltip", () => {
  it("affiche le libellé de la série et sa valeur", () => {
    render(<ChartTooltip title="Semaine du 17 août">3 candidatures</ChartTooltip>);

    expect(screen.getByText("Semaine du 17 août")).toBeInTheDocument();
    expect(screen.getByText("3 candidatures")).toBeInTheDocument();
  });
});
