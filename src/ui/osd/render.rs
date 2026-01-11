use gdk::RGBA;
use gtk::graphene::Point;
use gtk::gsk::{FillRule, PathBuilder, RoundedRect, Stroke};
use gtk::pango;
use gtk::prelude::*;
use gtk::Snapshot;

use crate::constants::{
    OSD_BASE_FONT_SIZE, OSD_BASE_HEIGHT, OSD_BASE_LINE_WIDTH, OSD_BASE_PADDING, OSD_BASE_WIDTH,
    OSD_BG_CORNER_RADIUS, OSD_BG_OPACITY, OSD_MUTE_OPACITY, OSD_NUM_BARS, OSD_TEXT_OPACITY,
};

/// Rendering state passed from controller to rendering module
#[derive(Clone, Default)]
pub struct RenderState {
    pub volume: f64,
    pub muted: bool,
    pub opacity: f64,
    pub scale: f64,
    /// True when a compositor handles transparency (enables rounded corners).
    /// False on plain X11 WMs where transparent regions render as black.
    pub composited: bool,
}

/// Main rendering function
pub fn build_snapshot(
    state: &RenderState,
    snapshot: &Snapshot,
    pango_context: &gtk::pango::Context,
) {
    let scale = state.scale;
    let width = (OSD_BASE_WIDTH * scale) as f32;
    let height = (OSD_BASE_HEIGHT * scale) as f32;
    let padding = (OSD_BASE_PADDING * scale) as f32;
    let font_size = (OSD_BASE_FONT_SIZE * scale) as f32;
    let line_width = (OSD_BASE_LINE_WIDTH * scale) as f32;
    let corner_radius = (OSD_BG_CORNER_RADIUS * scale) as f32;

    let xcenter = width / 2.0;
    let mute_opacity = if state.muted { OSD_MUTE_OPACITY } else { 1.0 };

    // BACKGROUND (rounded rectangle when composited, sharp when not)
    let bg_builder = PathBuilder::new();
    if state.composited {
        let rect = RoundedRect::from_rect(
            gtk::graphene::Rect::new(0.0, 0.0, width, height),
            corner_radius,
        );
        bg_builder.add_rounded_rect(&rect);
    } else {
        // No compositor: use a plain rectangle so there's no transparent corner area
        let rect = gtk::graphene::Rect::new(0.0, 0.0, width, height);
        bg_builder.add_rect(&rect);
    }
    let bg_path = bg_builder.to_path();

    let bg_color = RGBA::new(0.1, 0.1, 0.1, (OSD_BG_OPACITY * state.opacity) as f32);
    snapshot.append_fill(&bg_path, FillRule::Winding, &bg_color);

    // TEXT (percentage)
    let text = format!("{} %", (state.volume * 100.0).round() as i32);

    // Use the provided PangoContext (has proper font map)
    let layout = gtk::pango::Layout::new(pango_context);
    layout.set_text(&text);

    // Set font
    let font_desc = gtk::pango::FontDescription::from_string(&format!("sans-serif {}", font_size));
    layout.set_font_description(Some(&font_desc));

    // Get text dimensions - use ink extents for accurate bounding box (like Cairo)
    let (ink_rect, _) = layout.extents();
    let text_width = ink_rect.width() as f32 / pango::SCALE as f32;
    let text_height = ink_rect.height() as f32 / pango::SCALE as f32;

    // Get baseline position within the layout
    let baseline = layout.baseline() as f32 / pango::SCALE as f32;

    // Position text (centered horizontally, BASELINE at height - padding)
    // Matching Python: move_to sets baseline position
    // Pango renders from top-left, so offset by ascent (baseline position in layout)
    let text_x = xcenter - text_width / 2.0;
    let text_y = height - padding - baseline;

    // Text color (matching Python: white with TEXT_OPACITY * mute_opacity * opacity)
    let text_color = RGBA::new(
        1.0,
        1.0,
        1.0,
        (OSD_TEXT_OPACITY * mute_opacity * state.opacity) as f32,
    );

    // Draw text using GTK4 snapshot API
    snapshot.save();
    snapshot.translate(&Point::new(text_x, text_y));
    snapshot.append_layout(&layout, &text_color);
    snapshot.restore();

    // VOLUME INDICATOR (radial bars)
    let ind_height = height - 3.0 * padding - text_height;
    let outer_radius = ind_height / 2.0;
    let inner_radius = outer_radius / 1.618;
    let bars = ((OSD_NUM_BARS as f64 * state.volume).round() as i32).min(OSD_NUM_BARS);
    let center_y = padding + ind_height / 2.0;

    let bar_color = RGBA::new(
        1.0,
        1.0,
        1.0,
        (OSD_TEXT_OPACITY * mute_opacity * state.opacity) as f32,
    );
    let stroke = Stroke::new(line_width);
    stroke.set_line_cap(gtk::gsk::LineCap::Round);

    for i in 0..bars {
        // Start from SOUTH (90° = π/2) and distribute clockwise
        let angle = std::f64::consts::FRAC_PI_2
            + 2.0 * std::f64::consts::PI / OSD_NUM_BARS as f64 * i as f64;

        // Create bar path for this specific angle (in world coordinates)
        let cos_a = angle.cos() as f32;
        let sin_a = angle.sin() as f32;
        let x1 = xcenter + cos_a * inner_radius;
        let y1 = center_y + sin_a * inner_radius;
        let x2 = xcenter + cos_a * outer_radius;
        let y2 = center_y + sin_a * outer_radius;

        let bar_builder = PathBuilder::new();
        bar_builder.move_to(x1, y1);
        bar_builder.line_to(x2, y2);
        let bar_path = bar_builder.to_path();

        snapshot.append_stroke(&bar_path, &stroke, &bar_color);
    }
}
