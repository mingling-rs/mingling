mingling::macros::pack!(ResultPack1 = String);
mingling::macros::pack_err!(ErrorPack1);
mingling::macros::pack_err!(ErrorPack2 = PathBuf);

pack!(ResultPack2 = (u8, String));
pack_err!(ErrorPack3);
pack_err!(ErrorPack4 = PathBuf);

pub mod sub {
    mingling::macros::pack!(ResultPack1 = String);
    mingling::macros::pack_err!(ErrorPack1);
    mingling::macros::pack_err!(ErrorPack2 = PathBuf);

    pack!(ResultPack2 = (u8, String));
    pack_err!(ErrorPack3);
    pack_err!(ErrorPack4 = PathBuf);
}
