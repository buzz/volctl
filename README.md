# volctl

![Screenshot](https://raw.githubusercontent.com/buzz/volctl/master/volctl_screenshot.png)

[PulseAudio](http://www.freedesktop.org/wiki/Software/PulseAudio/)-enabled
tray icon volume control for GNU/Linux desktops written in
[GTK+](http://www.gtk.org/).

I couldn't find a simple tray icon that allows to control multiple
applications easily from the task bar.

So I wrote my own. The program is written in Python and fairly short
and should not be too hard to understand. Bug reports and patches welcome!

It's not meant to be an replacement for a full-featured mixer
application. If you're looking for that check out the excellent
[pavucontrol](http://freedesktop.org/software/pulseaudio/pavucontrol/).

## Features

* Runs on virtually every desktop environment under GNU/Linux. (Needs to support the freedesktop system tray specs)
* Control main volumes as well as individual applications
* Shows applications icons and names
* Internally uses the PulseAudio library directly which turned out to work much better then DBUS
* Double-click opens `pavucontrol`
* Mouse-wheel support

## Usage

1. Clone this repository somewhere to your home folder.
1. Run `create-venv.sh` to initialize virtual env that holds deps.
1. Make `volctl.sh` auto-start with your desktop environment. This step depends on which desktop you use.

## Dependencies

* python2-gobject
* [python-pulseaudio](https://github.com/Valodim/python-pulseaudio)
