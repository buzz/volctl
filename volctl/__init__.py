import gi
gi.require_version('Gtk', '3.0')
gi.require_version('AppIndicator3', '0.1')
from gi.repository import Gtk, GObject, AppIndicator3

from tray import VolCtlTray
import signal
 
APPINDICATOR_ID = 'VolCtl'


PROGRAM_NAME = 'Volume Control'
VERSION =      '0.3'
COPYRIGHT =    '(c) buzz'
LICENSE =      Gtk.License.GPL_2_0
COMMENTS =     'Per-application volume control for GNU/Linux desktops'
WEBSITE =      'https://buzz.github.io/volctl/'

def main():
    GObject.threads_init()
    vctray = VolCtlTray()

    indicator = AppIndicator3.Indicator.new(APPINDICATOR_ID, 'preferences-desktop', AppIndicator3.IndicatorCategory.SYSTEM_SERVICES)
    indicator.set_status(AppIndicator3.IndicatorStatus.ACTIVE)
    indicator.set_menu(vctray.menu)

    signal.signal(signal.SIGINT, signal.SIG_DFL)

    Gtk.main()
