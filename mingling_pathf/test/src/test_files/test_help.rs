#[mingling::macros::help]
fn my_help1(prev: Some1) {
}

#[mingling::macros::help]
pub fn my_help2(prev: Some2) {
}

#[help]
fn my_help3(prev: Some3) {
}

#[help]
pub fn my_help4(prev: Some4) {
}

pub mod sub {
    #[mingling::macros::help]
    fn my_help1(prev: Some1) {
    }

    #[mingling::macros::help]
    pub fn my_help2(prev: Some2) {
    }

    #[help]
    fn my_help3(prev: Some3) {
    }

    #[help]
    pub fn my_help4(prev: Some4) {
    }
}
