import { useEffect, useState } from "react";

/** Compte le temps écoulé tant que `running` est vrai, et s'arrête proprement au démontage. */
export function useElapsedClock(running: boolean, startedAt: number | null): number {
  const [elapsed, setElapsed] = useState(0);

  useEffect(() => {
    if (!running || startedAt === null) return;
    const tick = () => setElapsed(Date.now() - startedAt);
    const id = window.setInterval(tick, 1000);
    return () => window.clearInterval(id);
  }, [running, startedAt]);

  if (startedAt === null) return 0;
  return elapsed;
}
