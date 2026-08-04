// Root-level metadata functions
#[mingling::macros::metadata(EntryGreet1)]
pub fn get_desc1() -> Description1 {
    Description1 {}
}

#[metadata(EntryGreet2)]
fn get_desc2() -> Description2 {
    Description2 {}
}

// Local DataType (defined in-crate) + foreign DataType
#[metadata(EntryGreet3)]
pub fn get_desc3() -> LocalType3 {
    LocalType3 {}
}

use std::collections::HashMap;

#[metadata(EntryGreet4)]
fn get_desc4() -> HashMap<String, String> {
    HashMap::new()
}

#[metadata(EntryGreet5)]
pub fn get_desc5() -> crate::fully::Qualified5 {
    crate::fully::Qualified5 {}
}

pub mod sub {
    #[mingling::macros::metadata(EntrySub1)]
    pub fn get_sub1() -> SubType1 {
        SubType1 {}
    }

    #[metadata(EntrySub2)]
    fn get_sub2() -> SubType2 {
        SubType2 {}
    }
}
