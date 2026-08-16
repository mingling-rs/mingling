//! Example Custom Pickable
//!
//! > This example demonstrates how to use the Pickable trait to add parsing for your types
//!
//! Run:
//! ```bash
//! cargo run --manifest-path examples/example-custom-pickable/Cargo.toml --quiet -- connect 127.0.0.1:5012
//! cargo run --manifest-path examples/example-custom-pickable/Cargo.toml --quiet -- connect 127.0.0.1
//! ```
//!
//! Output:
//! ```plaintext
//! Connected to "127.0.0.1:5012"
//! Failed to parse address
//! ```

use mingling::{macros::route, parser::Pickable, prelude::*, Grouped};
use std::io::Write;

// Define types that can be recognized by Mingling
//               ________________________ `Pickable` trait needs to implement Default
//              /                ________ The Grouped derive macro registers an ID for this type
//              |               /           Mingling uses this ID to identify the type
//              vvvvvvv         vvvvvvv
#[derive(Debug, Default, Clone, Grouped)]
pub struct Address {
    pub ip: [u8; 4],
    pub port: u16,
}

// --------- IMPORTANT ---------
impl Pickable for Address {
    type Output = Address;
    fn pick(args: &mut mingling::parser::Argument, flag: mingling::Flag) -> Option<Self::Output> {
        // Extract the raw string from Argument using the Flag
        let raw: String = args.pick_argument(flag)?.clone();

        // Use TryFrom to parse the address
        Address::try_from(raw).ok()
    }
}
// --------- IMPORTANT ---------

dispatcher!("connect", CMDConnect => EntryConnect);
pack!(ErrorParseAddressFailed = ());

#[chain]
fn handle_connect(prev: EntryConnect) -> Next {
    let connect: Address =
        route! { prev.pick_or_route((), ErrorParseAddressFailed::default()).unpack() };
    connect.to_chain()
}

/// Renders the connected address.
#[renderer]
pub fn render_address(addr: Address) -> RenderResult {
    let mut render_result = RenderResult::new();
    write!(render_result, "Connected to \"{}\"", addr).ok();
    render_result
}

/// Renders the error message when address parsing fails.
#[renderer]
pub fn render_error_parse_address_failed(_: ErrorParseAddressFailed) -> RenderResult {
    let mut render_result = RenderResult::new();
    write!(render_result, "Failed to parse address").ok();
    render_result
}

gen_program!();

fn main() {
    ThisProgram::new().exec_and_exit();
}

// Address conversion

impl TryFrom<String> for Address {
    type Error = String;

    fn try_from(raw: String) -> Result<Self, Self::Error> {
        // Expected format: "192.168.1.1:8080"
        let parts: Vec<&str> = raw.split(':').collect();
        if parts.len() != 2 {
            return Err("Invalid format: expected 'IP:PORT'".to_string());
        }

        let ip_str = parts[0];
        let port_str = parts[1];

        // Parse IP address (4 octets separated by dots)
        let ip_parts: Vec<&str> = ip_str.split('.').collect();
        if ip_parts.len() != 4 {
            return Err("Invalid IP address format".to_string());
        }

        let mut ip = [0u8; 4];
        for (i, part) in ip_parts.iter().enumerate() {
            ip[i] = part
                .parse::<u8>()
                .map_err(|_| format!("Invalid IP octet: {part}"))?;
        }

        // Parse port
        let port = port_str
            .parse::<u16>()
            .map_err(|_| format!("Invalid port: {port_str}"))?;

        Ok(Address { ip, port })
    }
}

impl From<Address> for String {
    fn from(addr: Address) -> String {
        format!(
            "{}.{}.{}.{}:{}",
            addr.ip[0], addr.ip[1], addr.ip[2], addr.ip[3], addr.port
        )
    }
}

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}.{}.{}.{}:{}",
            self.ip[0], self.ip[1], self.ip[2], self.ip[3], self.port
        )
    }
}
