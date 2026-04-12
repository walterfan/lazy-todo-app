from __future__ import annotations

import json
import os
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACKAGE_JSON = ROOT / "package.json"
PACKAGE = json.loads(PACKAGE_JSON.read_text(encoding="utf-8"))

project = "Lazy Todo App"
author = "Walter Fan"
copyright = "2026, Walter Fan"
release = PACKAGE.get("version", "0.1.0")
version = release

extensions = [
    "myst_parser",
    "sphinxcontrib.mermaid",
]

source_suffix = {
    ".md": "markdown",
}

templates_path = ["_templates"]
exclude_patterns = [
    "_build",
    "Thumbs.db",
    ".DS_Store",
    "locale/**",
]

language = os.environ.get("SPHINX_LANGUAGE", "en")
locale_dirs = ["locale/"]
gettext_compact = False
gettext_uuid = True

html_theme = "sphinx_rtd_theme"
html_title = f"{project} Docs"
html_static_path = ["_static"]
html_css_files = ["custom.css"]
html_context = {
    "available_languages": [
        {"code": "en", "label": "English"},
        {"code": "zh_CN", "label": "中文"},
    ]
}

myst_heading_anchors = 3
myst_enable_extensions = [
    "colon_fence",
    "deflist",
]
myst_fence_as_directive = ["mermaid"]
