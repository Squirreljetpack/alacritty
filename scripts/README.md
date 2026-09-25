# Scripts

## Build and Run

Builds the settings app (`$HOME/gh/_fzs/commandspace-settings`) and a release
CommandSpace, then launches it.

On macOS the release `CommandSpace.app` bundle is created and opened. On Linux
the release binary is launched in the background; a running instance is stopped
first, and output goes to `~/.cache/commandspace/build-and-run.log`.

```sh
./build-and-run.sh
```

## Flamegraph

Run the release version of Alacritty while recording call stacks. After the
Alacritty process exits, a flamegraph will be generated and it's URI printed
as the only output to STDOUT.

```sh
./create-flamegraph.sh
```

Running this script depends on an installation of `perf`.

## ANSI Color Tests

We include a few scripts for testing the color of text inside a terminal. The
first shows various foreground and background variants. The second enumerates
all the colors of a standard terminal. The third enumerates the 24-bit colors.

```sh
./fg-bg.sh
./colors.sh
./24-bit-colors.sh
```
