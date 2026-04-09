import shutil
import subprocess
import sys
from pathlib import Path
from typing import Any

from hatchling.builders.hooks.plugin.interface import BuildHookInterface
from packaging import tags


class CustomHook(BuildHookInterface[Any]):
    source_dir = Path("target/release/")
    target_dir = Path("vapoursynth/plugins")

    def initialize(self, version: str, build_data: dict[str, Any]) -> None:
        build_data["pure_python"] = False
        build_data["tag"] = f"py3-none-{next(tags.platform_tags())}"
        subprocess.run(["cargo", "build", "--release"], check=True)
        self.target_dir.mkdir(parents=True, exist_ok=True)
        for file_path in self.source_dir.glob("*"):
            if file_path.is_file() and file_path.suffix in [".dll", ".so", ".dylib"]:
                shutil.copy2(file_path, self.target_dir)

    def finalize(self, version: str, build_data: dict[str, Any], artifact_path: str) -> None:
        shutil.rmtree(self.target_dir.parent, ignore_errors=True)