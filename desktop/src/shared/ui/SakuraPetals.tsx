import { cn } from "@/shared/lib/cn";

/** The kit's petal outline, drawn in a 20x20 box. */
const PETAL_PATH = "M10,2 C6,6 5,12 10,18 C15,12 14,6 10,2 Z";

type Petal = {
  /** Percentages of the field, so the drift scales with the container. */
  left: number;
  top: number;
  size: number;
  rotate: number;
};

/**
 * Fixed placement, not a random scatter: these petals sit behind onboarding
 * and empty states that the screenshot suite captures, and a layout that
 * reshuffles per render is a diff on every run.
 */
const PETALS: readonly Petal[] = [
  { left: 8, top: 14, size: 46, rotate: -18 },
  { left: 23, top: 68, size: 28, rotate: 32 },
  { left: 47, top: 8, size: 34, rotate: 12 },
  { left: 62, top: 52, size: 22, rotate: -44 },
  { left: 78, top: 22, size: 40, rotate: 24 },
  { left: 88, top: 74, size: 26, rotate: -8 },
];

/**
 * Decorative sakura petals for onboarding and empty states.
 *
 * Kubo reserves color for meaning, so the brand's one flourish shows up only
 * where there is nothing to read: a few petals at 6% behind an empty screen.
 * Never on a working surface — over content they compete with it.
 *
 * The field is inert: `aria-hidden`, no pointer events, no motion, and it
 * fills its nearest positioned ancestor.
 */
export function SakuraPetals({ className }: { className?: string }) {
  return (
    <div
      aria-hidden="true"
      className={cn(
        "pointer-events-none absolute inset-0 select-none overflow-hidden",
        className,
      )}
      data-testid="sakura-petals"
    >
      {PETALS.map((petal) => (
        <svg
          aria-hidden="true"
          className="absolute opacity-[0.06]"
          fill="var(--sakura-decor)"
          key={`${petal.left}-${petal.top}`}
          style={{
            height: petal.size,
            left: `${petal.left}%`,
            top: `${petal.top}%`,
            transform: `translate(-50%, -50%) rotate(${petal.rotate}deg)`,
            width: petal.size,
          }}
          viewBox="0 0 20 20"
          xmlns="http://www.w3.org/2000/svg"
        >
          <path d={PETAL_PATH} />
        </svg>
      ))}
    </div>
  );
}
