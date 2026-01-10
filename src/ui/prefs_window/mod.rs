mod imp;

glib::wrapper! {
  pub struct PreferencesWindow(ObjectSubclass<imp::PreferencesWindow>)
      @extends gtk::Window, gtk::Widget,
      @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl PreferencesWindow {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }
}

impl Default for PreferencesWindow {
    fn default() -> Self {
        Self::new()
    }
}
