#[mingling::macros::chain]
fn my_chain1(prev: Some1) -> Next {

}

#[mingling::macros::chain]
pub fn my_chain2(prev: Some2) -> Next {

}

#[mingling::macros::chain]
pub async fn my_chain3(prev: Some3) -> Next {

}

#[chain]
fn my_chain4(prev: Some4) {

}

#[chain]
pub fn my_chain5(prev: Some5) {

}

#[chain]
pub async fn my_chain6(prev: Some6) {

}

pub mod sub {
    #[mingling::macros::chain]
    fn my_chain1(prev: Some1) -> Next {

    }

    #[mingling::macros::chain]
    pub fn my_chain2(prev: Some2) -> Next {

    }

    #[mingling::macros::chain]
    pub async fn my_chain3(prev: Some3) -> Next {

    }

    #[chain]
    fn my_chain4(prev: Some4) {

    }

    #[chain]
    pub fn my_chain5(prev: Some5) {

    }

    #[chain]
    pub async fn my_chain6(prev: Some6) {

    }
}
