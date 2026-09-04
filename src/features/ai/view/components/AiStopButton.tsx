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
      icon={stopping ? "progress_activity" : "stop"}
      className="w-full"
      disabled={stopping}
      onClick={onStop}
    >
      {stopping ? "Arrêt…" : "Arrêter"}
    </Button>
  );
}
