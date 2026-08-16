mingling::macros::dispatcher!("greet", EntryGreet);
dispatcher!("delete", EntryDelete);

pub mod sub {
    mingling::macros::dispatcher!("greet", EntryGreet);
    dispatcher!("delete", EntryDelete);
}
