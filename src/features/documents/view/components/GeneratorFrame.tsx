import type { ComponentProps } from "react";
import { useNavigate } from "react-router-dom";
import { PATHS } from "@/shared/lib/paths";
import { WorkSurface } from "@/shared/ui";

export { PaneSection, RunMeter, StepList } from "@/shared/ui";
export type { GeneratorStep } from "@/shared/ui";

/**
 * Surcouche des générateurs et de l'analyse (`screens/15`, `16`, `09`) : une `WorkSurface`
 * rattachée à Documents, où la fermeture ramène.
 */
export function GeneratorFrame(props: Omit<ComponentProps<typeof WorkSurface>, "crumbRoot" | "onClose">) {
  const navigate = useNavigate();
  return <WorkSurface {...props} crumbRoot="Documents" onClose={() => void navigate(PATHS.documents)} />;
}
