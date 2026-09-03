import { SakuraPetals } from "@/shared/ui/SakuraPetals";

export function ChannelScreenEmptyState() {
  return (
    <div className="relative flex min-h-0 flex-1 items-center justify-center px-6 py-8">
      <SakuraPetals />
      <p className="relative text-sm text-muted-foreground">
        Select a channel to view messages.
      </p>
    </div>
  );
}
