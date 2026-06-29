#[derive(Groupped)]
struct Derived1 {
    value: String,
}

#[derive(Groupped, Debug, Clone)]
struct Derived2 {
    value: i32,
}

#[derive(GrouppedSerialize)]
struct Derived3 {
    value: bool,
}

#[derive(Groupped)]
enum EnumDerived1 {
    A,
    B,
}

#[derive(GrouppedSerialize)]
enum EnumDerived2 {
    X(String),
    Y(i32),
}

pub mod sub {
    #[derive(Groupped)]
    struct Derived1 {
        value: String,
    }

    #[derive(GrouppedSerialize)]
    struct Derived3 {
        value: bool,
    }

    #[derive(Groupped)]
    enum EnumDerived1 {
        A,
    }
}
