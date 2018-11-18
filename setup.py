#!/usr/bin/env python
"""setup.py for volctl"""

import re
from setuptools import setup


# parse version (setup.py should not import module!)
def get_version():
    """Get version using regex parsing."""
    versionfile = 'volctl/meta.py'
    with open(versionfile, 'rt') as file:
        version_file_content = file.read()
    match = re.search(
        r"^VERSION = ['\"]([^'\"]*)['\"]", version_file_content, re.M)
    if match:
        return match.group(1)
    raise RuntimeError(
        "Unable to find version string in {}.".format(versionfile))


setup(
    name='volctl',
    version=get_version(),
    description='Per-application volume control for GNU/Linux desktops',
    author='buzz',
    author_email='buzz@users.noreply.github.com',
    license='GPLv2',
    url='https://buzz.github.io/volctl/',
    packages=['volctl'],
    entry_points={
        'gui_scripts': [
            'volctl = volctl.__main__:main',
        ],
    },
    data_files=[
        ('share/applications', ['data/volctl.desktop']),
        ('share/glib-2.0/schemas', ['data/apps.volctl.gschema.xml']),
    ],
)
