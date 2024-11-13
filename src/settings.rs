use std::convert::TryFrom;

use anyhow::{Error, Result};

use crate::errors::VolctlError;

enum OsdPosition {
    Center,
    BottomRight,
    MiddleRight,
    TopRight,
    TopCenter,
    TopLeft,
    MiddleLeft,
    BottomLeft,
    BottomCenter,
}

impl TryFrom<i32> for OsdPosition {
    type Error = Error;

    fn try_from(v: i32) -> Result<Self> {
        match v {
            x if x == OsdPosition::Center as i32 => Ok(OsdPosition::Center),
            x if x == OsdPosition::BottomRight as i32 => Ok(OsdPosition::BottomRight),
            x if x == OsdPosition::MiddleRight as i32 => Ok(OsdPosition::MiddleRight),
            x if x == OsdPosition::TopRight as i32 => Ok(OsdPosition::TopRight),
            x if x == OsdPosition::TopCenter as i32 => Ok(OsdPosition::TopCenter),
            x if x == OsdPosition::TopLeft as i32 => Ok(OsdPosition::TopLeft),
            x if x == OsdPosition::MiddleLeft as i32 => Ok(OsdPosition::MiddleLeft),
            x if x == OsdPosition::BottomLeft as i32 => Ok(OsdPosition::BottomLeft),
            x if x == OsdPosition::BottomCenter as i32 => Ok(OsdPosition::BottomCenter),
            _ => Err(VolctlError::InvalidSettingsValue {
                field: "OsdPosition".to_owned(),
                value: "value".to_owned(),
            }
            .into()),
        }
    }
}
