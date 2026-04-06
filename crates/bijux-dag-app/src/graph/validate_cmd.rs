use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ValidateCommandResponse {
    pub ok: bool,
}
