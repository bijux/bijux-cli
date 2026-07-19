pub(crate) const DAG_COMMAND_NAME: &str = "bijux-dag";

pub(crate) fn command_name() -> &'static str {
    DAG_COMMAND_NAME
}

#[cfg(test)]
mod tests {
    use super::{command_name, DAG_COMMAND_NAME};

    #[test]
    fn command_name_is_stable() {
        assert_eq!(command_name(), "bijux-dag");
    }

    #[test]
    fn command_name_constant_matches_function() {
        assert_eq!(DAG_COMMAND_NAME, command_name());
    }
}
