all: lint

lint: pylint flake8

pylint:
	pylint volctl bin setup.py

flake8:
	flake8
