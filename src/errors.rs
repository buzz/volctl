use thiserror::Error;

#[derive(Error, Debug)]
pub enum VolctlError {
    #[error("invalid settings value `{value:?}` for field `{field:?}`")]
    InvalidSettingsValue { field: String, value: String },
}
