use crate::config::Profile;
use chrono::prelude::*;
use chrono::Duration;
use regex::Regex;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process::exit;
use text_placeholder::Template;

/// Match 'expected' to 'date' capture from regex and return the captured
/// named values in a HashMap.
///
/// Note: Hardwired to 'date' and only direct match of that one capture.
fn match_data(filename: String, pattern: String, expected: &String) -> HashMap<String, String> {
    let re = Regex::new(pattern.as_str()).unwrap_or_else(|err| {
        eprintln!("Format - {err}");
        exit(1);
    });
    let file = File::open(&filename).unwrap_or_else(|err| {
        eprintln!("File {filename} -- {err}");
        exit(1);
    });
    let reader = BufReader::new(file);
    for line_res in reader.lines() {
        let line = line_res.unwrap_or_else(|err| {
            eprintln!("Read line from {filename} -- {err}");
            exit(1);
        });
        let Some(caps) = re.captures(&line) else {
            continue;
        };
        let dict: HashMap<String, String> = re
            .capture_names()
            .flatten()
            .filter_map(|n| Some((n.to_string(), caps.name(n)?.as_str().to_string())))
            .collect();
        let Some(date) = dict.get("date") else {
            println!("Date not a captured field in format");
            break;
        };
        if date.eq(expected) {
            return dict;
        }
    }
    eprintln!("No line in data file matched expected date");
    exit(1)
}

/// Return the date of the next Friday in the form of yyyy-mm-dd.
///
/// Note: This is one of the current hardwired limitations.
fn get_next_friday() -> String {
    let now = chrono::Local::now();
    // Calculation is unsigned so added 7 to index of 4 to prevent overflow error.
    let days_ahead = (11 - now.weekday().num_days_from_monday()) % 7;
    let fri = now + Duration::days(i64::from(days_ahead));
    fri.date_naive().to_string()
}

pub fn apply_template(body: String, data: &mut HashMap<String, String>) -> String {
    let template = Template::new(body.as_str());
    let default = "".to_string();
    // TODO: Checking source for fill_with_function, it can return an error if key is
    // not found. This currently returns "" for unset values. Alter to use code similar
    // to fill_with_hashmap_strict()? If so, address the String to str when updating.
    let Ok(new_body) = template.fill_with_function(|key| {
        Some(Cow::Owned(
            data.get(&key.to_string()).unwrap_or(&default).to_string(),
        ))
    }) else {
        eprintln!("Template substitution failed.");
        exit(1)
    };
    new_body
}

pub fn prepare_body(mut settings: Profile) -> (Profile, String) {
    let Some(mut body) = settings.body.take() else {
        eprintln!("No body template for message.");
        exit(1)
    };
    let Some(file) = settings.event_file.clone() else {
        eprintln!("No event_file defined in profile");
        exit(1)
    };
    let Some(pattern) = settings.format.take() else {
        eprintln!("No format given for file.");
        exit(1)
    };
    // Next two checks concerning date_spec are to maintain
    // config file compatibility when different days of week are allowed.
    // TODO: Delete these for next version
    let Some(date_spec) = settings.date_spec.take() else {
        eprintln!("No date_spec defined in profile.");
        exit(1)
    };
    if date_spec != "Friday" {
        eprintln!("Only date_spec of 'Friday' is currently supported.");
        exit(1);
    }
    let expected = get_next_friday();
    let mut ans = match_data(file, pattern, &expected);
    body = apply_template(body, &mut ans);
    (settings, body)
}
