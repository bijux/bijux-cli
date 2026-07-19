use crate::ExitCode;
use std::collections::BTreeMap;

pub(crate) fn parse_resource_capacities(
    raw_assignments: &[String],
) -> Result<BTreeMap<String, u32>, ExitCode> {
    let mut capacities = BTreeMap::new();
    for raw in raw_assignments {
        let Some((name, amount)) = raw.split_once('=') else {
            return Err(ExitCode::from(2));
        };
        let name = name.trim();
        let amount = amount.trim();
        if name.is_empty() || name.chars().any(char::is_whitespace) {
            return Err(ExitCode::from(2));
        }
        let amount = amount.parse::<u32>().map_err(|_| ExitCode::from(2))?;
        if amount == 0 {
            return Err(ExitCode::from(2));
        }
        if capacities.insert(name.to_string(), amount).is_some() {
            return Err(ExitCode::from(2));
        }
    }
    Ok(capacities)
}

#[cfg(test)]
mod tests {
    use super::parse_resource_capacities;

    #[test]
    fn parses_named_resource_capacities() {
        let capacities = parse_resource_capacities(&[
            "database_slot=2".to_string(),
            "license.render=1".to_string(),
        ])
        .expect("capacities");
        assert_eq!(capacities.get("database_slot"), Some(&2));
        assert_eq!(capacities.get("license.render"), Some(&1));
    }

    #[test]
    fn rejects_invalid_capacity_assignments() {
        for raw in [
            vec!["database_slot".to_string()],
            vec!["database slot=1".to_string()],
            vec!["database_slot=0".to_string()],
            vec!["database_slot=two".to_string()],
            vec!["database_slot=1".to_string(), "database_slot=2".to_string()],
        ] {
            assert!(parse_resource_capacities(&raw).is_err(), "raw={raw:?}");
        }
    }
}
