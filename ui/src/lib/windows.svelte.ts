// The secondary windows (spec §11, §16.4): Settings, About, Onboarding,
// the Panorama preview. Drawn over the main window, one at a time. All Sessions is a screen of the
// detail area, selected from the sidebar like a Game.
export type WindowName = "settings" | "about" | "onboarding" | "panorama";

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
