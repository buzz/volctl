#!/usr/bin/env python2.7
from distutils.core import setup

# parse version (setup.py should not import module!)
import re
VERSIONFILE = 'volctl/_version.py'
with open(VERSIONFILE, 'rt') as f:
    version_file_content = f.read()
version_regex = r"^__version__ = ['\"]([^'\"]*)['\"]"
m = re.search(version_regex, version_file_content, re.M)
if m:
    version = m.group(1)
else:
    raise RuntimeError('Unable to find version string in %s.' % VERSIONFILE)

setup(name='volctl',
      version=version,
      description='Per-application volume control for GNU/Linux desktops',
      author='buzz',
      author_email='buzz-AT-l4m1-DOT-de',
      license='GPLv2',
      url='https://buzz.github.io/volctl/',
      packages=['volctl'],
      scripts=['bin/volctl'],
      data_files=[
          ('share/applications', ['data/volctl.desktop']),
          ('share/glib-2.0/schemas', ['data/apps.volctl.gschema.xml']),
      ],
)
