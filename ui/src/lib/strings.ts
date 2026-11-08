// Every user-visible string of the UI, keyed as spec §13 names them. Rules
// of §13: short sentences, imperative or "you", never an exclamation mark,
// glossary terms capitalised. `{name}` is a value filled by `interpolate`;
// `[start]` and `[stop]` are the two global shortcuts, rendered as key caps
// from the current Settings by `RichLine`, never written out.

export const strings = {
  app: {
    name: "Panowww",
  },
  sidebar: {
    games: "Games",
    noGames: "No Game yet.",
    new: "New",
    import: "Import",
    allSessions: "All Sessions",
    settings: "Settings",
    about: "About",
    damaged: "Damaged file",
  },
  detail: {
    setup: "Setup",
    sessions: "Sessions",
    duplicate: "Duplicate",
    share: "Share",
    reveal: "Reveal",
    delete: "Delete",
    deleteConfirm: "Delete this Game?",
    revealInFinder: "Reveal in Finder",
    revealInExplorer: "Reveal in Explorer",
    startSession: "Start Session",
  },
  setup: {
    game: {
      title: "Game & input",
      name: "Name",
      notes: "Notes",
      notesPlaceholder: "Photo mode, HUD off, borderless 2560 × 1440, weather settled.",
      device: "Input device",
    },
    input: {
      keyboard: {
        turnRight: "Turn right",
        lookUp: "Look up",
        lookDown: "Look down",
        turnHold: "Turn hold",
        pitchHold: "Pitch hold",
        change: "Change",
        cancel: "Cancel",
        pressKey: "Press a key",
        keys: "Three distinct keys, no modifiers. Panowww presses the key at that position on your keyboard, whatever the layout.",
        hold: "How long the key stays pressed for one Step. Calibrate with the Turn and Pitch tests.",
      },
      mouse: {
        modeLabel: "Mode",
        move: "Move",
        drag: "Drag",
        dragButton: "Drag button",
        left: "Left",
        right: "Right",
        middle: "Middle",
        turnDelta: "Turn delta",
        pitchDelta: "Pitch delta",
        unit: "units per Step",
        mode: "Move: the cursor alone. Drag: a button held while moving, what most free cameras expect.",
        crossOs:
          "Mouse values are not promised to match between macOS and Windows: recalibrate on each.",
      },
      gamepad: {
        deflectionLabel: "Stick deflection",
        unit: "of full travel",
        unavailableMac:
          "Virtual gamepad is unavailable on macOS. You can set this Game up here, but it only runs on Windows, where it needs ViGEmBus.",
        deflection:
          "Needs ViGEmBus. Right stick of a virtual Xbox 360 controller, as a share of full travel.",
        hold: "How long the stick stays pushed for one Step. Calibrate with the Turn and Pitch tests.",
      },
      vertical: "Vertical",
      invertVertical: "Invert",
    },
    grid: {
      title: "Grid & timing",
      startPoleLabel: "Start pole",
      nadir: "Nadir",
      zenith: "Zenith",
      shotsPerRowLabel: "Shots per Row",
      rowsLabel: "Rows",
      waitAfterMoveLabel: "Wait after move",
      waitAfterShotLabel: "Wait after Shot",
      startPole: {
        nadir: "Start looking straight down. The first Row is the ground, the last one the sky.",
        zenith: "Start looking straight up. The first Row is the sky, the last one the ground.",
      },
      shotsPerRow: "Each Row ends with a closing turn back to the start heading.",
      rows: "Off-target Overlap? Change Shots per Row or Rows, then recalibrate.",
      waitAfterMove: "Motion blur in the Shots → raise this.",
      waitAfterShot: "A swallowed Step → raise this.",
    },
    start: {
      shortcutStart: "Start from the game  [start]",
      shortcutStop: "Stop  [stop]",
      disabled: {
        noZone: "Set the Capture zone first.",
        gamepadMac: "Gamepad Games run on Windows only.",
        damaged: "Fix the file to start a Session.",
      },
      /** `≈ 4 min 20 s`, then `78 Shots · 6 Rows × 13 · from nadir · wait 400 / 250 ms` */
      estimate: "{duration}",
      estimateDetail:
        "{shots} Shots · {rows} Rows × {shotsPerRow} · from {pole} · wait {move} / {shot} ms",
    },
    zone: {
      title: "Capture zone",
      set: "Display {n} · {w} × {h} at ({x}, {y})",
      notSet: "Not set",
      displayChanged: "Display {n} is gone or its resolution changed. Draw the zone again.",
      hdr: "HDR is on for Display {n}. Shots come out washed out: turn HDR off in Windows or in the game.",
      setZone: "Set zone",
      changeZone: "Change zone",
      showZone: "Show zone",
      hideZone: "Hide zone",
    },
    calibrate: {
      title: "Calibrate",
      countdown: "Tests start after a 5 s countdown: switch to the game.",
      run: "Run",
      open: "Open",
      stop: "Stop",
      overshot: "Overshot",
      undershot: "Undershot",
      turn: "Turn",
      pitch: "Pitch",
      turnTestTitle: "Turn test",
      overlapTurnTitle: "Check overlap",
      pitchTestTitle: "Pitch test",
      overlapPitchTitle: "Check overlap",
      stepOnceTitle: "Step once",
      running: "{tool} running…",
      turnTest:
        "Turns the camera a full circle, Step by Step, and should end exactly where it started. Went past the start? Reduce (overshot). Stopped short? Increase (undershot). Then run it again.",
      overlapTurn:
        "Aim at the horizon first. Off-target Overlap → change Shots per Row, then back to step 1.",
      pitchTest: {
        nadir:
          "Point the camera straight down first. The test tilts it up, Step by Step, and should end exactly straight up. Went past the top? Reduce (overshot). Stopped short? Increase (undershot). Then run it again; reposition the camera by hand afterwards.",
        zenith:
          "Point the camera straight up first. The test tilts it down, Step by Step, and should end exactly straight down. Went past the bottom? Reduce (overshot). Stopped short? Increase (undershot). Then run it again; reposition the camera by hand afterwards.",
      },
      overlapPitch: "Off-target Overlap → change Rows, then back to step 3.",
      stepOnce: "One Step on its own, to feel its size.",
      result: {
        turnTest: "Turn test done · {n} Steps. Back on the landmark?",
        pitchTest: "Pitch test done · {n} Steps. On the opposite pole?",
        stepOnce: "Step done.",
        stopped: "Test stopped.",
      },
      nothingMoved: {
        link: "Nothing moved?",
        detected: "The screen did not change during the Test.",
      },
    },
  },
  zonePicker: {
    instruction: "Draw the zone over the game. Esc cancels.",
    tip: "Game in borderless windowed mode. Keep HUD and menus out of the zone.",
    display: "Display {n} · {w} × {h} @{s}x",
    size: "{w} × {h}",
    showFrame: "Show frame on screen",
    cancel: "Cancel",
    save: "Save zone",
  },
  overlap: {
    instruction: "Two Samples one Step apart, superimposed. Slide until they line up.",
    title: "Check overlap · {game}",
    target: "Target 25–35 %",
    label: {
      ok: "Good",
      low: "Too little",
      high: "Too much",
    },
    sample1: "Sample 1",
    sample2: "Sample 2",
    chip: "Overlap {p} %",
    align: "Align",
    waiting: "Taking the Samples…",
    none: "No Samples yet. Open Check overlap from the Calibrate card.",
    verdict: {
      ok: "{p} % Overlap, in the 25–35 % target.",
      low: {
        turn: "{p} % Overlap, too little → more Shots per Row.",
        pitch: "{p} % Overlap, too little → more Rows.",
      },
      high: {
        turn: "{p} % Overlap, more than needed → fewer Shots per Row.",
        pitch: "{p} % Overlap, more than needed → fewer Rows.",
      },
    },
    tip: "Aim at the horizon before taking Samples. Samples stay in memory, nothing is saved.",
    retake: "Retake Samples",
    done: "Done",
  },
  countdown: {
    starting: "Starting",
    title: "Switch to the game",
    body: "The Session starts when the countdown ends. Then leave the mouse and the keyboard alone: {n} Shots, about {duration}.",
    bodyTool:
      "The {tool} runs when the countdown ends. Leave the mouse and the keyboard alone until it is done.",
    stop: "Stop at any time with [stop]",
    cancel: "Cancel",
    error: {
      title: "Cannot start",
      accessibility:
        "Accessibility permission is missing. Panowww cannot move the camera without it.",
      screenRecording:
        "Screen Recording permission is missing. Panowww cannot capture the zone without it.",
      vigem: "ViGEmBus is not installed. Gamepad Games need it.",
      displayGone: "Display {n} is gone or its resolution changed. Draw the zone again.",
      captureUnavailable:
        "The display could not be captured. Close other capture tools, then try again.",
      back: "Back",
      openSystemSettings: "Open System Settings",
      changeZone: "Change zone…",
      installVigem: "Install ViGEmBus ↗",
      tryAgain: "Try again",
    },
  },
  session: {
    running: {
      title: "Session running",
      hint: "The game keeps the focus. Do not touch the mouse or the keyboard. Stop with [stop]",
      live: "live",
      rowOf: "of {rows}",
      identical: "The first two Shots are identical. Nothing moved? Stop and run a Turn test.",
      row: "Row {row} of {rows}",
      rowOnly: "Row {row}",
      shots: "{taken} / {expected} Shots",
      remaining: "{duration} remaining",
      stop: "Stop",
    },
    preview: {
      hide: "Hide preview",
      show: "Show preview",
    },
    end: {
      completed: { title: "All {n} Shots are in" },
      interrupted: {
        title: "Stopped after {n} Shots",
        note: "The Shots taken are kept. An interrupted Session cannot be resumed: start a new one from the same pole.",
      },
      failed: { title: "Failed at Shot {n}" },
      next: "Next: drop the folder or the zip into PTGui or Autopano Giga.",
      zip: "Zip Session",
      backToGame: "Back to Game",
      shots: "{n} Shots",
      shotsLabel: "Shots",
      onDisk: "on disk",
      grid: "{rows} Rows × {shotsPerRow}",
      png: "PNG, lossless",
      chip: {
        completed: "Session completed",
        interrupted: "Session interrupted",
        failed: "Session failed",
        running: "Session running",
      },
    },
  },
  test: {
    running: {
      hint: "Leave the mouse and the keyboard alone. Stop with [stop].",
    },
  },
  sessions: {
    empty: "No Session yet. Start one from the Setup tab.",
    folder: "Sessions are saved in",
    changeFolder: "Change the folder in Settings",
    deleteConfirm: "Delete the whole folder?",
    deleteAll: "Delete all",
    deleteAllConfirm: "Delete all {n} Sessions of this Game?",
    failed: {
      why: "Why did it fail?",
      template: "Stopped at Shot {n}: {reason} The {taken} Shots taken are kept in the folder.",
      reason: {
        captureUnavailable: "the display could not be captured.",
        displayGone: "Display {d} was disconnected.",
        permissionRevoked: "Screen Recording permission was revoked.",
        shotFailed: "the Shot failed twice in a row.",
      },
    },
    today: "Today",
    yesterday: "Yesterday",
    columns: {
      when: "When",
      game: "Game",
      state: "State",
      shots: "Shots",
      size: "Size",
    },
    state: {
      completed: "Completed",
      interrupted: "Interrupted",
      failed: "Failed",
      running: "Running",
    },
    gone: "This Session folder is gone.",
    zip: "Zip",
    reveal: "Reveal",
    delete: "Delete",
    cancel: "Cancel",
    zipped: "Zipped",
    title: "All Sessions",
    summary: "{n} Sessions · {size}",
    folderShort: "Sessions folder: {path}",
    revealFolder: "Reveal folder",
    changeInSettings: "Change in Settings…",
  },
  overlay: {
    done: "Done · {n} Shots",
    stopped: "Stopped · {k} / {n}",
  },
  empty: {
    homeTitle: "Choose a Game",
    homeBody: "Pick a Game on the left to set it up and start a Session, or add a new one.",
    title: "Add your first Game",
    body: "A Game holds the camera moves, the grid and the delays for one title. Set it up once, then shoot as many panoramas as you like.",
    import: "Someone already set up this title? Import their shared .panowww.json file.",
    newButton: "New Game",
    importButton: "Import Game",
  },
  damaged: {
    title: "This Game file cannot be read",
    where: "{file} · in the Panowww data folder",
    fix: "Fix the file to start a Session",
    body: "Edit the file by hand and save it, Panowww reloads it. Or delete the Game and set it up again.",
  },
  onboarding: {
    title: "Welcome to Panowww",
    later: "Later",
    continue: "Continue",
    mac: {
      subtitle: "Two permissions before the first Session",
      intro:
        "Panowww moves the camera for you and captures the game. macOS asks you to allow both.",
      accessibility: {
        name: "Accessibility",
        ok: "Granted · lets Panowww send keys and mouse moves to the game",
        missing: "Not granted · needed to move the camera",
      },
      screenRecording: {
        name: "Screen Recording",
        ok: "Granted · lets Panowww capture the zone",
        missing: "Not granted · needed to capture the zone",
      },
      stuck:
        "Ticked in System Settings but nothing happens? Remove Panowww from the list and add it again, then relaunch.",
      picker: {
        chip: "macOS 26",
        title: "A system window picker may appear at the first capture.",
        body: "Click Allow to let Panowww bypass it. Without that, the picker interrupts the Session.",
      },
      gamepad: {
        chip: "Gamepad",
        title: "Virtual gamepad unavailable on macOS.",
        body: "Games that need a controller can be shot on Windows only. Keyboard and mouse work here.",
      },
    },
    win: {
      subtitle: "Before the first Session",
      admin:
        "Panowww must run as administrator. Windows asks at every launch; this is normal. Elevated, Panowww can inject input into games that run elevated themselves, which Windows would otherwise block.",
      vigem: "Virtual controller driver, only needed for gamepad Games.",
      vigemTip: "Keyboard and mouse Games work without ViGEmBus.",
      installVigem: "Install ViGEmBus ↗",
    },
  },
  settings: {
    title: "Settings",
    sessionsTitle: "Sessions",
    previewTitle: "Display preview",
    recorder: {
      press: "Press the new combo…",
      f12: "F12 cannot be used: games and browsers own it.",
      noModifier: "Add a modifier such as Ctrl or Shift.",
      winKey: "The Win key cannot be part of a shortcut.",
      optionOnly: "Option alone, or with Shift, types characters: add Ctrl or Cmd.",
    },
    sessionsFolder: "Sessions folder",
    folderLabel: "Folder",
    change: "Change",
    reveal: "Reveal",
    folder: "One folder per Game, one timestamped folder per Session.",
    shortcutsTitle: "Shortcuts",
    start: "Start",
    stop: "Stop",
    record: "Record",
    reset: "Reset",
    shortcuts: "Pick a combo the game does not use. The game keeps the focus when you press it.",
    shortcutsMac: "On a Mac laptop, avoid the F-keys: they act as media keys unless fn is held.",
    defaultsTitle: "New Game defaults",
    defaults: "Applied to every new Game. Existing Games keep their values.",
    displayPreview: {
      label: "Show the Display preview during a countdown and a Session",
      hint: "Turn it off if the game stutters while Panowww runs.",
    },
    general: "General",
    appearance: {
      label: "Appearance",
      auto: "Auto",
      light: "Light",
      dark: "Dark",
    },
    updatesTitle: "Updates",
    checkForUpdates: "Check for updates at launch",
    includePrereleases: {
      label: "Include pre-releases",
      hint: "Also offer betas and release candidates. Off, only stable releases show up.",
    },
    reviewPermissions: "Review permissions…",
    revealLogs: "Reveal logs",
  },
  about: {
    links: {
      site: "respawwwn.com",
      discord: "Discord",
      kofi: "Ko-fi",
      github: "GitHub",
    },
    tagline: "Automated 360° panorama capture for video games.",
    by: "By",
    brand: "Respawwwn",
    licence:
      "Panowww is a Respawwwn project by Kevin Wenger. MIT License. The Respawwwn name and logo are not covered by the license.",
    upToDate: "You're up to date.",
    title: "Panowww",
    version: "Version {version}",
  },
  update: {
    banner: "Panowww {version} is available · See what's new",
  },
  import: {
    title: "This Game is already here",
    body: "{existing} has the same id as the file you are importing ({incoming}). Replace it, keeping its Capture zone, or keep both as separate Games.",
    replace: "Replace",
    keepBoth: "Keep both",
    cancel: "Cancel",
  },
  nothingMoved: {
    title: "Nothing moved?",
    intro:
      "Panowww sends the keys and mouse moves but gets nothing back from the game. Check, in this order:",
    fullscreen: "The game runs in borderless windowed mode, not exclusive fullscreen.",
    focus: "The game had the focus when the countdown ended. Click into it next time.",
    keyboard:
      "The three keys match the game's camera bindings. Panowww presses the key at that position on your keyboard: check them on your layout.",
    mouse:
      "Switch between Move and Drag, or try another drag button. Some games read the mouse only after a click inside their window.",
    gamepad:
      "ViGEmBus is installed and the game accepts an Xbox 360 controller. Unplug physical controllers: the virtual pad must be player 1. Some games only look for controllers at launch: restart the game once after the first countdown.",
    mac: {
      controllerMouse:
        "Some Mac games read the mouse through the game-controller path and ignore injected moves. Use the game's keyboard camera controls if it has them.",
    },
    win: {
      rawInput:
        "Some games read the mouse and keyboard straight from the device and ignore injected input. Use a gamepad Game for them.",
    },
    antiCheat:
      "Multiplayer games with anti-cheat block injected input and may ban you for it. Use Panowww offline, in single-player.",
  },
} as const;

