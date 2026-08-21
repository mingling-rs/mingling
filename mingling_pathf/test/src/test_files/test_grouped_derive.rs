#[derive(Grouped)]
struct Derived1 {
    value: String,
}

#[derive(Grouped, Debug, Clone)]
struct Derived2 {
    value: i32,
}

#[derive(GroupedSerialize)]
struct Derived3 {
    value: bool,
}

#[derive(Grouped)]
enum EnumDerived1 {
    A,
    B,
}

#[derive(GroupedSerialize)]
enum EnumDerived2 {
    X(String),
    Y(i32),
}

#[derive(StructuralData)]
struct Derived4 {
    value: String,
}

pub mod sub {
    #[derive(Grouped)]
    struct Derived1 {
        value: String,
    }

    #[derive(GroupedSerialize)]
    struct Derived3 {
        value: bool,
    }

    #[derive(Grouped)]
    enum EnumDerived1 {
        A,
    }

    #[derive(StructuralData)]
    struct Derived4 {
        value: bool,
    }
}
