import { ipc } from "@/shared/services/ipc";
import type { SystemResourceSnapshot } from "@/shared/types/generated/ai";

export const SYSTEM_RESOURCES_KEY = ["system-resources"] as const;

export const systemResourceService = {
  snapshot: () => ipc<SystemResourceSnapshot>("system_resource_snapshot"),
};
