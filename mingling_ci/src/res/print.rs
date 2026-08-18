use colored::Colorize;
use mingling::config::ErrorOutput;
use mingling::hook::ProgramHook;
use mingling::{Program, macros::program_setup};
use mingling::{StringVec, this};

use crate::ThisProgram;

#[program_setup]
pub fn print_setup(p: &mut Program<ThisProgram>) {
    p.with_resource(CargoError::default());
    p.with_resource(CargoWarn::default());
    p.with_resource(CargoHelp::default());
    p.with_resource(CargoStatus::default());

    p.with_hook(ProgramHook::empty().on_begin::<_, ()>(move |_| {
        let p = this::<ThisProgram>();
        let silence_err = p.stdout_setting.error_output == ErrorOutput::Hide;

        p.modify_res(|r: &mut CargoError| r.silence = silence_err);
        p.modify_res(|r: &mut CargoWarn| r.silence = silence_err);
        p.modify_res(|r: &mut CargoHelp| r.silence = silence_err);
        p.modify_res(|r: &mut CargoStatus| r.silence = silence_err);
    }));
}

#[derive(Default, Clone)]
pub struct CargoError {
    silence: bool,
}

impl MessagePrinter for CargoError {
    fn format(&self, msg: impl Into<StringVec>) -> String {
        format!("{}: {}", "error".bold().bright_red(), msg.into().join(""))
    }

    fn std_mode(&self) -> StandardOutMode {
        if self.silence {
            StandardOutMode::Silence
        } else {
            StandardOutMode::Error
        }
    }
}

#[derive(Default, Clone)]
pub struct CargoWarn {
    silence: bool,
}

impl MessagePrinter for CargoWarn {
    fn format(&self, msg: impl Into<StringVec>) -> String {
        format!("{}: {}", "warning".bright_yellow(), msg.into().join(""))
    }

    fn std_mode(&self) -> StandardOutMode {
        if self.silence {
            StandardOutMode::Silence
        } else {
            StandardOutMode::Error
        }
    }
}

#[derive(Default, Clone)]
pub struct CargoHelp {
    silence: bool,
}

impl MessagePrinter for CargoHelp {
    fn format(&self, msg: impl Into<StringVec>) -> String {
        format!("{}: {}", "help".bright_white(), msg.into().join(""))
    }

    fn std_mode(&self) -> StandardOutMode {
        if self.silence {
            StandardOutMode::Silence
        } else {
            StandardOutMode::Error
        }
    }
}

#[derive(Default, Clone)]
pub struct CargoStatus {
    silence: bool,
}

impl MessagePrinter for CargoStatus {
    fn format(&self, msg: impl Into<StringVec>) -> String {
        let parts: Vec<String> = msg.into().to_vec();
        let first = if parts.is_empty() {
            String::new()
        } else {
            parts[0].trim().to_string()
        };

        let (prefix, content) = if first.is_empty() {
            // Empty: fall back to Info with full message
            ("Info".to_string(), parts.join(" "))
        } else if first.chars().count() == 1 {
            // Single character: prefix is Info, entire message is content
            ("Info".to_string(), parts.join(" "))
        } else if first.chars().count() <= 12 {
            // Single part that is a status prefix (no message after it)
            if parts.len() == 1 {
                ("Info".to_string(), first)
            } else {
                // First part is a status prefix, remaining parts are the message
                let content = parts[1..].join(" ").trim_start().to_string();
                (first, content)
            }
        } else {
            // First part too long: all is message, fall back to Info
            ("Info".to_string(), parts.join(" "))
        };

        let padding = " ".repeat(12usize.saturating_sub(prefix.chars().count()));

        format!(
            "{}{} {}",
            padding,
            prefix.bold().bright_green(),
            content.trim()
        )
    }

    fn std_mode(&self) -> StandardOutMode {
        if self.silence {
            StandardOutMode::Silence
        } else {
            StandardOutMode::Out
        }
    }
}

pub trait MessagePrinter {
    #[doc(hidden)]
    fn println(&self, msg: impl Into<StringVec>) {
        match self.std_mode() {
            StandardOutMode::Out => println!("{}", self.format(msg)),
            StandardOutMode::Error => eprintln!("{}", self.format(msg)),
            StandardOutMode::Silence => {}
        }
    }

    #[doc(hidden)]
    fn print(&self, msg: impl Into<StringVec>) {
        match self.std_mode() {
            StandardOutMode::Out => print!("{}", self.format(msg)),
            StandardOutMode::Error => eprint!("{}", self.format(msg)),
            StandardOutMode::Silence => {}
        }
    }

    /// Formats the message string before output.
    fn format(&self, msg: impl Into<StringVec>) -> String;

    /// Returns the standard output mode (stdout or stderr).
    fn std_mode(&self) -> StandardOutMode;
}

/// Specifies where standard output messages should be directed.
///
/// This enum determines whether messages are printed to stdout, stderr, or suppressed entirely.
#[repr(u8)]
pub enum StandardOutMode {
    /// Print messages to standard output (stdout).
    Out,
    /// Print messages to standard error (stderr).
    Error,
    /// Suppress all output.
    Silence,
}
