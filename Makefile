all: lint

lint: pylint flake8

pylint:
	pylint volctl setup.py

flake8:
	flake8 volctl setup.py
