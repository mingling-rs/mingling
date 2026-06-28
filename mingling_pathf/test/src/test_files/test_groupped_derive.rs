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

pub mod sub {
    #[derive(Groupped)]
    struct Derived1 {
        value: String,
    }

    #[derive(GrouppedSerialize)]
    struct Derived3 {
        value: bool,
    }
}
