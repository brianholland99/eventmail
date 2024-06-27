use crate::config::{Args, Profile};
use lettre::address::AddressError;
use lettre::message::{header, Mailbox, Mailboxes};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use std::process::exit;

pub fn send_mail(settings: Profile, args: Args, body: String, subject: String) {
    // FROM
    let Some(from_string) = settings.from else {
        eprintln!("From must be set");
        exit(1);
    };
    let from: Result<Mailbox, AddressError> = from_string.parse();
    let from = match from {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Address error for 'from' -- {from_string} -- {e}");
            exit(1);
        }
    };

    // TO
    let Some(recipients) = settings.to else {
        eprintln!("Addresses 'to' must be set");
        exit(1);
    };
    let mut mailboxes = Mailboxes::new();
    for recipient in recipients.iter() {
        let mailbox = recipient.parse().unwrap_or_else(|err| {
            eprintln!("Address error for 'to': -- {recipient} -- {err}");
            exit(1);
        });
        mailboxes = mailboxes.with(mailbox);
    }
    let to_header: header::To = mailboxes.clone().into();

    if args.dry_run {
        println!("From: {from_string}");
        println!("To: {mailboxes}");
        println!("Subject: {subject}");
        println!("Body:\n{body}");
        exit(0);
    }
    let message = Message::builder()
        .from(from)
        .mailbox(to_header)
        .subject(subject)
        .body(body);

    let email = message.unwrap_or_else(|err| {
        eprintln!("Error: {err}");
        exit(1);
    });

    let Some(user) = settings.user else {
        eprintln!("A 'user' string must be set in profile.");
        exit(1);
    };
    let Some(pass) = settings.password else {
        eprintln!("A 'password' string must be set in profile.");
        exit(1);
    };
    let creds = Credentials::new(user, pass);
    let Some(relay) = settings.server else {
        eprintln!("A 'server' string must be set in profile.");
        exit(1);
    };

    let mailer = SmtpTransport::relay(&relay);
    let mut mailer = match mailer {
        Ok(c) => c.credentials(creds),
        Err(e) => {
            eprintln!("Error setting relay - {e}");
            exit(1);
        }
    };
    // Port is optional.
    if let Some(port) = settings.port {
        mailer = mailer.port(port);
    }
    let mailer = mailer.build();

    match mailer.send(&email) {
        Ok(_) => println!("Email send successfully!"),
        Err(e) => eprintln!("Could not send email! - {e}"),
    }
}
