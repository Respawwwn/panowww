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
        hold: "How long the key stays pressed for one Step. Calibrate with Check overlap.",
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
        hold: "How long the stick stays pushed for one Step. Calibrate with Check overlap.",
      },
      vertical: "Vertical",
      invertVertical: "Invert",
      msPerStep: "ms per Step",
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
      rows: "Turn test off its landmark? Change Shots per Row. Pitch test off the pole? Change Rows.",
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
      decrease: "Decrease",
      increase: "Increase",
      turn: "Turn",
      pitch: "Pitch",
      turnTestTitle: "Turn test",
      overlapTurnTitle: "Check overlap",
      pitchTestTitle: "Pitch test",
      overlapPitchTitle: "Check overlap",
      stepOnceTitle: "Step once",
      running: "{tool} running…",
      overlapTurn:
        "Aim at the horizon first. Takes two Samples one turn Step apart and superimposes them; slide until they line up. Too little Overlap? Reduce the Step (Overshot). Too much? Increase it (Undershot). Then check again.",
      turnTest:
        "Turns the camera a full circle, Step by Step, and should end exactly where it started. Went past the start? Decrease Shots per Row. Stopped short? Increase them. Then run it again.",
      overlapPitch:
        "Aim at the horizon first. Takes two Samples one pitch Step apart and superimposes them; slide until they line up. Too little Overlap? Reduce the Step (Overshot). Too much? Increase it (Undershot). Then check again.",
      pitchTest: {
        nadir:
          "Point the camera straight down first. The test tilts it up, Step by Step, and should end exactly straight up. Went past the top? Decrease Rows. Stopped short? Increase them. Then run it again; reposition the camera by hand afterwards.",
        zenith:
          "Point the camera straight up first. The test tilts it down, Step by Step, and should end exactly straight down. Went past the bottom? Decrease Rows. Stopped short? Increase them. Then run it again; reposition the camera by hand afterwards.",
      },
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
        turn: "{p} % Overlap, too little: the turn Step is too big → Overshot (−) in the Calibrate card.",
        pitch:
          "{p} % Overlap, too little: the pitch Step is too big → Overshot (−) in the Calibrate card.",
      },
      high: {
        turn: "{p} % Overlap, more than needed: the turn Step is too small → Undershot (+) in the Calibrate card.",
        pitch:
          "{p} % Overlap, more than needed: the pitch Step is too small → Undershot (+) in the Calibrate card.",
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
      next: {
        stitchAndOpen: "Next: {stitchHere}, or open it in {stitchers}.",
        stitchOrDrop: "Next: {stitchHere}, or drop the folder or the zip into a Stitcher.",
        openIn: "Next: open the Session in {stitchers}, or drop the folder or the zip into one.",
        drop: "Next: open the Session in a Stitcher, or drop the folder or the zip into one.",
        stitchHere: "stitch the Session here",
        /** Between two Stitchers in `{stitchers}`. */
        or: " or ",
      },
      stitch: "Stitch",
      stitching: "Stitching… {pct} %",
      previewPanorama: "Preview Panorama",
      stitchAgain: "Stitch again",
      /** `{file}` is drawn as code. */
      panorama: "Panorama stitched {when} · {file} · {width} × {height} · {size}",
      openIn: "Open in {stitcher}",
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
    stitchersTitle: "Stitchers",
    stitchers:
      "A Stitcher assembles a Session's Shots into a panorama. Set one and a completed Session gets an Open in… button.",
    stitchersNote: "Panowww never stitches itself.",
    stitchersNoteExperimental:
      "Panowww stitches itself only with Stitch in Panowww, under Experimental.",
    stitcher: { ptgui: "PTGui", autopanoGiga: "Autopano Giga" },
    stitcherNotSet: "Not set",
    stitcherVersion: "{stitcher} version: {version}",
    stitcherVersionUnknown: "not detected",
    stitcherNotInstalled: "not installed",
    stitcherWrongProgram: "not {stitcher} ({product})",
    stitcherUnknownProgram: "unknown program",
    recheck: "Check again",
    stitcherHint: {
      macos: "Pick the application.",
      windows: "Pick the program's .exe, for example PTGui.exe or AutopanoGiga_x64.exe.",
    },
    choose: "Choose…",
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
    experimental: {
      title: "Experimental",
      chip: "Beta",
      intro: "Features still in the making. They may be slow, change, or go away.",
      stitch: {
        label: "Stitch in Panowww",
        hint: "A completed Session gets a Stitch button. Panowww assembles the Shots itself, on every core, and saves the Panorama in the Session folder. No editing: if the result is off, adjust the Game's Steps and capture again.",
      },
    },
  },
  /** The Stitch panel and the Panorama preview (spec §16.3, §16.4, §16.7). */
  stitch: {
    title: {
      idle: "Stitch the {n} Shots",
      running: "Stitching the {n} Shots…",
      finished: "Panorama stitched in {duration}",
      failed: "Could not stitch this Session",
      /** The red chip beside the failed title (board StitchFailed). */
    },
    subtitle: "{date} · {rows} Rows × {shotsPerRow} · {w} × {h} Shots",
    /** Once a Panorama exists: its size instead of the Shots'. */
    subtitleFinished: "{date} · {w} × {h} px",
    willWrite: "Writes panorama.jpg in the Session folder, about {w} × {h}.",
    stage: {
      read: "Read the Shots",
      overlaps: "Find the Overlaps",
      align: "Align",
      alignRow: "Align · Row {row} of {rows}",
      render: "Render the sphere",
      blend: "Blend the seams",
      save: "Save 360° panorama",
    },
    progress: {
      time: "{elapsed} elapsed · about {left} left",
      cores: "Uses every core. The game can keep running: nothing is captured.",
    },
    options: {
      title: "Options",
      antiGhost: "Anti-ghost",
      antiGhostHint: "Shots meet where they agree best, so a moving object is not doubled.",
      unaligned: "Shots without control points",
      placeFromGrid: "Place from the grid",
      leaveOut: "Leave out",
      leaveOutHint: "What only these Shots saw stays black.",
      quality: "JPEG quality",
      /** The slider's value and each preset button. */
      percent: "{n} %",
    },
    measured: {
      title: "Measured from the Shots",
      /** The grid of the Session, the Stitch's starting guess: first in the card. */
      rows: "Rows",
      shotsPerRow: "Shots per Row",
      pending:
        "Field of view, Steps and drift, once the Stitch runs. Nothing is taken from the Game's numbers.",
      fov: "Field of view",
      turnStep: "Turn Step",
      pitchStep: "Pitch Step",
      startPole: "Start pole",
      closing: "Closing turn",
      drift: "Drift per Shot",
      fovValue: "{deg}° wide",
      poleShort: "{deg}° short of the {pole}",
      polePast: "{deg}° past the {pole}",
      poleOn: "on the {pole}",
      closingShort: "{deg}° short",
      closingLong: "{deg}° long",
      closes: "closes",
      driftValue: "under {deg}°",
    },
    shots: {
      reading: "reading and comparing…",
      aligning: "Row {row} of {rows} · {k} / {n} Shots aligned",
      aligned: "{n} / {n} Shots aligned",
      /** Caption of the failed grid (board StitchFailed). */
      linked: "{k} / {n} Shots linked",
      /** Caption before a Stitch. */
      count: "{n} Shots",
      /** Between the last two names of a list of Shots. */
      and: "and",
      legend:
        "Row 1 is the start pole of this Game, the {pole}. The framed Row is being aligned; dimmed Shots are waiting.",
      legendUnaligned: "Amber: Shots placed from the grid, not aligned.",
      legendLeftOut: "Amber: Shots left out, without control points.",
      legendFailed:
        "Red: Shots without any control points. They look alike: sky, fog, a dark scene, or a camera that did not move.",
    },
    /** The heading's chip: the Panorama's state, not the Session's. */
    chip: {
      idle: "Not stitched",
      running: "Stitching",
      finished: "Panorama stitched",
      failed: "Stitch failed",
    },
    unaligned: {
      chip: { one: "1 Shot not aligned", other: "{n} Shots not aligned" },
      /** The same Shots when the Options card left them out. */
      chipLeftOut: { one: "1 Shot left out", other: "{n} Shots left out" },
      boxLeftOut:
        "Panowww was unable to detect control points for some of the Shots. {list} do not have any control points linking them to the other Shots, so they were left out: what only they saw is black.",
      box: "Panowww was unable to detect control points for some of the Shots. {list} do not have any control points linking them to the other Shots, so they were placed from the grid, not aligned.",
    },
    dropped: {
      chip: { one: "1 Step did not move", other: "{n} Steps did not move" },
      box: "The camera did not move for some Steps. {pairs} are the same image, so that Row turned {deg}° short and its last Shots were placed by their neighbours.",
    },
    failed: {
      tooFew:
        "Too few Shots could be linked. Panowww was unable to detect control points for {k} of the {n} Shots, so the field of view cannot be measured and the sphere cannot be placed.",
      other: "{message} Nothing was saved.",
      stitcherHint: "A Stitcher may still manage it. It searches the whole image for matches.",
      noStitcher:
        "Try a Stitcher. PTGui and Autopano Giga let you add control points by hand, so they may still stitch this Session.",
      setOne: "Set one in Settings",
      fixGame: "To fix the Game itself, run Check overlap, then capture a new Session.",
    },
    cancelled: {
      byYou: "Stopped. Nothing was saved.",
      session: "Stopped: a Session started. Nothing was saved.",
    },
    actions: {
      cancel: "Cancel",
      /** The idle panel's primary button, in the actions row and the Options card. */
      start: "Start Stitch",
      tryAgain: "Try again",
      /** `{file}` is drawn as code where the layout allows. */
      replace: "Replace {file}?",
      backToSession: "Back to Session",
      done: "Done",
      /** The preview takes the whole screen, the Panorama alone. */
      fill: "Fill the screen",
      /** Back from the whole screen to the preview window. */
      reduce: "Reduce the window",
    },
    preview: {
      title: "Panorama",
      /** Chip in the title bar: when the Panorama was stitched. */
      chip: "Stitched {when}",
      hint: "Drag to look around · scroll to zoom",
      fov: "FOV {deg}°",
      noWebgl: "This window cannot show the 360° view. The Panorama is saved: open it in a viewer.",
    },
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

/** A counted string: `one` for exactly 1, `other` for the rest, with `{n}`. */
export interface Plural {
  readonly one: string;
  readonly other: string;
}

/** Picks the form of `forms` for `n` and fills `{n}` and `values`. */
export function plural(forms: Plural, n: number, values: InterpolationValues = {}): string {
  return interpolate(n === 1 ? forms.one : forms.other, { n, ...values });
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

export type PlaceholderSegment = { kind: "text"; text: string } | { kind: "slot"; name: string };

/**
 * Splits a template around the `{name}` placeholders listed in `slots`, so a
 * renderer can put a link or a component there; every other placeholder
 * stays text for `interpolate`.
 */
export function splitPlaceholders(
  template: string,
  slots: readonly string[],
): PlaceholderSegment[] {
  const segments: PlaceholderSegment[] = [];
  let last = 0;
  for (const match of template.matchAll(/\{(\w+)\}/g)) {
    const name = match[1] ?? "";
    if (!slots.includes(name)) {
      continue;
    }
    const index = match.index ?? 0;
    if (index > last) {
      segments.push({ kind: "text", text: template.slice(last, index) });
    }
    segments.push({ kind: "slot", name });
    last = index + match[0].length;
  }
  if (last < template.length) {
    segments.push({ kind: "text", text: template.slice(last) });
  }
  return segments;
}
