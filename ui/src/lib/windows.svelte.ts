// The secondary windows (spec §11): All Sessions, Settings, About,
// Onboarding. Drawn over the main window, one at a time.
export type WindowName = "sessions" | "settings" | "about" | "onboarding";

let open = $state<WindowName | null>(null);

export const windows = {
  get open(): WindowName | null {
    return open;
  },
  show(name: WindowName) {
    open = name;
  },
  close() {
    open = null;
  },
};
