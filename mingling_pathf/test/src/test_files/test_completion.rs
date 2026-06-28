#[mingling::macros::completion(Some1)]
fn my_completion1(ctx: &mingling::ShellContext) -> mingling::Suggest {
    mingling::Suggest::new()
}

#[mingling::macros::completion(Some2)]
pub fn my_completion2(ctx: &mingling::ShellContext) -> mingling::Suggest {
    mingling::Suggest::new()
}

#[completion(Some3)]
fn my_completion3(ctx: &mingling::ShellContext) -> mingling::Suggest {
    mingling::Suggest::new()
}

#[completion(Some4)]
pub fn my_completion4(ctx: &mingling::ShellContext) -> mingling::Suggest {
    mingling::Suggest::new()
}

pub mod sub {
    #[mingling::macros::completion(Some1)]
    fn my_completion1(ctx: &mingling::ShellContext) -> mingling::Suggest {
        mingling::Suggest::new()
    }

    #[mingling::macros::completion(Some2)]
    pub fn my_completion2(ctx: &mingling::ShellContext) -> mingling::Suggest {
        mingling::Suggest::new()
    }

    #[completion(Some3)]
    fn my_completion3(ctx: &mingling::ShellContext) -> mingling::Suggest {
        mingling::Suggest::new()
    }

    #[completion(Some4)]
    pub fn my_completion4(ctx: &mingling::ShellContext) -> mingling::Suggest {
        mingling::Suggest::new()
    }
}
