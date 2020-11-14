pub mod translator;
pub mod commands;
mod stroke;
mod machine;

pub fn start() {
    println!("starting plojo...");
    machine::run();
}
