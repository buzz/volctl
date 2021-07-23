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
* Mute individual applications
* Shows application icons and names
* Per-application VU meter
* Double-click opens *pavucontrol* (or custom mixer application)
* Mouse-wheel support
* On-screen volume display (OSD)

## Installation

### Manual installation

1. Clone this repository somewhere and cd into it.
1. To install: `sudo ./setup.py install`
   Note: You might need to copy `data/apps.volctl.gschema.xml` to `/usr/share/glib-2.0/schemas/` manually.
1. For the application icon to show up in the menu: `sudo update-desktop-database`
1. Compile GSettings schemas: `sudo glib-compile-schemas /usr/share/glib-2.0/schemas/` or sudo `glib-compile-schemas /usr/local/share/glib-2.0/schemas/`

### Arch Linux

Available in AUR: [volctl](https://aur.archlinux.org/packages/volctl/)

## Status/tray icon implementation

volctl strives to achieve a high level of support across different Desktop
Environments. Unfortunately, on the Linux Desktop several tray icon
implemenations with various levels of support and capabilities co-exist.

volctl supports

- [*SNI*](https://freedesktop.org/wiki/Specifications/StatusNotifierItem/)  
  Supported on modern Desktop Environments, like Gnome, KDE, works also on Wayland
- [*XEmbed*](https://www.freedesktop.org/wiki/Specifications/systemtray-spec/)
(through `Gtk.StatusIcon`)  
  Not supported on Gnome, KDE (only through extensions/plugins). No Wayland
  support.

Your Desktop Environment might support both, one or none of these standards.
Personally I use XEmbed as it allows for all important user interactions (mouse
wheel, double-click, etc.) on my current system. The default is to prefer SNI
which can be changed under the Preferences ➝ Prefer XEmbed.

*Please try for yourself which type of tray icon works best for you.*

**Note:** If you need support for SNI you have to compile and install
[statusnotifier](https://jjacky.com/statusnotifier/). Use the configure flags
`--enable-introspection` and `--enable-dbusmenu`. If you're on Arch Linux you
can use the AUR package
[statusnotifier-introspection-dbus-menu](https://aur.archlinux.org/packages/statusnotifier-introspection-dbus-menu/).

## No Wayland support ([#39](https://github.com/buzz/volctl/issues/39))

Through SNI volctl now supports tray icons under Wayland. Unfortunately it's not
possible to display the volume slider window on Wayland at the mouse pointer
position. The Wayland protocol does not allow this unless non-standard Wayland
extensions are used. The only entity that is capable of doing so is the Wayland
compositor (generally your Desktop Environment).

## Development

### Deploy dev version in Virtualenv

You can start volctl from the source tree.

```sh
$ python -m venv --system-site-packages venv
$ ./setup.py develop
$ venv/bin/volctl
```

### Linting

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
