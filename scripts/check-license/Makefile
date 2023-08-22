VENV?=.venv
BIN?=$(VENV)/bin
PYTHON?=$(BIN)/python
PIP?=$(BIN)/pip
PYTEST?=$(BIN)/pytest

.PHONY: default
default: install
	$(PYTEST) tests/

.PHONY: venv
venv:
	pip install virtualenv
	[ -d $(VENV) ] || virtualenv $(VENV)
	$(PIP) install -r requirements.txt
	make install

.PHONY: clean
clean:
	python setup.py clean
	rm -fr .venv
	rm -fr build
	rm -fr dist

.PHONY: install
install:
	$(PYTHON) setup.py install
