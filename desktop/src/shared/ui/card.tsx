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
 * Kept for callers that used to request the powder-texture surface. The
 * texture is gone with the Kubo rebrand — no textures anywhere — so this now
 * returns the plain card surface. `size` and `tone` no longer change
 * anything; they remain so call sites did not all have to change at once.
 */
export const TEXTURED_SURFACE_CLASS = CARD_SURFACE_CLASS;

export function texturedSurfaceClasses(
  _options: { size?: CardTextureSize; tone?: CardTextureTone } = {},
): string {
  return CARD_SURFACE_CLASS;
}

const cardVariants = cva("text-card-foreground", {
  variants: {
    variant: {
      default: CARD_SURFACE_CLASS,
      textured: `${CARD_SURFACE_CLASS} flex flex-col justify-center`,
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
      textureSize: _textureSize,
      textureTone: _textureTone,
      variant,
      ...props
    },
    ref,
  ) => {
    const Comp = asChild ? Slot : "div";
    return (
      <Comp
        ref={ref}
        className={cn(cardVariants({ variant, className }))}
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
