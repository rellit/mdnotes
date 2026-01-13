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

pub fn validate_due_inner(raw: &str) -> MdResult<String> {
    let parts: Vec<&str> = raw.split('-').collect();
    if parts.len() != 3 {
        return Err("Due date must be in YYYY-MM-DD format".into());
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
    Ok(raw.to_string())
}
