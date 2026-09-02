import { cn } from "@/shared/lib/cn";
import { Tabs, TabsList, TabsTrigger } from "@/shared/ui/tabs";
import type { AvatarTab } from "./AgentCreationPreview.utils";
import {
  PRESET_AVATARS,
  parsePresetAvatarDataUrl,
  presetAvatarDataUrl,
} from "./presetAvatars";

/**
 * The avatar picker's tab strip and its preset gallery panel.
 *
 * Split out of `AgentCreationPreview` so that file — already over the repo's
 * size ceiling — does not carry the gallery's markup too.
 */
export function AvatarPickerTabs({
  activeTab,
  onCloseCustomColorPicker,
  onTabChange,
  showPresetGallery,
}: {
  activeTab: AvatarTab;
  onCloseCustomColorPicker: () => void;
  onTabChange: (tab: AvatarTab) => void;
  showPresetGallery: boolean;
}) {
  // The sliding indicator is positioned arithmetically, so it needs the tab's
  // index rather than a hard-coded "emoji is the second one".
  const tabCount = showPresetGallery ? 3 : 2;
  const tabIndex = activeTab === "gallery" ? 2 : activeTab === "emoji" ? 1 : 0;
  const triggerClass =
    "relative z-10 h-full rounded-md bg-transparent text-xs font-medium shadow-none transition-colors data-[state=active]:bg-transparent data-[state=active]:shadow-none";

  return (
    <Tabs
      className="w-full"
      onValueChange={(tab) => {
        onTabChange(tab as AvatarTab);
        onCloseCustomColorPicker();
      }}
      value={activeTab}
    >
      <TabsList
        className={cn(
          "relative isolate mb-3 grid h-9 w-full overflow-hidden rounded-lg bg-muted p-0.5",
          showPresetGallery ? "grid-cols-3" : "grid-cols-2",
        )}
      >
        <div
          aria-hidden="true"
          className="absolute bottom-0.5 left-0.5 top-0.5 z-0 rounded-md bg-background shadow-sm transition-transform duration-[250ms] ease-out"
          style={{
            transform: `translateX(${tabIndex * 100}%)`,
            width: `calc((100% - 4px) / ${tabCount})`,
          }}
        />
        <TabsTrigger className={triggerClass} value="image">
          Image
        </TabsTrigger>
        <TabsTrigger className={triggerClass} value="emoji">
          Emoji
        </TabsTrigger>
        {showPresetGallery ? (
          <TabsTrigger className={triggerClass} value="gallery">
            Gallery
          </TabsTrigger>
        ) : null}
      </TabsList>
    </Tabs>
  );
}

/** The 7-column grid of Kubo preset avatars. */
export function AvatarPresetGallery({
  avatarUrl,
  disabled,
  isRoundedSquare,
  onSelect,
  testIdPrefix,
}: {
  avatarUrl: string | null;
  disabled: boolean;
  isRoundedSquare: boolean;
  onSelect: (avatarUrl: string) => void;
  testIdPrefix: string;
}) {
  const selectedPresetId =
    parsePresetAvatarDataUrl(avatarUrl ?? "")?.id ?? null;

  return (
    <div
      className="grid grid-cols-7 gap-1.5"
      data-testid={`${testIdPrefix}-preset-grid`}
    >
      {PRESET_AVATARS.map((preset) => {
        const isSelected = preset.id === selectedPresetId;
        return (
          <button
            aria-label={preset.name}
            aria-pressed={isSelected}
            className={cn(
              "flex h-10 w-10 items-center justify-center overflow-hidden bg-muted outline-hidden transition-[box-shadow,opacity] duration-150 ease-out disabled:pointer-events-none disabled:opacity-50",
              isRoundedSquare ? "rounded-lg" : "rounded-full",
              isSelected
                ? "ring-2 ring-primary ring-offset-2 ring-offset-popover"
                : "hover:opacity-80 focus-visible:ring-2 focus-visible:ring-ring",
            )}
            data-testid={`${testIdPrefix}-preset-${preset.id}`}
            disabled={disabled}
            key={preset.id}
            onClick={() => onSelect(presetAvatarDataUrl(preset))}
            title={preset.name}
            type="button"
          >
            {/* The tile renders the very data URL the click persists, so what
                the user picks is what the agent gets. */}
            <img
              alt=""
              className="h-full w-full"
              src={presetAvatarDataUrl(preset)}
            />
          </button>
        );
      })}
    </div>
  );
}
