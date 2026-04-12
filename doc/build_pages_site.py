from __future__ import annotations

import shutil
from pathlib import Path


ROOT = Path(__file__).resolve().parent
BUILD_ROOT = ROOT / "_build"
EN_SOURCE = BUILD_ROOT / "en" / "html"
ZH_SOURCE = BUILD_ROOT / "zh_CN" / "html"
SITE_ROOT = BUILD_ROOT / "site"


def copy_tree(source: Path, target: Path) -> None:
    if not source.exists():
        raise FileNotFoundError(f"Expected build output missing: {source}")
    shutil.copytree(source, target, dirs_exist_ok=True)


def write_index(target: Path) -> None:
    target.write_text(
        """<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>Lazy Todo App Docs</title>
    <style>
      body {
        font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
        margin: 0;
        min-height: 100vh;
        display: grid;
        place-items: center;
        background: #0f172a;
        color: #e2e8f0;
      }
      main {
        max-width: 720px;
        padding: 2rem;
        text-align: center;
      }
      h1 {
        margin-bottom: 0.75rem;
      }
      p {
        line-height: 1.6;
        color: #cbd5e1;
      }
      .links {
        display: flex;
        justify-content: center;
        gap: 1rem;
        margin-top: 1.5rem;
        flex-wrap: wrap;
      }
      a {
        display: inline-block;
        padding: 0.8rem 1.2rem;
        border-radius: 999px;
        text-decoration: none;
        font-weight: 600;
        color: #0f172a;
        background: #f8fafc;
      }
      a.secondary {
        background: #38bdf8;
      }
    </style>
  </head>
  <body>
    <main>
      <h1>Lazy Todo App Documentation</h1>
      <p>
        This site publishes the bilingual Project Knowledge Base built with
        Sphinx, MyST, and sphinx-intl.
      </p>
      <p>
        该站点发布了使用 Sphinx、MyST 和 sphinx-intl 构建的双语项目知识库。
      </p>
      <div class="links">
        <a href="./en/">English</a>
        <a class="secondary" href="./zh_CN/">简体中文</a>
      </div>
    </main>
  </body>
</html>
""",
        encoding="utf-8",
    )


def main() -> None:
    if SITE_ROOT.exists():
        shutil.rmtree(SITE_ROOT)

    (SITE_ROOT / "en").mkdir(parents=True, exist_ok=True)
    (SITE_ROOT / "zh_CN").mkdir(parents=True, exist_ok=True)

    copy_tree(EN_SOURCE, SITE_ROOT / "en")
    copy_tree(ZH_SOURCE, SITE_ROOT / "zh_CN")
    write_index(SITE_ROOT / "index.html")
    (SITE_ROOT / ".nojekyll").write_text("", encoding="utf-8")


if __name__ == "__main__":
    main()
