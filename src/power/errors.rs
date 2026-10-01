use smarthome_sdk_rs::Error as SdkError;
use std::fmt::Display;

pub type Result<T> = std::result::Result<T, Error>;

pub enum Error {
    GetDevices(SdkError),
    GetPowerDrawData(SdkError),
    Smarthome(SdkError),
    InvalidDevice(String),
    NoPowerCapability(String),
    PermissionDenied(String),
    NotEnoughPowerDrawData,
    Json(serde_json::Error),
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidDevice(id) => write!(f, "the device `{id}` does not exist or is inaccessible"),
            Self::NoPowerCapability(id) => write!(f, "the device `{id}` cannot be switched on or off"),
            Self::PermissionDenied(id) => write!(
                f,
                "you are either lacking permission to use devices or you do not have access to the device `{id}`"
            ),
            Self::GetDevices(err) => write!(f, "could not get devices: {err}"),
            Self::NotEnoughPowerDrawData => write!(
                f,
                "not enough power draw data: averaging requires more power draw data, please wait a few hours"
            ),
            Self::Smarthome(err) => write!(f, "{err}"),
            Self::GetPowerDrawData(err) => write!(f, "could not get power draw data: {err}"),
            Self::Json(err) => write!(f, "could not encode JSON: {err}"),
        }
    }
}
