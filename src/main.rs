use std::{io::{self, Write}, fs};

fn main() {
    loop {
        print!("Provide space-separated divisor, base, and remainder, and any optional fourth argument to redirect to output.txt.\n'q' to quit: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("failed to read line");

        if input.trim() == "q".to_string() {
            break;
        }

        let args = input.split_whitespace().collect::<Vec<&str>>();

        if args.len() < 3 || args.len() > 4 {
            println!("Invalid number of arguments\n");
        } else {
            let divisor = (args[0]).parse::<usize>().unwrap_or_default();
            let base = (args[1]).parse::<usize>().unwrap_or_default();
            let remainder = (args[2]).parse::<usize>().unwrap_or_default();
            let re = lib::mod_regex(divisor, base, remainder);
            if args.len() == 4 {
                fs::write("output.txt", re).unwrap();
                println!("Regex written to 'output.txt'");
            } else {
                println!("{re}");
            }
        }
    }
    
}