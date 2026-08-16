use mingling::{
    macros::{chain, dispatcher, gen_program, pack, r_println, renderer},
    parser::Picker,
    this,
};

// Define a struct with Default and Clone derives
#[derive(Debug, Default, Clone)]
struct Global {
    name: String,
    age: i32,
}

fn main() {
    let mut program = ThisProgram::new();

    // Add a global resource here
    program.with_resource(Global::default());

    program.exec();
}

dispatcher!("modify", EntryModify);

pack!(DisplayGlobal = ());

#[chain]
fn modify(prev: EntryModify) {
    let (name, age) = Picker::<()>::new(prev.inner)
        .pick::<String>("--name")
        .pick::<i32>("--age")
        .unpack_directly();

    // Modify the global resource
    this::<ThisProgram>().modify_res(|r: &mut Global| {
        r.name = name;
        r.age = age
    });

    DisplayGlobal::default()
}

#[renderer]
fn render_global(_prev: DisplayGlobal) {
    // Read the global resource
    let global = this::<ThisProgram>().res_or_default::<Global>();
    r_println!("Name: {}, Age: {}", global.name, global.age)
}

gen_program!();
