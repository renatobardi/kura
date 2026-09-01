import type { ReactNode } from "react";

/**
 * Flat app-surface layer. Kubo has no gradients: this layer exists only so the
 * optional macOS glass background has a single canvas to tint.
 */
export function SurfaceTintLayer() {
  return (
    <div
      aria-hidden="true"
      className="kura-theme-tint-layer pointer-events-none absolute inset-0 -z-10"
      data-kura-surface-tint-layer
    />
  );
}

export function ContentSurface({
  children,
  unframed = false,
  terminal,
}: {
  children: ReactNode;
  terminal?: ReactNode;
  /** Used by dedicated huddle windows, which should not resemble app cards. */
  unframed?: boolean;
}) {
  return (
    <div
      className={
        unframed
          ? "relative z-10 flex min-h-0 flex-1 flex-col overflow-hidden bg-background"
          : "relative z-10 mb-2 ml-px mr-2 mt-px flex min-h-0 flex-1 flex-col overflow-hidden rounded-2xl bg-background shadow-content-edge"
      }
      data-kura-content-surface
      data-kura-content-unframed={unframed ? true : undefined}
    >
      <div className="kura-content-primary flex min-h-0 flex-1 flex-col overflow-hidden">
        {children}
      </div>
      <div className="kura-terminal-dock-host" data-terminal-dock>
        {terminal}
      </div>
    </div>
  );
}
