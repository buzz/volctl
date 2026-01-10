use std::{cell::RefCell, rc::Rc};

use glib::subclass::object::{ObjectImpl, ObjectImplExt};
use glib::subclass::types::{ObjectSubclass, ObjectSubclassExt};
use gtk::gio;
use gtk::prelude::{
    BoxExt, CheckButtonExt, GridExt, GtkWindowExt, RangeExt, SettingsExt, SettingsExtManual,
    WidgetExt,
};
use gtk::subclass::{widget::WidgetImpl, window::WindowImpl};
use gtk::{
    Adjustment, Align, CheckButton, Entry, Grid, HeaderBar, Label, Orientation, Scale, SizeGroup,
    SizeGroupMode, Switch,
};

use crate::constants::{
    SETTINGS_ALLOW_EXTRA_VOLUME, SETTINGS_AUTO_CLOSE, SETTINGS_MIXER_COMMAND,
    SETTINGS_MOUSE_WHEEL_STEP, SETTINGS_OSD_ENABLED, SETTINGS_OSD_POSITION, SETTINGS_OSD_SCALE,
    SETTINGS_OSD_TIMEOUT, SETTINGS_PATH, SETTINGS_SCHEMA_KEY, SETTINGS_SHOW_PERCENTAGE,
    SETTINGS_TIMEOUT, SETTINGS_VU_ENABLED,
};

const MARGIN: i32 = 12;
const COL_SPACING: i32 = 36;
const ROW_SPACING: i32 = 24;
const OSD_GRID_SPACING: i32 = 8;

const OSD_POSITION_NAMES_X: [&str; 3] = ["left", "center", "right"];
const OSD_POSITION_NAMES_Y: [&str; 3] = ["top", "center", "bottom"];

pub struct PreferencesWindow {
    settings: RefCell<Option<gio::Settings>>,
    label_size_group: SizeGroup,
    row_timeout: RefCell<Option<Scale>>,
    row_osd_timeout: RefCell<Option<Scale>>,
    row_osd_size: RefCell<Option<Scale>>,
    row_osd_position_group: RefCell<Vec<CheckButton>>,
}

impl Default for PreferencesWindow {
    fn default() -> Self {
        Self {
            settings: RefCell::new(Some(gio::Settings::with_path(
                SETTINGS_SCHEMA_KEY,
                SETTINGS_PATH,
            ))),
            label_size_group: SizeGroup::new(SizeGroupMode::Horizontal),
            row_timeout: RefCell::new(None),
            row_osd_timeout: RefCell::new(None),
            row_osd_size: RefCell::new(None),
            row_osd_position_group: RefCell::new(Vec::new()),
        }
    }
}

#[glib::object_subclass]
impl ObjectSubclass for PreferencesWindow {
    const NAME: &'static str = "VolctlPreferencesWindow";
    type Type = super::PreferencesWindow;
    type ParentType = gtk::Window;
}

