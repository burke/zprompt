// Near the top of the file, with the other `use` statements
mod widgets;
use widgets::{stash, path, prompt, ref_info, pending, exit, jobs, sync, async_data, space_if_git};

// Add this near the top with other use statements
mod formatting;

// Add this function after the imports and before the main function

// Add this near the top with other use statements
mod context;
use context::Context;

fn main() {
    let context = Context::new();
    let args: Vec<String> = std::env::args().collect();

    let format_string = if args.len() > 1 {
        args[1].clone()
    } else {
        "%p%X%s%a%r%n%y %e%P%j ".to_string()
    };

    print_formatted(&context, &format_string);
}

fn print_formatted(context: &Context, format: &str) {
    let mut output = String::new();
    let mut in_control = false;
    
    for c in format.chars() {
        if in_control {
            match c {
                'p' => output.push_str(&path::generate()),
                'X' => output.push_str(&space_if_git::generate(context)),
                's' => output.push_str(&stash::generate(context)),
                'a' => output.push_str(&async_data::generate()),
                'r' => output.push_str(&ref_info::generate(context)),
                'n' => output.push_str(&pending::generate(context)),
                'y' => output.push_str(&sync::generate(context)),
                'e' => output.push_str(&exit::generate()),
                'P' => output.push_str(&prompt::generate()),
                'j' => output.push_str(&jobs::generate()),
                '%' => output.push('%'), // Literal % when in control mode
                _ => {
                    output.push('%');  // Print the % for unrecognized control
                    output.push(c);    // And then print the unrecognized character
                }
            }
            in_control = false;
        } else if c == '%' {
            in_control = true;
        } else {
            output.push(c);
        }
    }
    
    // Handle a trailing '%' if the format string ends with it
    if in_control {
        output.push('%');
    }
    
    print!("{}", output);
}
