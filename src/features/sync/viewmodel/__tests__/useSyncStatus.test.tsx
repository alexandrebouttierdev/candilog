import type { ReactNode } from "react";
import { QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { createTestQueryClient } from "@/shared/lib/test-utils";
import type { SyncStatus } from "@/shared/types/generated/sync";
import { syncService } from "../../services/syncService";
import { SYNC_STATUS_KEY, useSyncEvents, useSyncStatus } from "../useSyncStatus";

const unlisten = vi.fn();
let handler: ((event: { payload: SyncStatus }) => void) | null = null;

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn((_name: string, callback: typeof handler) => {
    handler = callback;
    return Promise.resolve(unlisten);
  }),
}));

const status = (phase: SyncStatus["phase"]): SyncStatus => ({
  provider: "folder",
  phase,
  auto_sync: true,
  location: "/tmp",
  pending_changes: 0,
  conflicts: 0,
  waiting_changes: 0,
  missing_files: 0,
  computers: 2,
  last_success_at: null,
  last_attempt_at: null,
  next_attempt_at: null,
  message: null,
});

beforeEach(() => {
  vi.restoreAllMocks();
  handler = null;
  unlisten.mockReset();
});

describe("pont de l'état de synchronisation", () => {
  it("met l'état à jour et relit les écrans à la fin d'un cycle", async () => {
    vi.spyOn(syncService, "status").mockResolvedValue(status("syncing"));
    const client = createTestQueryClient();
    const wrapper = ({ children }: { children: ReactNode }) => (
      <QueryClientProvider client={client}>{children}</QueryClientProvider>
    );
    client.setQueryData(["applications", "list"], []);
    const { result, unmount } = renderHook(
      () => {
        useSyncEvents();
        return useSyncStatus();
      },
      { wrapper },
    );
    await waitFor(() => expect(result.current.data?.phase).toBe("syncing"));
    await waitFor(() => expect(handler).not.toBeNull());

    act(() => handler?.({ payload: status("up_to_date") }));

    expect(client.getQueryData<SyncStatus>(SYNC_STATUS_KEY)?.phase).toBe("up_to_date");
    expect(client.getQueryState(["applications", "list"])?.isInvalidated).toBe(true);
    unmount();
    expect(unlisten).toHaveBeenCalledOnce();
  });
});
