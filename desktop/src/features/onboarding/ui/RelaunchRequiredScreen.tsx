import { RecoveryScreen } from "./RecoveryScreen";

export function RelaunchRequiredScreen() {
  return (
    <RecoveryScreen
      testId="relaunch-required"
      title="Restart Kura to finish recovery"
      body="Your identity was updated. Kura needs to restart so syncing and agents run under it."
    />
  );
}
