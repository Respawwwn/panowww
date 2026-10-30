// Every user-visible string of the UI (spec §13). Issue 14 fills the whole
// module; the shell needs these few now, keyed as §13 names them.
export const strings = {
  app: {
    name: "Panowww",
  },
  sidebar: {
    new: "New",
    import: "Import…",
    allSessions: "All Sessions",
    settings: "Settings",
    about: "About",
  },
  detail: {
    setup: "Setup",
    sessions: "Sessions",
  },
  empty: {
    title: "Add your first Game",
    body: "A Game holds the camera moves, the grid and the delays for one title. Set it up once, then shoot as many panoramas as you like.",
    import: "Someone already set up this title? Import their shared .panowww.json file.",
    newButton: "New Game",
    importButton: "Import Game…",
  },
} as const;
