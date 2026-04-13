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
:caption: Getting Started

00-overview
01-quick-start
```

```{toctree}
:maxdepth: 2
:caption: Design & Structure

02-architecture
03-tech-stack
04-repo-map
05-data-and-api
06-workflows
```

```{toctree}
:maxdepth: 2
:caption: Development

07-conventions
08-build
09-testing
```

```{toctree}
:maxdepth: 2
:caption: Operations

10-runbook
11-observability
12-document
```

```{toctree}
:maxdepth: 2
:caption: Appendix

appendix-01-faq
appendix-02-glossary
ai-guide
adr/index
changes/index
```

---
<!-- PKB-metadata
last_updated: 2026-04-12
commit: 628f0c1
updated_by: human+ai
-->
