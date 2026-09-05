mingling::macros::import_type!(std::io::Error);
mingling::macros::import_type!(GroupAlias1 = std::fmt::Error);

import_type!(std::num::ParseIntError);
import_type!(GroupAlias2 = serde_json::Error);

pub mod sub {
    mingling::macros::import_type!(std::io::Error);
    mingling::macros::import_type!(GroupAlias1 = std::fmt::Error);

    import_type!(std::num::ParseIntError);
    import_type!(GroupAlias2 = serde_json::Error);
}
