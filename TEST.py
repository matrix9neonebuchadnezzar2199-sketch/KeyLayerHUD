#!/usr/bin/env python3
"""KeyLayerHUD verification — protocol unit tests via cargo + file checks."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent


def run(cmd: list[str], cwd: Path) -> None:
    print(f"+ {' '.join(cmd)}")
    subprocess.run(cmd, cwd=cwd, check=True)


def check_files() -> None:
    required = [
        "layouts/corne-42.json",
        "keymaps/sample-via.json",
        "firmware/qmk/layer_report.c",
        "docs/mockup_hud.html",
        "docs/mockup_settings.html",
        "hud.html",
        "settings.html",
    ]
    missing = [p for p in required if not (ROOT / p).exists()]
    if missing:
        raise SystemExit(f"Missing files: {missing}")


def main() -> int:
    check_files()
    run(["cargo", "test"], ROOT / "src-tauri")
    print("TEST.py: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
