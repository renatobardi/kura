import type * as React from "react";

/**
 * Kura's brand mark: a five-petal sakura drawn in line, rotated 72 degrees per
 * petal around five stamens. `SakuraBlossom` is the geometry alone — every
 * other mark in the app composes it, so the shape is defined exactly once.
 */
function SakuraBlossom() {
  return (
    <g fill="none" stroke="currentColor">
      <path
        d="M50,50 C38,43 33,27 39,15 C42,8 47,10 50,17 C53,10 58,8 61,15 C67,27 62,43 50,50 Z"
        strokeLinejoin="round"
        strokeWidth={6}
      />
      <path
        d="M50,50 C38,43 33,27 39,15 C42,8 47,10 50,17 C53,10 58,8 61,15 C67,27 62,43 50,50 Z"
        strokeLinejoin="round"
        strokeWidth={6}
        transform="rotate(72 50 50)"
      />
      <path
        d="M50,50 C38,43 33,27 39,15 C42,8 47,10 50,17 C53,10 58,8 61,15 C67,27 62,43 50,50 Z"
        strokeLinejoin="round"
        strokeWidth={6}
        transform="rotate(144 50 50)"
      />
      <path
        d="M50,50 C38,43 33,27 39,15 C42,8 47,10 50,17 C53,10 58,8 61,15 C67,27 62,43 50,50 Z"
        strokeLinejoin="round"
        strokeWidth={6}
        transform="rotate(216 50 50)"
      />
      <path
        d="M50,50 C38,43 33,27 39,15 C42,8 47,10 50,17 C53,10 58,8 61,15 C67,27 62,43 50,50 Z"
        strokeLinejoin="round"
        strokeWidth={6}
        transform="rotate(288 50 50)"
      />
      <line
        strokeLinecap="round"
        strokeWidth={3.3}
        transform="rotate(36 50 50)"
        x1={50}
        x2={50}
        y1={50}
        y2={34}
      />
      <line
        strokeLinecap="round"
        strokeWidth={3.3}
        transform="rotate(108 50 50)"
        x1={50}
        x2={50}
        y1={50}
        y2={34}
      />
      <line
        strokeLinecap="round"
        strokeWidth={3.3}
        transform="rotate(180 50 50)"
        x1={50}
        x2={50}
        y1={50}
        y2={34}
      />
      <line
        strokeLinecap="round"
        strokeWidth={3.3}
        transform="rotate(252 50 50)"
        x1={50}
        x2={50}
        y1={50}
        y2={34}
      />
      <line
        strokeLinecap="round"
        strokeWidth={3.3}
        transform="rotate(324 50 50)"
        x1={50}
        x2={50}
        y1={50}
        y2={34}
      />
      <circle cx={50} cy={50} fill="currentColor" r={5.4} />
    </g>
  );
}

/**
 * The app tile: the blossom in sakura pink on the near-black primary square.
 * Used wherever the mark needs its own container (landing corner, app icon
 * stand-ins). Inside themed surfaces prefer {@link SakuraGlyph}.
 */
export function SakuraMark({
  className,
  size,
  ...props
}: {
  className?: string;
  size?: number;
} & React.SVGAttributes<SVGSVGElement>) {
  return (
    <svg
      aria-label="Kura"
      className={["kura-mark", className].filter(Boolean).join(" ")}
      height={size}
      role="img"
      viewBox="0 0 100 100"
      {...props}
      width={size}
    >
      <rect fill="#1c1917" height={100} rx={26} width={100} />
      <g color="#f4c9d4" transform="translate(11 11) scale(0.78)">
        <SakuraBlossom />
      </g>
    </svg>
  );
}

/**
 * The bare blossom in the current text color. Presentational by default
 * (aria-hidden); passing an aria-label promotes it to a labeled image.
 */
export function SakuraGlyph({
  className,
  size,
  "aria-label": ariaLabel,
  ...props
}: {
  className?: string;
  size?: number;
} & React.SVGAttributes<SVGSVGElement>) {
  const labelProps = ariaLabel
    ? { role: "img" as const, "aria-label": ariaLabel }
    : { "aria-hidden": "true" as const };

  return (
    // biome-ignore lint/a11y/noSvgWithoutTitle: labelProps supplies aria-label whenever role="img" is set (see above).
    <svg
      className={["kura-glyph", className].filter(Boolean).join(" ")}
      height={size}
      viewBox="0 0 100 100"
      {...labelProps}
      {...props}
      width={size}
    >
      <SakuraBlossom />
    </svg>
  );
}

/**
 * Loading/splash treatment: the blossom draws itself in by stroke-dashoffset
 * once, in ~600ms, and then holds. No loop, no bounce — see
 * `kura-sakura-draw` in styles/globals/animations.css.
 */
export function SakuraDrawIn({
  className,
  size,
  "aria-label": ariaLabel,
  ...props
}: {
  className?: string;
  size?: number;
} & React.SVGAttributes<SVGSVGElement>) {
  return (
    <SakuraGlyph
      aria-label={ariaLabel}
      className={["kura-sakura-draw", className].filter(Boolean).join(" ")}
      size={size}
      {...props}
    />
  );
}
