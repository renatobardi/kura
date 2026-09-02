import * as React from "react";
import { Slot } from "@radix-ui/react-slot";
import { cva, type VariantProps } from "class-variance-authority";

import { cn } from "@/shared/lib/cn";

export type CardTextureTone = "light" | "dark";
export type CardTextureSize = "regular" | "compact";

/**
 * Kubo cards are flat: `bg-card` plus a 1px ring drawn as a box-shadow, no
 * border and no drop shadow. The ring keeps the card's own radius without
 * adding a layout box, so nested content never shifts by a pixel.
 */
export const CARD_RING_CLASS =
  "shadow-[0_0_0_1px_color-mix(in_oklab,hsl(var(--foreground))_10%,transparent)]";

const CARD_SURFACE_CLASS = `rounded-card bg-card ${CARD_RING_CLASS}`;

/**
 * The onboarding surface. It used to be a baked nine-slice powder texture;
 * Kubo has no textures, so it is now the flat card. The geometry the texture
 * imposed is kept deliberately — 5rem of padding (1.75rem compact) and a
 * 224px floor (136px compact) — because the onboarding layouts are built on
 * it: content is positioned against that padding, and panels overlap their
 * own controls without the floor.
 *
 * `tone="dark"` used to swap in a separately baked dark fill so its content
 * stayed bright; it now paints the near-black surface directly.
 */
export function texturedSurfaceClasses({
  size = "regular",
  tone = "light",
}: {
  size?: CardTextureSize;
  tone?: CardTextureTone;
} = {}): string {
  return cn(
    "relative isolate rounded-card",
    tone === "dark"
      ? "bg-foreground text-background"
      : `bg-card ${CARD_RING_CLASS}`,
    size === "compact"
      ? "min-h-[136px] min-w-[136px] p-7"
      : "min-h-56 min-w-56 p-20",
  );
}

export const TEXTURED_SURFACE_CLASS = texturedSurfaceClasses();

const cardVariants = cva("text-card-foreground", {
  variants: {
    variant: {
      default: CARD_SURFACE_CLASS,
      textured: "flex flex-col justify-center",
    },
  },
  defaultVariants: {
    variant: "default",
  },
});

export interface CardProps
  extends React.HTMLAttributes<HTMLDivElement>,
    VariantProps<typeof cardVariants> {
  asChild?: boolean;
  textureSize?: CardTextureSize;
  textureTone?: CardTextureTone;
}

const Card = React.forwardRef<HTMLDivElement, CardProps>(
  (
    {
      asChild = false,
      className,
      textureSize = "regular",
      textureTone = "light",
      variant,
      ...props
    },
    ref,
  ) => {
    const Comp = asChild ? Slot : "div";
    return (
      <Comp
        ref={ref}
        className={cn(
          variant === "textured" &&
            texturedSurfaceClasses({ size: textureSize, tone: textureTone }),
          cardVariants({ variant, className }),
        )}
        {...props}
      />
    );
  },
);
Card.displayName = "Card";

const CardHeader = React.forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement>
>(({ className, ...props }, ref) => (
  <div
    ref={ref}
    className={cn("flex flex-col space-y-1.5 p-6", className)}
    {...props}
  />
));
CardHeader.displayName = "CardHeader";

const CardTitle = React.forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement>
>(({ className, ...props }, ref) => (
  <div
    ref={ref}
    className={cn(
      "text-2xl font-semibold leading-none tracking-tight",
      className,
    )}
    {...props}
  />
));
CardTitle.displayName = "CardTitle";

const CardDescription = React.forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement>
>(({ className, ...props }, ref) => (
  <div
    ref={ref}
    className={cn("text-sm text-muted-foreground", className)}
    {...props}
  />
));
CardDescription.displayName = "CardDescription";

const CardContent = React.forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement>
>(({ className, ...props }, ref) => (
  <div ref={ref} className={cn("p-6 pt-0", className)} {...props} />
));
CardContent.displayName = "CardContent";

const CardFooter = React.forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement>
>(({ className, ...props }, ref) => (
  <div
    ref={ref}
    className={cn("flex items-center p-6 pt-0", className)}
    {...props}
  />
));
CardFooter.displayName = "CardFooter";

export {
  Card,
  CardHeader,
  CardFooter,
  CardTitle,
  CardDescription,
  CardContent,
};
