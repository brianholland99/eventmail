# eventmail

This program was written to send emails for events.  This is an
enhancement to the Python "lunchemail" program that sends out lunch
announcement using data from a text file. This now handles more than
just the next Friday that was used for the weekly lunch emails.

## Features

- TOML configuration file.
- Uses dirs::config_dir to locate default configuration dir. For Linux
  this is the XDG Base Directory Specification which is "~/.config/" followed
  by the program name. It seems like the config_dir on MacOS doesn't
  return what the XDG mentioned, but will check that in later version.
- Named profiles for different cases.
- Inheritance of profiles to reduce repeating common settings.
  (E.g., credentials for mail account)
- User-defined regex to extract information from the data file.
  The file is still processed one line at a time, so only a regex
  for a single line works.
- Body and subject of message can have substitutions of date and
  any fields captured from optional event_file.
- Allows user to specify email addresses, the subject, and connection
  information for sending the email.
- Profiles can specify any weekday for the next date.

## Usage

Running the program with no arguments will look for the configuration file
in the default location (E.g., ~/.config/eventmail/eventmail.toml on
Linux). If the first program has 'doc' defined, then it will be executed.
If a different profile is desired, it must be specified on the command line.

For command line options, run the program with the --help flag.

## Configuration

The configuration file is used to define the necessary key-value pairs for
constructing and sending the emails. The options passed when running the
program can select a configuration file other than the default and choose a
profile other than the first.

Note: The following forms for email addresses are acceptable.
   - "User Name <usermailname@gmail.com>"
   - "usermailname@gmail.com"

The configuration file consists of **profiles** defined by sections
whose name is the profile's name and containing key-value pairs holding
the data.

```TOML
# Example profile section "lunch"
[lunch]
```
Profiles may be ***callable*** or just have definitions that will be inherited
by other profiles. Even callable profiles may be inherited by other profiles
that can redefine some of the keys.

### Key-value pairs

The key-value pairs defined within a section apply to that profile.

Warning: The program currently ignores all keys that are not expected.
Later versions may add new keys. To be safe, don't use keys that are not
defined.

***There are no keys that are required to be in all profiles. However, many
current keys are required to be used for sending of messages. All callable
profiles must have those required fields after inheritance is applied.

The keys "port" and "inherit" are totally optional even after all inheritance
is applied. The "event_file" and "format" fields are optional if only the
date is needed for substitution into the body and subject.

#### Key - doc

Defining a value for 'doc' will allow the profile to be used directly, either
by being the first profile or being specified on the command line. It will
also appear in a listing of profiles.

This should not be defined this for profiles that are missing required keys
after inheritance or are only intended to be inherited.

```TOML
# Example
doc = "Send Friday lunch announcements."
```

#### Key - date_spec

The 'date_spec' currently only supports a weekday. Both fully spelled-out
weekdays (E.g., "Monday") and three letter abbreviations (E.g., "Mon") are
accepted. The weekday specified means the date of the upcoming weekday
including today.

```TOML
# Example
date_spec = "Friday"
```

#### Key - event_file

The 'event_file' is only required if data other than the date is to be
substituted in the subject or body.

```TOML
# Example
event_file = "/home/my_user/my_event_file.txt"
```

#### Key - format

The 'format' is ignored unless the 'event_file' is set and in that case it
is required. This 'format' describes the format of the lines in a regex
pattern with fields to capture that will be used for substitutions in the
subject or body. The format needs to include "date" as a capture that will be
checked against the date indicated by the 'date_spec' field.

```TOML
# Example format capturing "date" and "location"
format = '''^(?P<date>\S+) (?P<location>.*)$'''
```

#### Key - from

The 'from' value defines the string used for the "From" field in the email.
This can be either form of address described above. If the 'from' value
does not parse as a valid address, the program will terminate and state
that the 'from' was invalid.

```TOML
# Example
from = "This User <thisuser@gmail.com>"
```

#### Key - to

The 'to' value is a list of recipient addresses in either form mentioned
above. If any of the items in the list do not parse as valid addresses, the
program will terminate with a message stating which address was invalid.

```TOML
# Example
to = [
  "Some User <someuser@gmail.com>",
  "someotheruser@gmail.com"
]
```

#### Key - subject

The 'subject' value is a string that will be used as the "Subject" of the
email.

```TOML
# Example
subject = "Friday's lunch"
```

#### Key - body

The 'body' is a string with optional placeholders (I.e., placeholder_name
surrounded by double curly braces) that will be substituted
with values captured by the 'format' regex. Not all captured values need
to be used and any placeholder that there is no captured value for will be
substituted with the empty string.

```TOML
# Example
body = '''All,

Lunch Friday {{date}} will be at {{location}}, 11:15.

See you there!
Mike
'''
```

#### Keys - server, user, password, port

The 'server', 'user', and 'password' values are used to connect and
authenticate to the server. The 'server' is a string containing the address
of the mail server. The 'user' and 'password' are the credentials strings
for logging into the server. The optional 'port' is an integer that, if
defined, will be used for the TLS connection instead of the default port of
465. For gmail, the default works at the time this was tested.

```TOML
# Example
server = "smtp.not_gmail.com"
user = "my_user"
password = "xxxx xxxx xxxx xxxx"
port = 777 # 465 is used if this isn't defined
```

#### Key - inherit

The 'inherit' value is the name of a profile to inherit. Any of the
keys above other than doc that are not set will use the values defined in
the inherited profile. After that process, an "inherit" setting in the
inherited profile causes this to repeat until it gets to a profile with
'inherit' not set.

Note: The program will terminate with a message if the 'inherit' value
does not match a defined profile or if a loop is encountered.


### Example configuration file

```TOML
# The first profile is the default one to use if 'doc' is defined.
[lunch]
doc = "Normal lunch email"
date_spec = "Friday"
event_file = "/home/my_user/event_file.txt"
format = '''^(?P<date>\S+) (?P<location>.*)$'''
to = [
  "Some User <someuser@gmail.com>",
  "someotheruser@gmail.com"
]
subject = "Lunch Friday"
body = '''All,

Lunch Friday {{date}} will be at {{location}}, 11:15.

See you there!
NAME
'''
inherit = "comms"

####################################################
[lunch-absent]
doc = "Absentee lunch notice"
body = '''All,

Lunch Friday {{date}} will be at {{location}}, 11:15.

I won't be there this time; have fun!
NAME
'''
inherit = "lunch"

####################################################
[comms]
# No 'doc' for this one since it cannot be directly used. It is missing
# required information.
server = "smtp.gmail.com"
#port = 465
user = "my_user"
password = "xxxx xxxx xxxx xxxx"
from = "My User <my_user@gmail.com>"
```

## Some Current Limitations

- The date_spec can only specify a weekday to mean next date for that weekday.

## Thoughts
- Determine if date_spec makes sense to expand beyond just the next date
  matching the weekday.