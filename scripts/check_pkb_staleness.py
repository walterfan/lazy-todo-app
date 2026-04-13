#!/usr/bin/env python3
from __future__ import annotations

import argparse
import fnmatch
import os
import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
ZERO_SHA = "0" * 40


@dataclass(frozen=True)
class Rule:
    patterns: tuple[str, ...]
    docs: tuple[str, ...]


RULES: tuple[Rule, ...] = (
    Rule(
        patterns=(
            "src-tauri/src/commands/**",
            "src-tauri/src/models/**",
            "src-tauri/src/db.rs",
            "src/types/**",
        ),
        docs=("doc/05-data-and-api.md", "doc/09-testing.md"),
    ),
    Rule(
        patterns=(
            "src-tauri/src/lib.rs",
            "src/main.tsx",
            "src/components/NoteWindow.tsx",
            "src/components/NoteCard.tsx",
            "src/components/PomodoroPanel.tsx",
        ),
        docs=("doc/02-architecture.md", "doc/06-workflows.md", "doc/10-runbook.md"),
    ),
    Rule(
        patterns=(
            ".github/workflows/**",
            "src-tauri/tauri.conf.json",
            "src-tauri/Cargo.toml",
            "package.json",
            "start.sh",
        ),
        docs=("doc/08-build.md",),
    ),
    Rule(
        patterns=(
            "doc/12-document.md",
            "doc/ai-guide.md",
            "doc/conf.py",
            "doc/Makefile",
            "doc/pyproject.toml",
            "doc/requirements.txt",
            "doc/_templates/**",
            "doc/_static/**",
        ),
        docs=("doc/12-document.md", "doc/ai-guide.md"),
    ),
)


def run_git(*args: str, check: bool = True) -> str:
    result = subprocess.run(
        ["git", *args],
        cwd=ROOT,
        capture_output=True,
        text=True,
    )
    if check and result.returncode != 0:
        raise RuntimeError(result.stderr.strip() or "git command failed")
    return result.stdout.strip()


def ref_exists(ref: str) -> bool:
    result = subprocess.run(
        ["git", "rev-parse", "--verify", ref],
        cwd=ROOT,
        capture_output=True,
        text=True,
    )
    return result.returncode == 0


def detect_default_base() -> str | None:
    for candidate in ("origin/main", "origin/master", "main", "master"):
        if ref_exists(candidate):
            return run_git("merge-base", "HEAD", candidate)
    if ref_exists("HEAD~1"):
        return "HEAD~1"
    return None


def list_files_at_head() -> list[str]:
    output = run_git("ls-tree", "-r", "--name-only", "HEAD")
    return [line for line in output.splitlines() if line]


def changed_files(base: str | None, head: str) -> list[str]:
    files: set[str] = set()

    if base == ZERO_SHA:
        files.update(list_files_at_head())

    elif base:
        output = run_git(
            "diff",
            "--name-only",
            "--diff-filter=ACMRTUXB",
            f"{base}...{head}",
        )
        if not output:
            output = run_git(
                "diff",
                "--name-only",
                "--diff-filter=ACMRTUXB",
                base,
                head,
            )
        files.update(line for line in output.splitlines() if line)

    files.update(working_tree_files())
    return sorted(files)


def working_tree_files() -> set[str]:
    files: set[str] = set()
    for args in (
        ("diff", "--name-only", "--diff-filter=ACMRTUXB"),
        ("diff", "--name-only", "--cached", "--diff-filter=ACMRTUXB"),
        ("ls-files", "--others", "--exclude-standard"),
    ):
        output = run_git(*args)
        files.update(line for line in output.splitlines() if line)
    return files


def is_doc_file(path: str) -> bool:
    return path.startswith("doc/") and path.endswith(".md")


def mapped_docs_for_path(path: str) -> set[str]:
    mapped: set[str] = set()

    for rule in RULES:
        if any(fnmatch.fnmatch(path, pattern) for pattern in rule.patterns):
            mapped.update(rule.docs)

    if path.startswith("src/") or path.startswith("src-tauri/src/"):
        mapped.update(("doc/00-overview.md", "doc/09-testing.md"))

    return mapped


