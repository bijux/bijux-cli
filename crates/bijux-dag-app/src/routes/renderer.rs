use serde_json::Value;

pub(crate) fn print_pretty_json(value: &Value) {
    println!("{}", serde_json::to_string_pretty(value).unwrap());
}

#[cfg(test)]
mod tests {
    use super::print_pretty_json;

    #[test]
    fn renderer_does_not_panic_for_valid_json() {
        let value = serde_json::json!({"status":"ok"});
        print_pretty_json(&value);
    }
}
