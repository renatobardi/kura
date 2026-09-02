"use client";

import * as React from "react";
import * as DialogPrimitive from "@radix-ui/react-dialog";
import { X } from "lucide-react";

import { cn } from "@/shared/lib/cn";
import { CARD_RING_CLASS, texturedSurfaceClasses } from "@/shared/ui/card";
import { useTheme } from "@/shared/theme/ThemeProvider";
import { MODAL_BACKDROP_BLUR_CLASS } from "@/shared/ui/modalBackdrop";
import {
  MODAL_CONTENT_MOTION_CLASS,
  MODAL_OVERLAY_MOTION_CLASS,
} from "@/shared/ui/modalMotion";

const Dialog = DialogPrimitive.Root;
const DialogTrigger = DialogPrimitive.Trigger;
const DialogPortal = DialogPrimitive.Portal;
const DialogClose = DialogPrimitive.Close;

const DialogOverlay = React.forwardRef<
  React.ElementRef<typeof DialogPrimitive.Overlay>,
  React.ComponentPropsWithoutRef<typeof DialogPrimitive.Overlay>
>(({ className, ...props }, ref) => {
  const { isDark } = useTheme();

  return (
    <DialogPrimitive.Overlay
      className={cn(
        "fixed inset-0 z-50",
        MODAL_OVERLAY_MOTION_CLASS,
        MODAL_BACKDROP_BLUR_CLASS,
        isDark ? "bg-black/60" : "bg-black/10",
        className,
      )}
      ref={ref}
      {...props}
    />
  );
});
DialogOverlay.displayName = DialogPrimitive.Overlay.displayName;

type DialogContentProps = React.ComponentPropsWithoutRef<
  typeof DialogPrimitive.Content
> & {
  /** Extra classes for the built-in close button (e.g. a themed icon color). */
  closeButtonClassName?: string;
  /** Extra classes for this dialog's backdrop. */
  overlayClassName?: string;
  overlayVariant?: "default" | "transparent";
  showCloseButton?: boolean;
  /**
   * - `default`: the standard flat panel — pill radius, 1px ring, no shadow.
   * - `none`: no surface — the caller composes its own.
   * - `textured`: legacy alias of `default`; the powder texture is gone.
   */
  surface?: "default" | "none" | "textured";
};

const DialogContent = React.forwardRef<
  React.ElementRef<typeof DialogPrimitive.Content>,
  DialogContentProps
>(
  (
    {
      className,
      children,
      closeButtonClassName,
      overlayClassName,
      overlayVariant = "default",
      showCloseButton = true,
      surface = "default",
      ...props
    },
    ref,
  ) => (
    <DialogPortal>
      <DialogOverlay
        data-testid="dialog-overlay"
        className={cn(
          overlayVariant === "transparent"
            ? "bg-transparent backdrop-blur-none"
            : undefined,
          overlayClassName,
        )}
      />
      <div
        className={cn(
          "pointer-events-none fixed inset-0 z-50 grid place-items-center overflow-x-hidden overflow-y-auto p-4",
        )}
      >
        <DialogPrimitive.Content
          className={cn(
            "pointer-events-auto relative grid w-[calc(100vw-2rem)] max-w-2xl gap-4 outline-hidden",
            // Kubo dialogs are flat: the dialog radius, a plain surface and
            // the 1px ring — no drop shadow.
            surface === "default" &&
              `rounded-dialog bg-background p-6 ${CARD_RING_CLASS}`,
            surface === "none" && "bg-transparent p-0 shadow-none",
            // `textured` is the onboarding surface: same flat card, but it
            // keeps the padding and floor the old texture imposed, because
            // those dialogs lay their content out against them.
            surface === "textured" &&
              cn("box-border w-full", texturedSurfaceClasses()),
            MODAL_CONTENT_MOTION_CLASS,
            className,
          )}
          ref={ref}
          {...props}
        >
          {children}
          {showCloseButton ? (
            <DialogPrimitive.Close
              className={cn(
                "absolute flex h-8 w-8 items-center justify-center rounded-md text-muted-foreground transition-colors duration-150 ease-out hover:bg-accent hover:text-accent-foreground focus:outline-hidden focus-visible:ring-1 focus-visible:ring-ring",
                surface === "textured" ? "right-20 top-20" : "right-4 top-4",
                closeButtonClassName,
              )}
            >
              <X className="h-4 w-4" />
              <span className="sr-only">Close</span>
            </DialogPrimitive.Close>
          ) : null}
        </DialogPrimitive.Content>
      </div>
    </DialogPortal>
  ),
);
DialogContent.displayName = DialogPrimitive.Content.displayName;

const DialogHeader = ({
  className,
  ...props
}: React.HTMLAttributes<HTMLDivElement>) => (
  <div
    className={cn("flex flex-col space-y-2 text-left", className)}
    {...props}
  />
);
DialogHeader.displayName = "DialogHeader";

const DialogFooter = ({
  className,
  ...props
}: React.HTMLAttributes<HTMLDivElement>) => (
  <div
    className={cn(
      "flex flex-col-reverse gap-2 sm:flex-row sm:justify-end",
      className,
    )}
    {...props}
  />
);
DialogFooter.displayName = "DialogFooter";

const DialogTitle = React.forwardRef<
  React.ElementRef<typeof DialogPrimitive.Title>,
  React.ComponentPropsWithoutRef<typeof DialogPrimitive.Title>
>(({ className, ...props }, ref) => (
  <DialogPrimitive.Title
    className={cn("text-xl font-semibold tracking-tight", className)}
    ref={ref}
    {...props}
  />
));
DialogTitle.displayName = DialogPrimitive.Title.displayName;

const DialogDescription = React.forwardRef<
  React.ElementRef<typeof DialogPrimitive.Description>,
  React.ComponentPropsWithoutRef<typeof DialogPrimitive.Description>
>(({ className, ...props }, ref) => (
  <DialogPrimitive.Description
    className={cn("text-sm text-muted-foreground", className)}
    ref={ref}
    {...props}
  />
));
DialogDescription.displayName = DialogPrimitive.Description.displayName;

export {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogPortal,
  DialogTitle,
  DialogTrigger,
};
