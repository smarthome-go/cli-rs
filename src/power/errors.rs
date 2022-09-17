use smarthome_sdk_rs::Error as SdkError;
use std::fmt::Display;

pub type Result<T> = std::result::Result<T, Error>;

pub enum Error {
    GetSwitches(SdkError),
    GetPowerDrawData(SdkError),
    Unknown(SdkError),
    InvalidSwitch(String),
    PermissionDenied(String),
    NotEnoughPowerDrawData,
    ServerError,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::InvalidSwitch(switch_id) =>
                    format!("The switch `{switch_id}` does not exist"),
                Self::PermissionDenied(switch_id) => format!("You are either lacking permission to use switches or you do not have access to the switch `{switch_id}`"),
                Self::GetSwitches(err) => format!("Could not get switches: {err}"),
                    Self::NotEnoughPowerDrawData => "Not enough power draw data: averaging requires more power draw data: please wait a few hours".to_string(),
                Self::ServerError => "The server was unable to handle this switch".to_string(),
                Self::Unknown(err) => format!("Unknown error: {err}"),
                Self::GetPowerDrawData(err) => format!("Could not get power draw data: {err}"),
            }
        )
    }
}
