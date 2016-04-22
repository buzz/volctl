import gi
gi.require_version('Gtk', '3.0')
from gi.repository import Gtk, GObject

from tray import VolCtlTray


PROGRAM_NAME = 'Volume Control'
VERSION =      '0.3'
COPYRIGHT =    '(c) buzz'
LICENSE =      Gtk.License.GPL_2_0
COMMENTS =     'Per-application volume control for GNU/Linux desktops'
WEBSITE =      'https://buzz.github.io/volctl/'

def main():
    GObject.threads_init()
    vctray = VolCtlTray()
    Gtk.main()
