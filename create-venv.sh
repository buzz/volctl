#!/bin/sh

virtualenv --system-site-packages venv
source venv/bin/activate
cd /tmp
git clone https://github.com/Valodim/python-pulseaudio.git
cd python-pulseaudio
git co 7af33cf60f87f74851dd47359859d8c47c3f7d2d
python setup.py install
cd ..
rm -rf python-pulseaudio
