import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { usePointerDrag } from "../usePointerDrag";

function Handle() {
  const start = usePointerDrag(vi.fn());
  return <button onPointerDown={start}>Poignée</button>;
}

describe("usePointerDrag", () => {
  it("retire tous les écouteurs globaux si le composant est démonté pendant le drag", async () => {
    const removeDocument = vi.spyOn(document, "removeEventListener");
    const removeWindow = vi.spyOn(window, "removeEventListener");
    const { unmount } = render(<Handle />);

    await userEvent.pointer({ target: screen.getByRole("button"), keys: "[MouseLeft>]" });
    unmount();

    expect(removeDocument).toHaveBeenCalledWith("pointermove", expect.any(Function));
    expect(removeDocument).toHaveBeenCalledWith("pointerup", expect.any(Function));
    expect(removeDocument).toHaveBeenCalledWith("pointercancel", expect.any(Function));
    expect(removeWindow).toHaveBeenCalledWith("blur", expect.any(Function));
  });
});
