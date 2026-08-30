import { cn } from "@/shared/lib/cn";
import { KuraGlyph } from "@/shared/ui/kura-logo/KuraMark";

/** Centered, low-emphasis loading state for page and panel fetches. */
export function KuraLoadingState({
  className,
  fill = false,
  label = "Loading",
}: {
  className?: string;
  fill?: boolean;
  label?: string;
}) {
  return (
    <div
      className={cn(
        "flex w-full items-center justify-center text-muted-foreground/45",
        fill ? "min-h-0 flex-1" : "min-h-[calc(100dvh-7rem)]",
        className,
      )}
      data-testid="kura-loading-state"
      role="status"
    >
      <KuraGlyph
        aria-label={label}
        className="animate-pulse"
        style={{ width: "2rem" }}
      />
    </div>
  );
}
