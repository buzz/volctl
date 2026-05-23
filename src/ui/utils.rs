use gdk::prelude::DisplayExtManual;
use gtk_layer_shell::{Edge, LayerShell};

use crate::errors::X11Error;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum DisplayType {
    Wayland,
    X11,
}

pub fn get_display_type() -> Result<DisplayType, X11Error> {
    let display = gdk::Display::default().ok_or(X11Error::NoDisplay)?;
    Ok(if display.backend().is_wayland() {
        DisplayType::Wayland
    } else {
        DisplayType::X11
    })
}

/// Screen position for layer-shell anchored surfaces (OSD, mixer window).
///
/// Values match the `apps.volctl.position` GSettings enum.
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
#[repr(i32)]
pub enum Position {
    TopLeft = 0,
    TopCenter = 1,
    #[default]
    TopRight = 2,
    CenterLeft = 3,
    CenterCenter = 4,
    CenterRight = 5,
    BottomLeft = 6,
    BottomCenter = 7,
    BottomRight = 8,
}

impl TryFrom<i32> for Position {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::TopLeft),
            1 => Ok(Self::TopCenter),
            2 => Ok(Self::TopRight),
            3 => Ok(Self::CenterLeft),
            4 => Ok(Self::CenterCenter),
            5 => Ok(Self::CenterRight),
            6 => Ok(Self::BottomLeft),
            7 => Ok(Self::BottomCenter),
            8 => Ok(Self::BottomRight),
            _ => Err(()),
        }
    }
}

impl Position {
    /// Returns the vertical component of the position.
    pub fn vertical(&self) -> VerticalPos {
        match self {
            Self::TopLeft | Self::TopCenter | Self::TopRight => VerticalPos::Top,
            Self::CenterLeft | Self::CenterCenter | Self::CenterRight => VerticalPos::Center,
            Self::BottomLeft | Self::BottomCenter | Self::BottomRight => VerticalPos::Bottom,
        }
    }

    /// Returns the horizontal component of the position.
    pub fn horizontal(&self) -> HorizontalPos {
        match self {
            Self::TopLeft | Self::CenterLeft | Self::BottomLeft => HorizontalPos::Left,
            Self::TopCenter | Self::CenterCenter | Self::BottomCenter => HorizontalPos::Center,
            Self::TopRight | Self::CenterRight | Self::BottomRight => HorizontalPos::Right,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum VerticalPos {
    Top,
    Center,
    Bottom,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum HorizontalPos {
    Left,
    Center,
    Right,
}

/// Apply layer shell anchors and margins based on a [`Position`].
///
/// When a dimension is `Center`, both edges are anchored so the compositor
/// centers the surface (requires auto_exclusive_zone to be enabled).
pub fn apply_layer_shell_position<W: LayerShell>(window: &W, position: Position, margin: i32) {
    // Reset all anchors and margins first
    window.set_anchor(Edge::Top, false);
    window.set_anchor(Edge::Bottom, false);
    window.set_anchor(Edge::Left, false);
    window.set_anchor(Edge::Right, false);
    window.set_margin(Edge::Top, 0);
    window.set_margin(Edge::Bottom, 0);
    window.set_margin(Edge::Left, 0);
    window.set_margin(Edge::Right, 0);

    // Vertical anchor & margin
    match position.vertical() {
        VerticalPos::Top => {
            window.set_anchor(Edge::Top, true);
            window.set_margin(Edge::Top, margin);
        }
        VerticalPos::Bottom => {
            window.set_anchor(Edge::Bottom, true);
            window.set_margin(Edge::Bottom, margin);
        }
        VerticalPos::Center => {
            window.set_anchor(Edge::Top, true);
            window.set_anchor(Edge::Bottom, true);
        }
    }

    // Horizontal anchor & margin
    match position.horizontal() {
        HorizontalPos::Left => {
            window.set_anchor(Edge::Left, true);
            window.set_margin(Edge::Left, margin);
        }
        HorizontalPos::Right => {
            window.set_anchor(Edge::Right, true);
            window.set_margin(Edge::Right, margin);
        }
        HorizontalPos::Center => {
            window.set_anchor(Edge::Left, true);
            window.set_anchor(Edge::Right, true);
        }
    }
}
