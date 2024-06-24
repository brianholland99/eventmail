//! Use config file and command-line args to determine settings. First profile
//! in file is used if one is not passed on the command line.
//!
//! Order of precedence:
//! 1) Command-line args
//! 2) Profile
//! 3) Default settings
use clap::Parser;
use dirs::config_dir; // Determine default config directory
use indexmap::IndexMap; // Easy way to get default (first) profile
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use std::process::exit;
use toml::from_str;

const PROGRAM_NAME: &str = "eventmail";

#[derive(Parser)]
/// Command-line arguments
pub struct Args {
    /// Profile to use.
    ///
    /// If no profile is passed then first profile in file will be used.
    #[arg(short, long)]
    pub profile: Option<String>,

    /// Just display final data for sending (minus credentials).
    #[arg(long)]
    pub dry_run: bool,

    /// Path of config file to use.
    ///
    /// Default is to use 'eventmail.toml' in the default config directory.
    #[arg(short, long)]
    pub config: Option<String>,

    /// Just list any profiles with a 'doc' field in the config file.
    #[arg(short, long)]
    pub list: bool,
}

#[derive(Deserialize, Default)]
/// Configuration settings
/// Several fields are required for sending messages. All fields are set to
/// Option<T> due to the fact that inheritance may provide missing data.
/// Setting 'doc' is necessary if this profile is to appear in --list.
pub struct Profile {
    pub date_spec: Option<String>,
    pub event_file: Option<String>,
    pub format: Option<String>,
    pub server: Option<String>,
    pub port: Option<u16>,
    pub user: Option<String>,
    pub password: Option<String>,
    pub from: Option<String>,
    pub to: Option<Vec<String>>,
    pub subject: Option<String>,
    pub body: Option<String>,
    pub doc: Option<String>,
    pub inherit: Option<String>,
}

impl Profile {
    /// Inherit appropriate values, if not set in Self.
    ///
    /// Fields will not be overwritten if already set except for two cases.
    /// The 'inherit' field comes from the profile being inherited to support
    /// recursive inheritance. The 'doc' field is only used for --list, so no
    /// longer needed.
    fn inherit_from(self, input: Profile) -> Self {
        Self {
            date_spec: self.date_spec.or(input.date_spec),
            event_file: self.event_file.or(input.event_file),
            format: self.format.or(input.format),
            server: self.server.or(input.server),
            port: self.port.or(input.port),
            user: self.user.or(input.user),
            password: self.password.or(input.password),
            from: self.from.or(input.from),
            to: self.to.or(input.to),
            subject: self.subject.or(input.subject),
            body: self.body.or(input.body),
            doc: None,
            inherit: input.inherit,
        }
    }
}

/// Find config dir according to config_dir(). This is the XDG default on
/// Linux. It may not produce the XDG default for Windows or MacOS.
pub fn get_xdg_config_dir() -> PathBuf {
    let Some(mut config_dir) = config_dir() else {
        eprintln!("Unable to determine default configuration directory");
        exit(1);
    };
    config_dir.push(PROGRAM_NAME);
    config_dir
}

/// Read in the configuration data from the given file.
fn parse_toml_config_file(fname: PathBuf) -> IndexMap<String, Profile> {
    let Ok(config_spec) = fs::read_to_string(&fname) else {
        eprintln!("Could not read TOML config file '{}'", fname.display());
        exit(1);
    };
    let config = match from_str(config_spec.as_str()) {
        Ok(c) => c,
        Err(err) => {
            eprintln!(
                "Config file {} did not match specification.\n'{}'\n\n",
                fname.display(),
                err
            );
            exit(1);
        }
    };
    config
}

/// Get config file name using CLI config name or, if None, then determine
/// XDG default config file name.
fn get_config_file_name(args_config: Option<String>) -> PathBuf {
    let mut file_name = PathBuf::new();
    if let Some(cname) = args_config {
        file_name.push(cname);
    } else {
        file_name.push(get_xdg_config_dir());
        let mut fname = String::from(PROGRAM_NAME);
        fname.push_str(".toml");
        file_name.push(fname);
    }
    file_name
}

/// Get settings using CLI arguments and config file data.
///
/// CLI arguments always take precedence over config file settings.
///
/// Read config file and appropriate profile. If no profile was named
/// in args, then use first profile in config file.
/// Profiles are removed as used to prevent inheritance loops.
pub fn get_settings() -> (Profile, Args) {
    let mut args = Args::parse();
    let config_file = get_config_file_name(args.config.take());
    let mut config = parse_toml_config_file(config_file);
    if args.list {
        println!("The following profiles exist in the config file:");
        for (key, v) in config {
            // Only display documented (callable) profiles.
            if let Some(doc) = v.doc {
                println!("{key} - {doc}");
            }
        }
        exit(0);
    }
    let (name, mut profile) = match args.profile.take() {
        Some(c) => match config.swap_remove_entry(&c) {
            Some(d) => d,
            None => {
                eprintln!("Profile '{c}' was passed, but does not exist.");
                exit(1);
            }
        },
        None => match config.swap_remove_index(0) {
            Some(first) => first,
            None => {
                eprintln!("Did not find any profile in config file.");
                exit(1);
            }
        },
    };
    if profile.doc.is_none() {
        eprintln!("Profile {name} is not set as callable (no 'doc').");
        exit(1);
    }
    // Handle inheritance - remove as processing to detect loops
    while let Some(inherited) = profile.inherit.take() {
        let Some((_name, prof)) = config.swap_remove_entry(&inherited) else {
            eprintln!(
                "Inherited profile {inherited} either does not exist or inheritance loop exists."
            );
            exit(1);
        };
        profile = profile.inherit_from(prof);
    }
    (profile, args)
}
