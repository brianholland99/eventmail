#![warn(missing_docs)]
//! Send out announcement emails.
//! Current version only supports emails about the next Friday using
//! data from a file. See README.md for more information.
mod config;
mod mail;
mod prepmail;
use crate::config::get_settings;
use crate::mail::send_mail;
use crate::prepmail::prepare_body;

fn main() {
    let (settings, args) = get_settings();
    let (settings, body) = prepare_body(settings);
    send_mail(settings, args, body);
}
