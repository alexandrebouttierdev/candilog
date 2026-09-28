import { cn } from "@/shared/lib/cn";

/** Feuille vide aux proportions A4, en attendant le document. */
export function EmptySheet({ busy }: { busy: boolean }) {
  return (
    <div className="flex flex-1 justify-center p-[26px]">
      <div
        aria-hidden
        className="flex aspect-[210/297] w-full max-w-[520px] flex-col gap-6 rounded-[2px] bg-paper px-[8%] pt-[7%] shadow-sheet"
      >
        {[0, 1, 2, 3, 4].map((block) => (
          <div key={block} className="flex flex-col gap-2">
            <span className={cn("h-2 w-1/4 rounded-r2 bg-chip", busy && "animate-pulse")} />
            <span className={cn("h-1.5 w-full rounded-r2 bg-chip", busy && "animate-pulse")} />
            <span className={cn("h-1.5 w-4/5 rounded-r2 bg-chip", busy && "animate-pulse")} />
            <span className={cn("h-1.5 w-3/5 rounded-r2 bg-chip", busy && "animate-pulse")} />
          </div>
        ))}
      </div>
    </div>
  );
}
