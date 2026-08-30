// Static landing corner mark. The previous animated bee-swarm field has been
// removed as part of the Buzz -> Kura rebrand; kept as a tiny named component
// so callers don't need to change.
import { KuraMark } from "@/shared/ui/kura-logo/KuraMark";

export function LandingBees() {
  return (
    <div
      aria-hidden
      className="pointer-events-none absolute inset-0 overflow-hidden"
    >
      <span className="absolute left-6 top-12 block w-11 text-[#231E1E]">
        <KuraMark className="h-auto w-full" />
      </span>
    </div>
  );
}
