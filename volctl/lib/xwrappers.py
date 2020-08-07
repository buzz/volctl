"""
Python wrapper for some X-related stuff.

Copyright (C) 2017 Kozec

Adapted from:
https://github.com/kozec/sc-controller/blob/master/scc/lib/xwrappers.py
"""

from ctypes import c_int, c_short, c_ulong, c_ushort, c_void_p, CDLL, POINTER, Structure


def _load_lib(*names):
    """Try multiple alternative names to load .so library."""
    for name in names:
        try:
            return CDLL(name)
        except OSError:
            pass
    raise OSError("Failed to load %s, library not found" % (names[0],))


libXFixes = _load_lib("libXfixes.so", "libXfixes.so.3")

# Types
XID = c_ulong
Display = c_void_p
XserverRegion = c_ulong


# Structures
class XRectangle(Structure):
    """X11 XRectangle structure"""

    # pylint: disable=too-few-public-methods

    _fields_ = [
        ("x", c_short),
        ("y", c_short),
        ("width", c_ushort),
        ("height", c_ushort),
    ]


# Constants
SHAPE_BOUNDING = 0
SHAPE_INPUT = 2

create_region = libXFixes.XFixesCreateRegion
create_region.__doc__ = (
    "Creates rectanglular region for use with set_window_shape_region"
)
create_region.argtypes = [c_void_p, POINTER(XRectangle), c_int]
create_region.restype = XserverRegion

set_window_shape_region = libXFixes.XFixesSetWindowShapeRegion
set_window_shape_region.__doc__ = "Sets region in which window accepts inputs"
set_window_shape_region.argtypes = [c_void_p, XID, c_int, c_int, c_int, XserverRegion]

destroy_region = libXFixes.XFixesDestroyRegion
destroy_region.__doc__ = "Frees region created by create_region"
destroy_region.argtypes = [c_void_p, XserverRegion]
