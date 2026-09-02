import { clsx, type ClassValue } from "clsx";
import { extendTailwindMerge } from "tailwind-merge";

const mergeClassNames = extendTailwindMerge({
  extend: {
    classGroups: {
      "font-size": [
        {
          text: ["message", "message-timestamp"],
        },
      ],
      // Kubo component radii. Without this, tailwind-merge does not know
      // `rounded-pill` belongs to the radius group, keeps it alongside a
      // caller's `rounded-2xl`, and lets stylesheet order decide — so a
      // primitive's default radius silently overrode its call sites.
      rounded: [
        {
          rounded: ["pill", "card", "textarea", "dialog"],
        },
      ],
    },
  },
});

export function cn(...inputs: ClassValue[]) {
  return mergeClassNames(clsx(inputs));
}
