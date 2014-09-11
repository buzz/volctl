#!/bin/sh

virtualenv --system-site-packages venv
source venv/bin/activate
cd /tmp
wget 'http://datatomb.de/~valodim/libpulseaudio-1.1.tar.gz' -qO - | tar xzf -
cd libpulseaudio-1.1
python setup.py install
cd ..
rm -r libpulseaudio-1.1
