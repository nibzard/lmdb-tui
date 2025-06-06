#!/usr/bin/env python3
"""Generate Homebrew formula and Scoop manifest."""

import json
import pathlib
import tomllib

ROOT = pathlib.Path(__file__).resolve().parents[1]

def get_version() -> str:
    data = tomllib.loads((ROOT / "Cargo.toml").read_text())
    return data["package"]["version"]

version = get_version()

dist = ROOT / "dist"
dist.mkdir(exist_ok=True)

formula = f"""class LmdbTui < Formula
  desc \"Terminal UI for LMDB databases\"
  homepage \"https://github.com/nibzard/lmdb-tui\"
  url \"https://github.com/nibzard/lmdb-tui/archive/v{version}.tar.gz\"
  sha256 \"PUT_SHA256_HERE\"
  license \"Apache-2.0\"

  def install
    bin.install \"lmdb-tui\"
  end

  test do
    system \"#{bin}/lmdb-tui\", \"--version\"
  end
end
"""

(dist / "lmdb-tui.rb").write_text(formula)

manifest = {
    "version": version,
    "description": "Terminal UI for LMDB databases",
    "homepage": "https://github.com/nibzard/lmdb-tui",
    "license": "Apache-2.0",
    "bin": "lmdb-tui.exe",
    "architecture": {
        "64bit": {
            "url": f"https://github.com/nibzard/lmdb-tui/releases/download/v{version}/lmdb-tui-{version}-x86_64-pc-windows-gnu.zip",
            "hash": "TODO"
        }
    }
}

(dist / "lmdb-tui.json").write_text(json.dumps(manifest, indent=2) + "\n")

print("Manifests generated in", dist)

