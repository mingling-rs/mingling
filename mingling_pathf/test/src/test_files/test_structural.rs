use std::io::Error;

mingling::macros::structural!(Struct1);
structural!(Struct2);
structural!(Error);

pub mod sub {
    use std::io::Error;

    mingling::macros::structural!(Struct1);
    structural!(Struct2);
    structural!(Error);
}
