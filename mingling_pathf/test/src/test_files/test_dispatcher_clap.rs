// Basic: entry type only (no CMD type specified)
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

// With CMD type
#[dispatcher_clap("greet", )]
struct EntryWithCmd {
    name: String,
}

// With CMD + error
#[dispatcher_clap("delete", error = ErrorDelete)]
struct EntryWithError {
    id: u64,
}

// With CMD + help
#[dispatcher_clap("helpcmd", help = true)]
struct EntryWithHelp {
    verbose: bool,
}

// With CMD + error + help
#[dispatcher_clap("full", error = ErrorFull, help = true)]
struct EntryFull {
    all: bool,
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

    #[dispatcher_clap("greet", )]
    struct EntryWithCmd {
        name: String,
    }

    #[dispatcher_clap("delete", error = ErrorDelete)]
    struct EntryWithError {
        id: u64,
    }

    #[dispatcher_clap("helpcmd", help = true)]
    struct EntryWithHelp {
        verbose: bool,
    }
}
