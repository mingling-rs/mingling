use tools::{dependency_order::display_dependency_order, eprintln_cargo_style};

fn main() {
    let order = display_dependency_order();
    if order.is_empty() {
        eprintln_cargo_style!("could not find workspace root or mingling crates");
        std::process::exit(1);
    }
    for path in order {
        println!("{}", path.display());
    }
}
