//! Example Lazy Resources
//!
//! > This example demonstrates how to use `LazyRes` for lazy resource initialization.
//!
//! Run:
//! ```bash
//! cargo run --manifest-path examples/example-lazy-resources/Cargo.toml --quiet none
//!
//! cargo run --manifest-path examples/example-lazy-resources/Cargo.toml --quiet show
//! ```
//!
//! Output:
//! ```plaintext
//! None
//!
//! Initialized
//! foo: bar
//! rust: lang
//! baz: qux
//! hello: world
//! key: value
//! ```

use std::collections::BTreeMap;
use std::io::Write;

use mingling::{LazyInit, LazyRes, prelude::*};

type Key = String;
type Value = String;

// Define a resource that requires time-consuming initialization
#[derive(Default, Clone)]
pub struct ResLargeData {
    pub data: BTreeMap<Key, Value>,
}

fn init_res_large_data() -> ResLargeData {
    // Perform time-consuming initialization here
    let mut data = BTreeMap::new();
    data.insert("foo".to_string(), "bar".to_string());
    data.insert("baz".to_string(), "qux".to_string());
    data.insert("hello".to_string(), "world".to_string());
    data.insert("rust".to_string(), "lang".to_string());
    data.insert("key".to_string(), "value".to_string());

    // Print to indicate initialization is complete
    println!("Initialized");
    ResLargeData { data }
}

dispatcher!("show", EntryShow);
dispatcher!("none", EntryNone);

#[derive(Grouped, Wrap)]
pub struct ResultShow(BTreeMap<Key, Value>);

fn main() {
    let mut program = ThisProgram::new();

    // --------- IMPORTANT ---------
    //                     _ Use lazy_init to create LazyRes<ResLargeData>
    //                    /
    //                    vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv
    program.with_resource(ResLargeData::lazy_init(init_res_large_data));
    // --------- IMPORTANT ---------

    program.exec_and_exit();
}

// Inject LazyRes instead of a normal resource
//                                           __________________________ Must use &mut because `get_ref` and `get_mut`
//                                          /                             both require mutable borrow
//                                          |     _____________________ Use LazyRes<ResLargeData>
//                                          |    /                        instead of ResLargeData
#[renderer] //                              vvvv vvvvvvvvvvvvvvvvvvvvv
fn render_entry_show(_args: EntryShow, res: &mut LazyRes<ResLargeData>) -> RenderResult {
    let mut render_result = RenderResult::new();

    //             _______ Initialization happens here
    //            /
    //            vvvvvvv
    let res = res.get_ref();
    for (key, value) in &res.data {
        writeln!(render_result, "{}: {}", key, value).ok();
    }
    render_result
}

// When not using LazyRes<ResLargeData>, it will not be initialized
#[renderer]
fn render_entry_none(_args: EntryNone) -> RenderResult {
    let mut render_result = RenderResult::new();
    writeln!(render_result, "None").ok();
    render_result
}

gen_program!();
