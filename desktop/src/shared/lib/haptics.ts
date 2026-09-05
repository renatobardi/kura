import { isTauri } from "@tauri-apps/api/core";
import { platform } from "@/platform";

export function performDefaultHaptic() {
  if (!isTauri()) {
    return;
  }

  void platform.invoke("perform_sidebar_default_haptic").catch(() => {});
}

export function performSidebarDefaultHaptic() {
  performDefaultHaptic();
}
