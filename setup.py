#!/usr/bin/env python2.7

from distutils.core import setup

setup(name='volctl',
      version='0.3',
      description='Per-application volume control for GNU/Linux desktops',
      author='buzz',
      author_email='buzz-AT-l4m1-DOT-de',
      license='GPLv2',
      url='https://buzz.github.io/volctl/',
      packages=['volctl'],
      scripts=['bin/volctl'],
      data_files=[
          ('share/applications', ['volctl.desktop']),
      ],
)
