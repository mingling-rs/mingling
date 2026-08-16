mingling::macros::dispatcher!("greet", EntryGreet);
mingling::macros::dispatcher!("greet");
mingling::macros::dispatcher!("remote.add", EntryRemoteAdd);
mingling::macros::dispatcher!("remote.add");

dispatcher!("delete", EntryDelete);
dispatcher!("delete");
dispatcher!("remote.rm", EntryRemoteRm);
dispatcher!("remote.rm");

pub mod sub {
    mingling::macros::dispatcher!("greet", EntryGreet);
    mingling::macros::dispatcher!("greet");

    dispatcher!("delete", EntryDelete);
    dispatcher!("delete");
}
