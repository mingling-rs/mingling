#[mingling::macros::renderer]
fn my_renderer1(prev: Some1) {
}

#[mingling::macros::renderer]
pub fn my_renderer2(prev: Some2) {
}

#[renderer]
fn my_renderer3(prev: Some3) {
}

#[renderer]
pub fn my_renderer4(prev: Some4) {
}

pub mod sub {
    #[mingling::macros::renderer]
    fn my_renderer1(prev: Some1) {
    }

    #[mingling::macros::renderer]
    pub fn my_renderer2(prev: Some2) {
    }

    #[renderer]
    fn my_renderer3(prev: Some3) {
    }

    #[renderer]
    pub fn my_renderer4(prev: Some4) {
    }
}
