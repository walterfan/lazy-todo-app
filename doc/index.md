# Lazy Todo App Knowledge Base

<!-- maintained-by: human+ai -->

This directory is the Project Knowledge Base for `lazy-todo-app`.

The documentation is authored in English Markdown, built with `Sphinx + MyST`, and localized to Simplified Chinese with `sphinx-intl`. Use the language switcher in the generated site to jump between `en` and `zh_CN` builds.

## Build

```bash
poetry install
poetry run make html
```

The bilingual HTML output is written to `_build/en/html/` and `_build/zh_CN/html/`.

```{toctree}
:maxdepth: 2
:caption: Core Docs

00-overview
01-repo-map
02-architecture
03-workflows
04-data-and-api
05-conventions
06-runbook
07-testing
08-build
09-document
ai-guide
adr/index
changes/index
```

---
<!-- PKB-metadata
last_updated: 2026-04-12
commit: a34edf3
updated_by: human+ai
-->
