mingling::macros::dispatcher!("greet", CMDGreet => EntryGreet);
dispatcher!("delete", CMDDelete => EntryDelete);

pub mod sub {
    mingling::macros::dispatcher!("greet", CMDGreet => EntryGreet);
    dispatcher!("delete", CMDDelete => EntryDelete);
}