impl ObjectImpl for PreferencesWindow {
    fn constructed(&self) {
        self.parent_constructed();
        let obj = self.obj();

        let settings = self
            .settings
            .borrow()
            .clone()
            .expect("Settings should be initialized");

        obj.set_title(Some("volctl Preferences"));
        obj.set_icon_name(Some("preferences-desktop"));
        obj.set_resizable(false);
        obj.set_default_size(650, -1);

        let header_bar = HeaderBar::new();
        obj.set_titlebar(Some(&header_bar));

        let grid = Grid::builder()
            .margin_top(MARGIN)
            .margin_bottom(MARGIN * 2)
            .margin_start(MARGIN)
            .margin_end(MARGIN)
            .column_spacing(COL_SPACING)
            .row_spacing(ROW_SPACING)
            .column_homogeneous(false)
            .build();

        let mut row = 0;

        // Tray icon section
        self.create_section_label(&grid, "Tray icon", &mut row);
        self.add_scale(ScaleParams {
            grid: &grid,
            settings: &settings,
            key: SETTINGS_MOUSE_WHEEL_STEP,
            label_text: "Mouse wheel step",
            adjustment: &Adjustment::new(1.0, 1.0, 50.0, 1.0, 1.0, 0.0),
            digits: 1,
            format_value_func: |_, value| format!("{:.1} %", value),
            row: &mut row,
        });

        self.add_mixer_command_entry(&grid, &settings, &mut row);
        self.add_prefer_gtksi_switch(&grid, &settings, &mut row);

        // Volume sliders section
        self.create_section_label(&grid, "Volume sliders", &mut row);
        self.add_switch(
            &grid,
            &settings,
            SETTINGS_ALLOW_EXTRA_VOLUME,
            "Allow extra volume",
            &mut row,
        );
        self.add_switch(
            &grid,
            &settings,
            SETTINGS_SHOW_PERCENTAGE,
            "Show percentage",
            &mut row,
        );
        self.add_switch(
            &grid,
            &settings,
            SETTINGS_VU_ENABLED,
            "Show volume meters",
            &mut row,
        );
        self.add_switch(
            &grid,
            &settings,
            SETTINGS_AUTO_CLOSE,
            "Enable auto-close",
            &mut row,
        );

        let timeout_scale = self.add_scale(ScaleParams {
            grid: &grid,
            settings: &settings,
            key: SETTINGS_TIMEOUT,
            label_text: "Auto-close timeout",
            adjustment: &Adjustment::new(500.0, 500.0, 15000.0, 100.0, 100.0, 0.0),
            digits: 0,
            format_value_func: |_, value| format!("{:.1} sec", value / 1000.0),
            row: &mut row,
        });
        *self.row_timeout.borrow_mut() = Some(timeout_scale);

        // OSD section
        self.create_section_label(&grid, "On-screen display", &mut row);
        self.add_switch(
            &grid,
            &settings,
            SETTINGS_OSD_ENABLED,
            "Enable OSD",
            &mut row,
        );

        let osd_timeout_scale = self.add_scale(ScaleParams {
            grid: &grid,
            settings: &settings,
            key: SETTINGS_OSD_TIMEOUT,
            label_text: "OSD timeout",
            adjustment: &Adjustment::new(0.0, 0.0, 10000.0, 100.0, 100.0, 0.0),
            digits: 0,
            format_value_func: |_, value| format!("{:.1} sec", value / 1000.0),
            row: &mut row,
        });
        *self.row_osd_timeout.borrow_mut() = Some(osd_timeout_scale);

        let osd_size_scale = self.add_scale(ScaleParams {
            grid: &grid,
            settings: &settings,
            key: SETTINGS_OSD_SCALE,
            label_text: "OSD size",
            adjustment: &Adjustment::new(50.0, 50.0, 500.0, 1.0, 1.0, 0.0),
            digits: 0,
            format_value_func: |_, value| format!("{} %", value.round() as i32),
            row: &mut row,
        });
        *self.row_osd_size.borrow_mut() = Some(osd_size_scale);

        self.add_osd_position(&grid, &settings, &mut row);

        self.update_rows(&settings);
        obj.set_child(Some(&grid));
    }
}

impl PreferencesWindow {
    fn create_section_label(&self, grid: &Grid, caption: &str, row: &mut i32) {
        let label = Label::builder()
            .use_markup(true)
            .label(format!("<b>{}</b>", caption))
            .halign(Align::Start)
            .hexpand(true)
            .margin_top(6)
            .build();
        grid.attach(&label, 0, *row, 2, 1);
        *row += 1;
    }

    fn add_label(&self, grid: &Grid, label_text: &str, row: &mut i32) {
        let label = Label::builder()
            .halign(Align::Start)
            .valign(Align::Center)
            .label(label_text)
            .margin_bottom(6)
            .margin_start(MARGIN)
            .margin_top(6)
            .xalign(0.0)
            .build();

        self.label_size_group.add_widget(&label);
        grid.attach(&label, 0, *row, 1, 1);
    }

    fn add_switch(
        &self,
        grid: &Grid,
        settings: &gio::Settings,
        key: &str,
        label_text: &str,
        row: &mut i32,
    ) {
        self.add_label(grid, label_text, row);

        let switch = Switch::builder()
            .halign(Align::End)
            .valign(Align::Center)
            .build();

        let _ = settings.bind(key, &switch, "active");
        grid.attach(&switch, 1, *row, 1, 1);
        *row += 1;
    }

    fn add_scale<P: Fn(&Scale, f64) -> String + 'static>(&self, params: ScaleParams<P>) -> Scale {
        self.add_label(params.grid, params.label_text, params.row);

        let value_label = Label::builder().halign(Align::Start).build();

