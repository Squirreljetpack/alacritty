1. Global hotkey

- Toggle window
- Restart (?)
  - launch another program (settings)
- Center
- Hide on click away (toggleable)

2. Panel window (macOS)

3. Task tray

4. Simplify configuration

- No decorations, no tabs
- No subcommands?
- Remove WindowIdentity
- Figure out what you need to pass to new windows: command and anything else, and how to support that from cli.

5. redo configuration:
- Need to make binding serialize/deserializable from scratch, using library codes for easier maintenance.
- Need to include mode-awareness.

5. Change logo/name/plists

Where did I break in logging?

Config

- for now, lets put all new options into general
- command:

replace clear with shader

Launch style:
read config
overrides: some debug options


# Config

Binds:
  - Hotkey -> Action
  -