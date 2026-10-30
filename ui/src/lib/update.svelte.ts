// The update check result (spec §11.1 update bar): shown at the bottom of
// every screen until dismissed for this launch.
import { api, type UpdateInfo } from "./api";

let available = $state<UpdateInfo | null>(null);
let dismissed = $state(false);

export const update = {
  get banner(): UpdateInfo | null {
    return dismissed ? null : available;
  },
  async check(): Promise<void> {
    try {
      available = await api.update.check();
    } catch {
      // Offline, private repo, or the switch is off: silence (spec §14).
      available = null;
    }
  },
  dismiss() {
    dismissed = true;
  },
  /** For the gallery and tests. */
  set(info: UpdateInfo | null) {
    available = info;
    dismissed = false;
  },
};