        let scale = Scale::builder()
            .adjustment(params.adjustment)
            .digits(params.digits)
            .hexpand(true)
            .halign(Align::Fill)
            .orientation(Orientation::Horizontal)
            .build();

        // Sync the label text with the scale value
        let format_func = Rc::new(params.format_value_func);
        let format_clone = format_func.clone();
        let value_label_clone = value_label.clone();
        value_label.set_label(&format_clone(&scale, scale.value()));
        scale.connect_value_changed(move |s| {
            value_label_clone.set_label(&format_clone(s, s.value()));
        });

        let hbox = gtk::Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(MARGIN)
            .valign(Align::Center)
            .build();
        hbox.append(&scale);
        hbox.append(&value_label);

        let _ = params
            .settings
            .bind(params.key, &scale.adjustment(), "value");

        params.grid.attach(&hbox, 1, *params.row, 1, 1);
        *params.row += 1;

        scale
    }

    fn add_prefer_gtksi_switch(&self, grid: &Grid, settings: &gio::Settings, row: &mut i32) {
        self.add_switch(grid, settings, "prefer-gtksi", "Prefer XEmbed", row);
    }

    fn add_mixer_command_entry(&self, grid: &Grid, settings: &gio::Settings, row: &mut i32) {
        self.add_label(grid, "Custom mixer command", row);
        let entry = Entry::builder()
            .placeholder_text("Default: pavucontrol")
            .hexpand(true)
            .halign(Align::Fill)
            .build();
        let _ = settings.bind(SETTINGS_MIXER_COMMAND, &entry, "text");
        grid.attach(&entry, 1, *row, 1, 1);
        *row += 1;
    }

    fn add_osd_position(&self, grid: &Grid, settings: &gio::Settings, row: &mut i32) {
        self.add_label(grid, "OSD position", row);
        let pos_grid = Grid::builder()
            .column_spacing(OSD_GRID_SPACING)
            .row_spacing(OSD_GRID_SPACING)
            .halign(Align::End) // Keep the 3x3 grid small and to the right
            .build();

        let current_pos = settings.string(SETTINGS_OSD_POSITION);
        let mut radio_buttons = Vec::new();
        let mut first_radio: Option<CheckButton> = None;

        for (top, yname) in OSD_POSITION_NAMES_Y.iter().enumerate() {
            for (left, xname) in OSD_POSITION_NAMES_X.iter().enumerate() {
                let name = format!("{}-{}", yname, xname);
                let radio = CheckButton::builder().build();

                if let Some(first) = &first_radio {
                    radio.set_group(Some(first));
                }
                if name == current_pos {
                    radio.set_active(true);
                }

                let pos_name = name.clone();
                radio.connect_toggled(glib::clone!(
                    #[weak]
                    settings,
                    move |r| {
                        if r.is_active() {
                            let _ = settings.set_string(SETTINGS_OSD_POSITION, &pos_name);
                        }
                    }
                ));

                pos_grid.attach(&radio, left as i32, top as i32, 1, 1);
                if first_radio.is_none() {
                    first_radio = Some(radio.clone());
                }
                radio_buttons.push(radio);
            }
        }

        grid.attach(&pos_grid, 1, *row, 1, 1);
        *row += 1;
        *self.row_osd_position_group.borrow_mut() = radio_buttons;
    }

    fn update_rows(&self, settings: &gio::Settings) {
        let auto_close = settings.boolean(SETTINGS_AUTO_CLOSE);
        if let Some(ref s) = *self.row_timeout.borrow() {
            s.set_sensitive(auto_close);
        }

        let osd_enabled = settings.boolean(SETTINGS_OSD_ENABLED);
        if let Some(ref s) = *self.row_osd_timeout.borrow() {
            s.set_sensitive(osd_enabled);
        }
        if let Some(ref s) = *self.row_osd_size.borrow() {
            s.set_sensitive(osd_enabled);
        }
        for r in self.row_osd_position_group.borrow().iter() {
            r.set_sensitive(osd_enabled);
        }
    }
}

impl WindowImpl for PreferencesWindow {}
impl WidgetImpl for PreferencesWindow {}

struct ScaleParams<'a, P: Fn(&Scale, f64) -> String + 'static> {
    grid: &'a Grid,
    settings: &'a gio::Settings,
    key: &'a str,
    label_text: &'a str,
    adjustment: &'a Adjustment,
    digits: i32,
    format_value_func: P,
    row: &'a mut i32,
}
