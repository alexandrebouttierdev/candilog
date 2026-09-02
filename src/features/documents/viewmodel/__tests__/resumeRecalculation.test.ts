import { describe, expect, it, vi } from "vitest";
import type { ResumeWorkspace } from "@/shared/types/generated/documents";
import { workspaceFixture } from "../../model/resumeWorkspace";
import { runResumeRecalculation } from "../resumeRecalculation";

describe("frontière asynchrone du recalcul de CV", () => {
  it("ignore une réponse arrivée après le démontage", async () => {
    let resolveRequest: ((workspace: ResumeWorkspace) => void) | undefined;
    let active = true;
    const onSuccess = vi.fn();
    const onError = vi.fn();
    const onSettled = vi.fn();
    const request = runResumeRecalculation({
      workspace: workspaceFixture(),
      recalculate: () =>
        new Promise((resolve) => {
          resolveRequest = resolve;
        }),
      isCurrent: () => active,
      onSuccess,
      onError,
      onSettled,
    });

    active = false;
    resolveRequest?.(workspaceFixture());
    await request;

    expect(onSuccess).not.toHaveBeenCalled();
    expect(onError).not.toHaveBeenCalled();
    expect(onSettled).not.toHaveBeenCalled();
  });
});
