import { fireEvent, render, screen } from "@testing-library/react";
import { FollowUpList } from "../AnalyticsUi";

describe("AnalyticsUi", () => {
  it("transmet la candidature choisie à l'action de relance", () => {
    const onFollowUp = vi.fn();
    const item = {
      id: "candidate-1",
      job_title: "Développeur Rust",
      company_name: "Nova Digital",
      sent_date: "2026-08-10",
      days: 18,
    };
    render(<FollowUpList items={[item]} onFollowUp={onFollowUp} />);

    fireEvent.click(screen.getByRole("button", { name: /Relancer/ }));

    expect(onFollowUp).toHaveBeenCalledWith(item);
  });
});
