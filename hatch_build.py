import shutil
import subprocess
import sys
from pathlib import Path
from typing import Any
import os
from hatchling.builders.hooks.plugin.interface import BuildHookInterface
from packaging import tags
import platform

class CustomHook(BuildHookInterface[Any]):
    source_dir = Path("target/release/")
    target_dir = Path("vapoursynth/plugins")

    def initialize(self, version: str, build_data: dict[str, Any]) -> None:
        build_data["pure_python"] = False
        ptag = next(tags.platform_tags())
        envs = {}
        if "musllinux" in ptag:
            envs = { "RUSTFLAGS": "-C target-feature=-crt-static"}
        if platform.system() == "Darwin":
            envs = { "RUSTFLAGS": "-C link-arg=-headerpad_max_install_names"}

        build_data["tag"] = f"py3-none-{ptag}"
        subprocess.run(["cargo", "build", "--release"], env=envs | os.environ, check=True)
        self.target_dir.mkdir(parents=True, exist_ok=True)
        for file_path in self.source_dir.glob("*"):
            if file_path.is_file() and file_path.suffix in [".dll", ".so", ".dylib"]:
                shutil.copy2(file_path, self.target_dir)

    def finalize(self, version: str, build_data: dict[str, Any], artifact_path: str) -> None:
        shutil.rmtree(self.target_dir.parent, ignore_errors=True)