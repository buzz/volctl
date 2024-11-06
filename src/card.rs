#[derive(Debug, Clone, Default)]
pub struct CardData {
    pub index: u32,

    pub name: String,
    pub icon: String,

    pub profiles: Vec<(String, String)>,
    pub active_profile: String,
}
