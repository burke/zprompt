use getopts::Options;

// Near the top of the file, with the other `use` statements
mod widgets;
use widgets::{stash, path, prompt, ref_info, pending, exit, jobs, sync, async_data, space_if_git};

// Add this near the top with other use statements
mod formatting;

// Add this function after the imports and before the main function

// Add this near the top with other use statements
mod context;
use context::Context;

pub fn main() {
    let context = Context::new();

    let args: Vec<String> = std::env::args().collect();
    let mut opts = Options::new();
    opts.optflag("p", "", "print path info");
    opts.optflag("s", "", "print stash info");
    opts.optflag("a", "", "print async data");
    opts.optflag("r", "", "print ref info");
    opts.optflag("n", "", "print git pending");
    opts.optflag("y", "", "print git sync status");
    opts.optflag("e", "", "print exit status");
    opts.optflag("P", "", "print prompt char");
    opts.optflag("j", "", "print jobs");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(_) => panic!("invalid options"),
    };

    let names = &["p", "s", "a", "r", "n", "y", "e", "P", "j"].map(|s| s.to_string());

    if !matches.opts_present(names) {
        print_all(&context);
        return;
    }

    // print each option present
    for name in names {
        if matches.opt_present(name) {
            match name.as_ref() {
                "p" => print!("{}", path::generate()),
                "s" => print!("{}", stash::generate(&context)),
                "a" => print!("{}", async_data::generate()),
                "r" => print!("{}", ref_info::generate(&context)),
                "n" => print!("{}", pending::generate(&context)),
                "y" => print!("{}", sync::generate(&context)),
                "e" => print!("{}", exit::generate()),
                "P" => print!("{}", prompt::generate()),
                "j" => print!("{}", jobs::generate()),
                _ => panic!("invalid option"),
            }
        }
    }
}

fn print_all(context: &Context) {
    let mut out = String::new();
    out.push_str(&path::generate());
    out.push_str(&space_if_git::generate(context));
    out.push_str(&stash::generate(context));
    out.push_str(&async_data::generate());
    out.push_str(&ref_info::generate(context));
    out.push_str(&pending::generate(context));
    out.push_str(&sync::generate(context));
    out.push_str(" ");
    out.push_str(&exit::generate());
    out.push_str(&prompt::generate());
    out.push_str(&jobs::generate());
    out.push_str(" ");
    print!("{}", out);
}
