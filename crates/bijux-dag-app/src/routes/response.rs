use crate::ExitCode;
use serde_json::{json, Value};

pub(crate) fn simple_failure_payload(code: ExitCode, message: &str) -> Value {
    json!({
        "status": "invalid",
        "exit_code": if code == ExitCode::from(2) { 2 } else if code == ExitCode::from(3) { 3 } else if code == ExitCode::SUCCESS { 0 } else { 1 },
        "message": message,
    })
}

#[cfg(test)]
mod tests {
    use super::simple_failure_payload;
    use crate::ExitCode;

    #[test]
    fn builds_stable_failure_payload() {
        let payload = simple_failure_payload(ExitCode::from(3), "missing run");
        assert_eq!(payload.get("status").and_then(|v| v.as_str()), Some("invalid"));
        assert_eq!(payload.get("exit_code").and_then(|v| v.as_i64()), Some(3));
        assert_eq!(payload.get("message").and_then(|v| v.as_str()), Some("missing run"));
    }

    #[test]
    fn maps_known_exit_codes_to_stable_numbers() {
        assert_eq!(
            simple_failure_payload(ExitCode::SUCCESS, "ok")
                .get("exit_code")
                .and_then(|v| v.as_i64()),
            Some(0)
        );
        assert_eq!(
            simple_failure_payload(ExitCode::from(2), "bad")
                .get("exit_code")
                .and_then(|v| v.as_i64()),
            Some(2)
        );
        assert_eq!(
            simple_failure_payload(ExitCode::from(9), "unknown")
                .get("exit_code")
                .and_then(|v| v.as_i64()),
            Some(1)
        );
    }
}
