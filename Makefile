sources = volctl setup.py

all: lint

lint: pylint flake8

pylint:
	pylint $(sources)

flake8:
	flake8 $(sources)

black:
	black $(sources)

.PHONY: all lint pylint flake8 black
