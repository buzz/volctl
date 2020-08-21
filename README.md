# volctl

Per-application volume control and OSD for Linux desktops.

![Screenshot](https://buzz.github.io/volctl/screenshot.png)

I couldn't find a simple tray icon that allows to control multiple
applications easily from the task bar. So I wrote my own.

**Bug reports and patches welcome!**

It's not meant to be an replacement for a full-featured mixer
application. If you're looking for that check out the excellent
[pavucontrol](http://freedesktop.org/software/pulseaudio/pavucontrol/).

## Features

* Runs on virtually every desktop environment (Needs to support the freedesktop system tray specs)
* Control main volumes as well as individual applications
* Shows applications icons and names
* Per-application VU meter
* Double-click opens *pavucontrol*
* Mouse-wheel support
* On-screen volume display (OSD)

## Installation

Check the [homepage](https://buzz.github.io/volctl/) for details.

## Development

###### Deploy dev version in Virtualenv

You can start volctl from the source tree.

```sh
$ python -m venv --system-site-packages venv
$ ./setup.py develop
$ venv/bin/volctl
```

###### Linting

Use pylint and flake8 for linting the sources.

```sh
$ make lint
```

Use black to auto-format the code.

```sh
$ make black
```

## License

GNU General Public License v2.0
