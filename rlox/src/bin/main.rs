extern crate structopt;

use rlox::Lox;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Options {
    ///Display ast instead of executing
    #[structopt(short, long)]
    ast: bool,

    /// Lox file to run, if none execute REPL
    file: Option<String>,
}

fn main() {
    let opt = Options::from_args();
    let mut lox = Lox::new();

    if let Some(file) = opt.file {
        lox.run_file(&file, opt.ast);
    } else {
        lox.run_prompt(opt.ast);
    }
}
