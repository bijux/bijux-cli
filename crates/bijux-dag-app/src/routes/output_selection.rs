use crate::commands::DagCli;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OutputSelection {
    Json,
    Human,
    Quiet,
}

pub(crate) fn output_selection(cli: &DagCli) -> OutputSelection {
    if cli.quiet {
        OutputSelection::Quiet
    } else if cli.json {
        OutputSelection::Json
    } else {
        OutputSelection::Human
    }
}

#[cfg(test)]
mod tests {
    use super::{output_selection, OutputSelection};
    use crate::commands::DagCli;
    use clap::Parser;

    #[test]
    fn prefers_quiet_over_json() {
        let cli = DagCli::parse_from(["bijux-dag", "--json", "--quiet", "doctor"]);
        assert_eq!(output_selection(&cli), OutputSelection::Quiet);
    }

    #[test]
    fn selects_json_when_enabled() {
        let cli = DagCli::parse_from(["bijux-dag", "--json", "doctor"]);
        assert_eq!(output_selection(&cli), OutputSelection::Json);
    }

    #[test]
    fn selects_human_by_default() {
        let cli = DagCli::parse_from(["bijux-dag", "doctor"]);
        assert_eq!(output_selection(&cli), OutputSelection::Human);
    }
}