def metadata_for(doc_path: str) -> dict[str, str]:
    path = ROOT / doc_path
    if not path.exists():
        return {}

    content = path.read_text(encoding="utf-8")
    matches = list(
        re.finditer(r"<!-- PKB-metadata\s*\n(.*?)\n-->", content, re.DOTALL)
    )
    if not matches:
        return {}

    metadata: dict[str, str] = {}
    for line in matches[-1].group(1).splitlines():
        if ":" not in line:
            continue
        key, value = line.split(":", 1)
        key = key.strip()
        value = value.strip()
        if key in {"last_updated", "commit", "updated_by"}:
            metadata[key] = value
    return metadata


def build_summary(base: str | None, head: str, files: list[str]) -> str:
    changed_doc_pages = {path for path in files if is_doc_file(path)}
    triggers: dict[str, set[str]] = {}

    for path in files:
        for doc in mapped_docs_for_path(path):
            triggers.setdefault(doc, set()).add(path)

    lines: list[str] = []
    lines.append("# PKB freshness summary")
    lines.append("")
    lines.append(f"- Compared range: `{base or 'N/A'} -> {head}`")
    lines.append(f"- Changed files: `{len(files)}`")
    lines.append("")

    if files:
        lines.append("## Changed paths")
        lines.append("")
        for path in files[:25]:
            lines.append(f"- `{path}`")
        if len(files) > 25:
            lines.append(f"- ... and `{len(files) - 25}` more")
        lines.append("")

    if not triggers:
        lines.append("## Result")
        lines.append("")
        lines.append(
            "No PKB advisory suggestions were triggered by the current diff. "
            "Either only docs changed, or the changed files do not currently map "
            "to numbered PKB pages."
        )
        return "\n".join(lines)

    lines.append("## Suggested PKB pages")
    lines.append("")
    lines.append("| PKB page | Status | Triggered by | Last metadata |")
    lines.append("|---|---|---|---|")

    for doc in sorted(triggers):
        status = "updated in diff" if doc in changed_doc_pages else "review suggested"
        trigger_list = ", ".join(f"`{item}`" for item in sorted(triggers[doc])[:3])
        if len(triggers[doc]) > 3:
            trigger_list += f", +{len(triggers[doc]) - 3} more"
        metadata = metadata_for(doc)
        metadata_text = "n/a"
        if metadata:
            metadata_text = (
                f"{metadata.get('last_updated', 'unknown')} / "
                f"{metadata.get('commit', 'unknown')}"
            )
        lines.append(f"| `{doc}` | {status} | {trigger_list} | {metadata_text} |")

    lines.append("")
    stale_docs = sorted(doc for doc in triggers if doc not in changed_doc_pages)
    if stale_docs:
        lines.append("## Advisory next steps")
        lines.append("")
        lines.append("The following PKB pages may need a refresh:")
        lines.append("")
        for doc in stale_docs:
            lines.append(f"- `{doc}`")
        lines.append("")
        lines.append("Recommended follow-up:")
        lines.append("")
        lines.append("1. Run `npm run pkb:check` locally to inspect the same summary.")
        lines.append("2. Use your LLM + skills workflow to refresh the English PKB pages.")
        lines.append("3. If English docs changed, run:")
        lines.append("")
        lines.append("```bash")
        lines.append("cd doc")
        lines.append("poetry run make gettext")
        lines.append("poetry run make intl-update")
        lines.append("poetry run make html")
        lines.append("```")
    else:
        lines.append("## Result")
        lines.append("")
        lines.append("All mapped PKB pages were updated in the current diff.")

    return "\n".join(lines)


def write_github_summary(summary: str) -> None:
    summary_path = os.environ.get("GITHUB_STEP_SUMMARY")
    if not summary_path:
        return
    Path(summary_path).write_text(summary + "\n", encoding="utf-8")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Advisory PKB staleness checker")
    parser.add_argument("--base", help="Base git ref or commit")
    parser.add_argument("--head", default="HEAD", help="Head git ref or commit")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    base = args.base or detect_default_base()
    head = args.head

    if not ref_exists(head):
        raise RuntimeError(f"Head ref does not exist: {head}")

    files = changed_files(base, head)
    summary = build_summary(base, head, files)
    print(summary)
    write_github_summary(summary)
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as exc:  # pragma: no cover - best-effort CLI
        print(f"PKB checker failed: {exc}", file=sys.stderr)
        raise SystemExit(1)
