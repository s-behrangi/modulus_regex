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

#[cfg(test)]
mod tests {
    use regex::Regex;
    use rand::{seq::IteratorRandom, rng};

    #[test]
    fn test_zero_remainder() {
        for divisor in 3..9 {
            for base in [2, 8, 10, 16] {
                let re = Regex::new(&lib::mod_regex(divisor, base, 0)).unwrap();
                for n in 0..1000 {
                    let repr: String;
                    match base {
                        2 => repr = format!("{:b}", n),
                        8 => repr = format!("{:o}", n),
                        10 => repr = format!("{}", n),
                        _ => repr = format!("{:x}", n),
                    }
                    assert_eq!(re.is_match(&repr), n % divisor == 0, "Failed zero-remainder test on n = {}, divisor = {}, base = {}", n, divisor, base);
                }
            }
        }
    }

    #[test]
    fn test_nonzero_remainder() {
        let mut rng = rng();
        for divisor in 3..9 {
            for base in [2, 8, 10, 16] {
                let remainder = (1..divisor).choose(&mut rng).unwrap();
                let re = Regex::new(&lib::mod_regex(divisor, base, remainder)).unwrap();
                for n in 0..1000 {
                    let repr: String;
                    match base {
                        2 => repr = format!("{:b}", n),
                        8 => repr = format!("{:o}", n),
                        10 => repr = format!("{}", n),
                        _ => repr = format!("{:x}", n),
                    }
                    assert_eq!(re.is_match(&repr), n % divisor == remainder, "Failed nonzero-remainder test on n = {}, divisor = {}, base = {}, r = {}", n, divisor, base, remainder);
                }
            }
        }
    }

}
