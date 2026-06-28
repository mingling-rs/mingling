#[mingling::macros::dispatcher_clap]
struct EntryClap1 {
    name: String,
    age: i32,
}

#[mingling::macros::dispatcher_clap]
#[command(name = "greet")]
pub struct EntryClap2 {
    name: String,
}

#[dispatcher_clap]
struct EntryClap3 {
    value: String,
}

#[dispatcher_clap]
pub struct EntryClap4 {
    value: i32,
}

pub mod sub {
    #[mingling::macros::dispatcher_clap]
    struct EntryClap1 {
        name: String,
    }

    #[dispatcher_clap]
    struct EntryClap3 {
        value: String,
    }
}
