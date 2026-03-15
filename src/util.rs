use crate::MdResult;

pub fn parse_tags(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

pub fn validate_due(raw: &str) -> Result<String, String> {
    match validate_due_inner(raw) {
        Ok(val) => Ok(val),
        Err(err) => Err(err.0),
    }
}

/// Returns the shortest prefix of `id` that is unique among `all_ids`.
/// Falls back to the full `id` if no shorter prefix is unique.
pub fn shortest_unique_prefix(id: &str, all_ids: &[String]) -> String {
    for len in 4..=id.len() {
        let prefix = &id[..len];
        if all_ids
            .iter()
            .filter(|other| other.as_str().starts_with(prefix))
            .count()
            == 1
        {
            return prefix.to_string();
        }
    }
    id.to_string()
}

pub fn validate_due_inner(raw: &str) -> MdResult<String> {
    // Accept compact YYYYMMDD by inserting dashes.
    let normalized;
    let s: &str = if raw.len() == 8 && raw.bytes().all(|b| b.is_ascii_digit()) {
        normalized = format!("{}-{}-{}", &raw[..4], &raw[4..6], &raw[6..]);
        &normalized
    } else {
        raw
    };

    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 3 {
        return Err("Due date must be in YYYY-MM-DD or YYYYMMDD format".into());
    }
    let year = parts[0]
        .parse::<i32>()
        .map_err(|_| "Year must be numeric")?;
    if parts[0].len() != 4 || year <= 0 {
        return Err("Year must be four digits".into());
    }
    let month = parts[1]
        .parse::<u32>()
        .map_err(|_| "Month must be numeric")?;
    if !(1..=12).contains(&month) {
        return Err("Month must be between 01 and 12".into());
    }
    let day = parts[2].parse::<u32>().map_err(|_| "Day must be numeric")?;
    let is_leap = (year % 400 == 0) || (year % 4 == 0 && year % 100 != 0);
    let max_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap {
                29
            } else {
                28
            }
        }
        _ => 0,
    };
    if day == 0 || day > max_day {
        return Err("Day is out of range for the given month".into());
    }
    Ok(s.to_string())
}
