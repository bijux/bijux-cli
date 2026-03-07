use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct DoctorCommandResponse {
    pub status: String,
}