export type InterpolationValues = Record<string, string | number>;

/**
 * Fills every `{name}` of `template` from `values`. A missing value leaves
 * the placeholder in place so the gap is visible rather than silent.
 */
export function interpolate(template: string, values: InterpolationValues = {}): string {
  return template.replace(/\{(\w+)\}/g, (placeholder, name: string) => {
    const value = values[name];
    return value === undefined ? placeholder : String(value);
  });
}

/** Which global shortcut a `[…]` marker stands for. */
export type ShortcutRef = "start" | "stop";

export type LineSegment = { kind: "text"; text: string } | { kind: "keys"; shortcut: ShortcutRef };

/**
 * Splits a string around its `[start]` / `[stop]` markers so a renderer can
 * draw key caps there. Other bracketed text stays text.
 */
export function splitKeyCaps(text: string): LineSegment[] {
  const segments: LineSegment[] = [];
  const pattern = /\[(start|stop)\]/g;
  let last = 0;
  for (const match of text.matchAll(pattern)) {
    const index = match.index ?? 0;
    if (index > last) {
      segments.push({ kind: "text", text: text.slice(last, index) });
    }
    segments.push({ kind: "keys", shortcut: match[1] as ShortcutRef });
    last = index + match[0].length;
  }
  if (last < text.length) {
    segments.push({ kind: "text", text: text.slice(last) });
  }
  return segments;
}
