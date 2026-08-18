"""Full CI orchestration for the mingling project.

Runs every `cargo ci` step in order: lock the workspace, run all checks,
refresh the generated artifacts, then unlock. The final `git-unlock` doubles
as the idempotency check: it fails with a non-zero exit code when the run
left the working tree dirty.

The script locates the git repository root and runs with it as the working
directory, so it can be invoked from anywhere inside the repo.
"""

import os
import subprocess
import sys
from pathlib import Path

# The pipeline steps, in execution order.
STEPS = [
    "git-lock",
    "report-clean",
    "build-check",
    "clippy-check",
    "test-all",
    "example-check",
    "docs-check",
    "example-refresh",
    "docsify-refresh",
    "features-refresh",
    # Idempotency check: exits non-zero if CI contaminated the workspace.
    "git-unlock",
]


def find_repo_root() -> Path:
    """Return the nearest ancestor directory containing `.git`."""
    current = Path.cwd()
    for directory in (current, *current.parents):
        if (directory / ".git").is_dir():
            return directory
    raise SystemExit("error: not inside a git repository")


def main() -> int:
    root = find_repo_root()
    os.chdir(root)

    # Signature banner: docs/res/ci_banner.txt, relative to this script
    # (.run/src/bin -> four levels up is the repo root).
    banner = (
        Path(__file__).resolve().parent.parent.parent.parent
        / "docs"
        / "res"
        / "ci_banner.txt"
    )
    try:
        print(banner.read_text(encoding="utf-8"), end="")
    except OSError:
        pass

    for step in STEPS:
        print(f"==> cargo ci {step}")
        result = subprocess.run(["cargo", "ci", step], check=False)
        if result.returncode != 0:
            print(
                f"error: step `{step}` failed with exit code {result.returncode}",
                file=sys.stderr,
            )
            return result.returncode

    return 0


if __name__ == "__main__":
    sys.exit(main())
