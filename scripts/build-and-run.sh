#!/usr/bin/env bash
#
# Build the settings app and CommandSpace, then launch it.
#
# The settings app lives at the fixed path the terminal launches it from (see
# `settings_command()` in commandspace_config/src/paths.rs).

set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/.." && pwd)"

settings_dir="$HOME/gh/_fzs/commandspace-settings"
settings_manifest="$settings_dir/src-tauri/Cargo.toml"

if [[ ! -f "$settings_manifest" ]]; then
	echo "error: settings app not found at $settings_dir" >&2
	echo "hint: it must live where settings_command() looks for it" >&2
	exit 1
fi

echo "==> Building settings app ($settings_dir)"
if [[ ! -d "$settings_dir/node_modules" ]]; then
	npm --prefix "$settings_dir" install
fi
npm --prefix "$settings_dir" run build
cargo build --release --manifest-path "$settings_manifest"

case "$(uname -s)" in
Darwin)
	app_bundle="$repo_root/target/release/osx/CommandSpace.app"

	echo "==> Building $app_bundle"
	make -C "$repo_root" app

	echo "==> Opening $app_bundle"
	open "$app_bundle"
	;;
Linux)
	binary="$repo_root/target/release/commandspace"

	echo "==> Building $binary"
	cargo build --release --manifest-path "$repo_root/Cargo.toml"

	if pkill -x commandspace 2>/dev/null; then
		echo "==> Waiting for the running CommandSpace to stop"
		for _ in {1..20}; do
			pgrep -x commandspace >/dev/null || break
			sleep 0.1
		done
		pkill -KILL -x commandspace 2>/dev/null || true
	fi

	log_file="${XDG_CACHE_HOME:-$HOME/.cache}/commandspace/build-and-run.log"
	mkdir -p "$(dirname "$log_file")"

	nohup "$binary" >"$log_file" 2>&1 &
	echo "==> Launched $binary (pid $!, log: $log_file)"
	;;
*)
	echo "error: unsupported platform: $(uname -s)" >&2
	exit 1
	;;
esac
