// Static landing corner mark: the sakura tile. The previous animated
// bee-swarm field is long gone; the component keeps its name so callers
// don't need to change.
import { SakuraMark } from "@/shared/ui/kura-logo/SakuraMark";

export function LandingBees() {
  return (
    <div
      aria-hidden
      className="pointer-events-none absolute inset-0 overflow-hidden"
    >
      <span className="absolute left-6 top-12 block w-11">
        <SakuraMark className="h-auto w-full" />
      </span>
    </div>
  );
}
