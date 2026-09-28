import { Button } from "@/shared/ui";

export function AiStopButton({
  stopping,
  onStop,
}: {
  stopping: boolean;
  onStop: () => void;
}) {
  return (
    <Button
      variant="danger"
      className="w-full"
      disabled={stopping}
      onClick={onStop}
    >
      {stopping ? "Arrêt…" : "Arrêter"}
    </Button>
  );
}
