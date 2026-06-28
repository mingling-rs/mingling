mingling::macros::dispatcher!("greet", CMDGreet => EntryGreet);
mingling::macros::dispatcher!("greet");
mingling::macros::dispatcher!("remote.add", CMDRemoteAdd => EntryRemoteAdd);
mingling::macros::dispatcher!("remote.add");

dispatcher!("delete", CMDDelete => EntryDelete);
dispatcher!("delete");
dispatcher!("remote.rm", CMDRemoteRm => EntryRemoteRm);
dispatcher!("remote.rm");

pub mod sub {
    mingling::macros::dispatcher!("greet", CMDGreet => EntryGreet);
    mingling::macros::dispatcher!("greet");

    dispatcher!("delete", CMDDelete => EntryDelete);
    dispatcher!("delete");
}
