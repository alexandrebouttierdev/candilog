import { describe, expect, it, vi, beforeEach } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import type { PropsWithChildren } from "react";
import { useSystemResources } from "../useSystemResources";
import { systemResourceService } from "../../services/systemResourceService";

vi.mock("../../services/systemResourceService", () => ({
  systemResourceService: { snapshot: vi.fn() },
  SYSTEM_RESOURCES_KEY: ["system-resources"],
}));

function Wrapper({ children }: PropsWithChildren) {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
}

describe("useSystemResources", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("expose le snapshot système", async () => {
    vi.mocked(systemResourceService.snapshot).mockResolvedValue({
      cpu_percent: 12,
      ram_used_percent: 40,
      ram_used_mb: 6400,
      ram_total_mb: 16000,
      vram_available: true,
      vram_used_percent: 55,
      vram_used_mb: 4000,
      vram_total_mb: 8192,
    });

    const { result } = renderHook(() => useSystemResources(), { wrapper: Wrapper });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(result.current.data?.cpu_percent).toBe(12);
    expect(systemResourceService.snapshot).toHaveBeenCalled();
  });
});
