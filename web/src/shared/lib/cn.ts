import { type ClassValue, clsx } from "clsx";
import { extendTailwindMerge } from "tailwind-merge";

/**
 * tailwind-merge has to be told about the Kubo radius keys. Without them it
 * treats `rounded-pill` as unrelated to `rounded-2xl` and keeps both, leaving
 * the CSS order to decide — which silently lets a primitive's default win over
 * the call site that meant to override it.
 */
const twMerge = extendTailwindMerge({
  extend: {
    classGroups: {
      rounded: [{ rounded: ["pill", "card", "textarea", "dialog"] }],
    },
  },
});

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}
