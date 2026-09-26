#!/usr/bin/env python3
import argparse
import os
import subprocess
import sys
import time
import tomllib
from pathlib import Path

DEFAULT_THEMES_DIR = Path.home() / ".config" / "commandspace" / "themes"
CONFIG_PATH = Path.home() / ".config" / "commandspace" / "config.toml"
DEFAULT_OUTPUT_DIR = Path("/tmp")

def to_toml_val(v):
    if isinstance(v, bool):
        return "true" if v else "false"
    elif isinstance(v, (int, float)):
        return str(v)
    elif isinstance(v, str):
        escaped = v.replace("\\", "\\\\").replace("\"", "\\\"").replace("\n", "\\n").replace("\t", "\\t")
        return f"\"{escaped}\""
    elif isinstance(v, list):
        return "[" + ", ".join(to_toml_val(x) for x in v) + "]"
    elif isinstance(v, dict):
        items = ", ".join(f"{k} = {to_toml_val(val)}" for k, val in v.items())
        return f"{{ {items} }}"
    return str(v)

def dump_toml(d, prefix=""):
    scalars = {}
    tables = {}
    for k, v in d.items():
        if isinstance(v, dict) and not all(not isinstance(sub, dict) for sub in v.values()):
            tables[k] = v
        elif isinstance(v, dict) and any(isinstance(sub, dict) for sub in v.values()):
            tables[k] = v
        elif isinstance(v, dict) and prefix:
            tables[k] = v
        elif isinstance(v, dict) and k in [
            "padding", "frame", "primary", "cursor", "vi_mode_cursor",
            "selection", "normal", "bright", "dim", "identity", "class",
            "dimensions", "search", "hints", "line_indicator", "footer_bar",
            "offset", "glyph_offset", "lost_focus_ignore_duration"
        ]:
            tables[k] = v
        else:
            scalars[k] = v

    lines = []
    for k, v in scalars.items():
        lines.append(f"{k} = {to_toml_val(v)}")
    if scalars and tables:
        lines.append("")
    for k, v in tables.items():
        table_name = f"{prefix}.{k}" if prefix else k
        lines.append(f"[{table_name}]")
        sub = dump_toml(v, table_name)
        if sub:
            lines.append(sub)
        lines.append("")
    return "\n".join(lines).strip()

def is_window_viewable(wid):
    res = subprocess.run(["xwininfo", "-id", wid], capture_output=True, text=True)
    for line in res.stdout.splitlines():
        if "Map State:" in line:
            return "IsViewable" in line
    return False

def get_commandspace_window():
    res = subprocess.run(["xwininfo", "-root", "-tree"], capture_output=True, text=True)
    for line in res.stdout.splitlines():
        if '"Commandspace": ("Commandspace" "Commandspace")' in line and "10x10" not in line:
            wid = line.strip().split()[0]
            if is_window_viewable(wid):
                return wid
    return None

def ensure_window_visible():
    wid = get_commandspace_window()
    if wid:
        return wid
    subprocess.run(["xdotool", "key", "ctrl+space"])
    time.sleep(0.4)
    return get_commandspace_window()

def get_window_geometry(wid):
    res = subprocess.run(["xwininfo", "-id", wid], capture_output=True, text=True)
    x = y = w = h = None
    for line in res.stdout.splitlines():
        line = line.strip()
        if line.startswith("Absolute upper-left X:"):
            x = int(line.split(":")[1].strip())
        elif line.startswith("Absolute upper-left Y:"):
            y = int(line.split(":")[1].strip())
        elif line.startswith("Width:"):
            w = int(line.split(":")[1].strip())
        elif line.startswith("Height:"):
            h = int(line.split(":")[1].strip())
    if None in (x, y, w, h):
        return None
    return x, y, w, h

def capture_window(wid, out_path, margin=20, window_only=False):
    out_path = Path(out_path)
    out_path.parent.mkdir(parents=True, exist_ok=True)

    if window_only or margin == 0:
        subprocess.run(["import", "-window", wid, str(out_path)], check=True)
        return

    geom = get_window_geometry(wid)
    if not geom:
        subprocess.run(["import", "-window", wid, str(out_path)], check=True)
        return

    x, y, w, h = geom
    crop_x = max(0, x - margin)
    crop_y = max(0, y - margin)
    crop_w = w + (x - crop_x) + margin
    crop_h = h + (y - crop_y) + margin

    crop_arg = f"{crop_w}x{crop_h}+{crop_x}+{crop_y}"
    subprocess.run(["import", "-window", "root", "-crop", crop_arg, str(out_path)], check=True)

def apply_theme(theme_name):
    preset_file = DEFAULT_THEMES_DIR / f"{theme_name}.toml"
    if not preset_file.is_file():
        # Try direct path
        preset_file = Path(theme_name)
        if not preset_file.is_file():
            raise FileNotFoundError(f"Theme '{theme_name}' not found in {DEFAULT_THEMES_DIR}")

    with open(preset_file, "rb") as f:
        preset = tomllib.load(f)

    with open(CONFIG_PATH, "rb") as f:
        cfg = tomllib.load(f)

    cfg["colors"] = preset.get("colors", {})
    cfg["colors"]["name"] = preset_file.stem
    if "window" in preset:
        if "window" not in cfg:
            cfg["window"] = {}
        for k, v in preset["window"].items():
            if isinstance(v, dict) and k in cfg["window"] and isinstance(cfg["window"][k], dict):
                cfg["window"][k].update(v)
            else:
                cfg["window"][k] = v

    out = dump_toml(cfg) + "\n"
    with open(CONFIG_PATH, "w") as f:
        f.write(out)

    time.sleep(0.4)

def main():
    parser = argparse.ArgumentParser(description="Capture Commandspace window screenshot.")
    parser.add_argument("theme", nargs="?", default=None, help="Theme name to apply before capture, or 'all'. Omit to capture current state.")
    parser.add_argument("--margin", type=int, default=20, help="Padding in pixels around the window (default: 20).")
    parser.add_argument("--window-only", action="store_true", help="Capture only the client window drawable without desktop margin.")
    parser.add_argument("-o", "--output-dir", type=Path, default=DEFAULT_OUTPUT_DIR, help="Output directory for screenshots.")
    args = parser.parse_args()

    wid = ensure_window_visible()
    if not wid:
        print("Error: Could not find or open Commandspace window.", file=sys.stderr)
        sys.exit(1)

    if args.theme == "all":
        theme_files = sorted(DEFAULT_THEMES_DIR.glob("*.toml"))
        if not theme_files:
            print(f"No themes found in {DEFAULT_THEMES_DIR}", file=sys.stderr)
            sys.exit(1)
        for tf in theme_files:
            theme_name = tf.stem
            print(f"Applying and capturing: {theme_name}...")
            apply_theme(theme_name)
            wid = ensure_window_visible()
            out_file = args.output_dir / f"snap_{theme_name}.png"
            capture_window(wid, out_file, margin=args.margin, window_only=args.window_only)
            print(f"  Saved -> {out_file}")
    elif args.theme:
        print(f"Applying theme: {args.theme}...")
        apply_theme(args.theme)
        wid = ensure_window_visible()
        out_file = args.output_dir / f"snap_{Path(args.theme).stem}.png"
        capture_window(wid, out_file, margin=args.margin, window_only=args.window_only)
        print(f"Saved -> {out_file}")
    else:
        out_file = args.output_dir / "snap_current.png"
        capture_window(wid, out_file, margin=args.margin, window_only=args.window_only)
        print(f"Saved -> {out_file}")

if __name__ == "__main__":
    main()
