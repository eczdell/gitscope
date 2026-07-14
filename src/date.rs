// ═══════════════════════════════════════════════════════════════════════════
// Date Formatting
// ═══════════════════════════════════════════════════════════════════════════

fn is_leap(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

fn secs_to_ymdhm(secs: i64) -> (i64, u32, u32, u32, u32) {
    let days = secs / 86400;
    let remaining = secs % 86400;
    let hour = (remaining / 3600) as u32;
    let min = ((remaining % 3600) / 60) as u32;

    let mut year = 1970i64;
    let mut day_of_year = days;
    loop {
        let diy = if is_leap(year) { 366 } else { 365 };
        if day_of_year < diy {
            break;
        }
        day_of_year -= diy;
        year += 1;
    }
    let md = if is_leap(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut month = 1u32;
    for &d in &md {
        if day_of_year < d as i64 {
            break;
        }
        day_of_year -= d as i64;
        month += 1;
    }
    (year, month, (day_of_year + 1) as u32, hour, min)
}

pub fn format_date_full(time: git2::Time) -> String {
    let secs = time.seconds();
    let offset = time.offset_minutes();
    let (y, mo, d, h, mi) = secs_to_ymdhm(secs);
    let tz_h = offset / 60;
    let tz_m = (offset.abs() % 60) as u32;
    let sign = if offset >= 0 { '+' } else { '-' };
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:00{}{:02}:{:02}",
        y, mo, d, h, mi, sign, tz_h.abs(), tz_m
    )
}

#[allow(dead_code)]
pub fn format_date_short(time: git2::Time) -> String {
    let secs = time.seconds();
    let (y, mo, d, _, _) = secs_to_ymdhm(secs);
    format!("{:04}-{:02}-{:02}", y, mo, d)
}

