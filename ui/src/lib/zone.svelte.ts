// The Capture zone state of each Game on this machine, shared by the
// Setup's zone card and the toolbar's Start button so a zone drawn in the
// picker enables both at once.
import { api, type ZoneState } from "./api";

let states = $state<Record<string, ZoneState | null>>({});

export const zones = {
  get(id: string): ZoneState | null {
    return states[id] ?? null;
  },
  ready(id: string): boolean {
    return states[id]?.status === "ready";
  },
  set(id: string, state: ZoneState | null): void {
    states = { ...states, [id]: state };
  },
  async load(id: string): Promise<ZoneState | null> {
    const state = await api.zone.get(id).catch(() => null);
    this.set(id, state);
    return state;
  },
};
