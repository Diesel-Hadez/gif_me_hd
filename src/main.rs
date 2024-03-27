use gif_me_hd::decoder;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("Not enough arguments!");
    }
    let file = &args[1];
    let gif_file = decoder::load(&file[..]).unwrap();
    println!("Logical Screen Descriptor: {:#?}", gif_file.logical_screen_descriptor);
    match gif_file.global_color_table {
        Some(gct) => println!("Global Color Table: {:#?}", gct),
        None => println!("No Global Color Table"),
    }
}
