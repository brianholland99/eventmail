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
fn capture(filename: String, pattern: String, expected: &String) -> HashMap<String, String> {
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
fn get_next_date(day: Weekday) -> String {
    let now = chrono::Local::now();
    // Calculation is unsigned so added 7 to prevent overflow error.
    let days_ahead = (day.num_days_from_monday() + 7 - now.weekday().num_days_from_monday()) % 7;
    let date = now + Duration::days(i64::from(days_ahead));
    date.date_naive().to_string()
}

pub fn apply_template(body: String, data: &HashMap<String, String>) -> String {
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

/// Takes body and settings from Profile and applies template substitutions.
pub fn prepare_text(mut settings: Profile) -> (Profile, String, String) {
    let Some(mut body) = settings.body.take() else {
        eprintln!("No body template for message.");
        exit(1)
    };
    let Some(mut subject) = settings.subject.take() else {
        eprintln!("No subject template for message.");
        exit(1);
    };
    let Some(date_spec) = settings.date_spec.take() else {
        eprintln!("No date_spec defined in profile.");
        exit(1)
    };
    let day = date_spec.parse::<Weekday>().unwrap_or_else(|err| {
        eprintln!("'date_spec not a weekday (E.g. 'Monday' or 'Mon')");
        eprintln!("date_spec = {date_spec}, Err = {err}");
        exit(1);
    });
    let date = get_next_date(day);
    let mut dict: HashMap<String, String>;
    match settings.event_file.take() {
        // Capture fields from event_file.
        Some(f) => {
            let pattern = settings.format.take().unwrap_or_else(|| {
                eprintln!("Event file set, but no format defined");
                exit(1);
            });
            dict = capture(f, pattern, &date)
        }
        // Just set date.
        None => {
            dict = HashMap::new();
            dict.insert("date".to_string(), date);
            ()
        }
    };

    body = apply_template(body, &dict);
    subject = apply_template(subject, &dict);
    (settings, body, subject)
}
