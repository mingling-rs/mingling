mingling::macros::group!(Group1);
mingling::macros::group!(GroupAlias1 = std::io::Error);

group!(Group2);
group!(GroupAlias2 = std::num::ParseIntError);

pub mod sub {
    mingling::macros::group!(Group1);
    mingling::macros::group!(GroupAlias1 = std::io::Error);

    group!(Group2);
    group!(GroupAlias2 = std::num::ParseIntError);
}
