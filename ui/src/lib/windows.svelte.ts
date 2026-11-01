// The secondary windows (spec §11): Settings, About, Onboarding. Drawn
// over the main window, one at a time. All Sessions is a screen of the
// detail area, selected from the sidebar like a Game.
export type WindowName = "settings" | "about" | "onboarding";

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
