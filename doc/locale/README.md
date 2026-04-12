# Translation Workspace

English Markdown in `doc/` is the source of truth.

Generate or refresh translation catalogs with:

```bash
poetry run make gettext
poetry run make intl-update
```

Then edit `locale/zh_CN/LC_MESSAGES/*.po` and rebuild:

```bash
poetry run make html
```
