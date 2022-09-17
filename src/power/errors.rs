use std::fmt::Display;

pub enum PowerError {
    GetSwitches(smarthome_sdk_rs::Error),
    InvalidSwitch(String),
    PermissionDenied(String),
    ServerError,
    Unknown(smarthome_sdk_rs::Error),
}

impl Display for PowerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::InvalidSwitch(switch_id) =>
                    format!("The switch `{switch_id}` does not exist"),
                Self::PermissionDenied(switch_id) => format!("You are either lacking permission to use switches or you do not have access to the switch `{switch_id}`"),
                Self::ServerError => "The server was unable to handle this switch".to_string(),
                Self::GetSwitches(err) => format!("Could not get switches: {err}"),
                Self::Unknown(err) => format!("Unknown error: {err}")
            }
        )
    }
}
