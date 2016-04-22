# volctl

Per-application volume control for GNU/Linux desktops.

I couldn't find a simple tray icon that allows to control multiple
applications easily from the task bar:

![Screenshot](https://buzz.github.io/volctl/screenshot.png)

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

## Dependencies

* python2-gobject

