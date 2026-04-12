SPHINXBUILD ?= sphinx-build
SPHINXINTL ?= sphinx-intl
SOURCEDIR = .
GETTEXTDIR = _build/gettext
ENHTMLDIR = _build/en/html
ZHHTMLDIR = _build/zh_CN/html
PAGESDIR = _build/site

.PHONY: install clean gettext intl-update html html-en html-zh pages serve

install:
	poetry install

clean:
	rm -rf _build

gettext:
	$(SPHINXBUILD) -b gettext $(SOURCEDIR) $(GETTEXTDIR)

intl-update: gettext
	$(SPHINXINTL) update -p $(GETTEXTDIR) -l zh_CN

html-en:
	SPHINX_LANGUAGE=en $(SPHINXBUILD) -b html $(SOURCEDIR) $(ENHTMLDIR)

html-zh:
	SPHINX_LANGUAGE=zh_CN $(SPHINXBUILD) -b html $(SOURCEDIR) $(ZHHTMLDIR)

html: intl-update html-en html-zh

pages: html
	python3 build_pages_site.py

serve: html
	python3 -m http.server 8000 --directory _build
