use wasm_bindgen::prelude::wasm_bindgen;
use std::collections::HashMap;

#[wasm_bindgen]
pub fn mod_regex(divisor : usize, base : usize, remainder: usize) -> String {
    // handle degenerate cases:
    if base == 0 {
        if remainder == 0 {
            return format!("^(0{{{divisor}}})*$");
        } else {
            return format!("^0{{{remainder}}}(0{{{divisor}}})*$");
        }
    }
    
    let base_encoding: Vec<String> = ('0'..='9')
                                    .chain('a'..='f')
                                    .map(|x| x.to_string())
                                    .collect();
    let mut dfa: Vec<HashMap::<String, usize>> = (0..divisor).map(|_| HashMap::<String, usize>::new()).collect();
    // invariant: a State's index in the vector corresponds to its remainder
    // later on, "removing" a state just corresponds to rearranging edges so none from the remaining states reach the given state
    // we stuff everything in one function to make webassembly easier

    for (state, node) in dfa.iter_mut().enumerate() {
        for (next_digit, encoding) in base_encoding.iter().enumerate().take(base) {
            let r = (base * state  + next_digit) % divisor;
            node.insert(encoding.clone(), r);
        }
    }

    let order : Vec<usize> = if remainder == 0{
        (remainder..divisor).chain(0..remainder).rev().collect::<Vec<usize>>()
    } else { //behaviour is slightly different if the accepting state isn't 0
        (remainder + 1..divisor).chain(1..remainder).rev().chain([remainder, 0]).collect::<Vec<usize>>()
    };

    for (i, &state) in order.iter().enumerate() { // we proceed in "backwards" looping order down to the accepting state
        // collapse parallel edges
        for node in dfa.iter_mut() {
            let mut reduced: HashMap::<usize, Vec<String>> = HashMap::new(); // "reversed" HashMap to combine keys with the same value
            for (key, value) in &mut *node {
                if reduced.keys().any(|x| x == value){
                    reduced.get_mut(value).unwrap().push(key.clone());
                } else {
                    reduced.insert(*value, vec![key.clone()]);
                }
            }
            node.clear();
            for (key, value) in reduced {
                if value.len() == 1 {
                    node.insert(value[0].clone(), key);
                } else{
                    node.insert("(".to_string() + &value.join("|") + ")", key);
                }
            }
        }

        if state == remainder {
            continue;
        }

        // TODO: optimize regexes

        let mut recursive_string = if dfa.get(state).unwrap().values().any(|&x| x == state){
            "(".to_string() + &dfa.get(state)
                .unwrap()
                .iter()
                .filter(|(_, value)| **value == state)
                .map(|(key, _)| key.clone())
                .collect::<Vec<String>>()
                .join("|") + ")*"
        } else { String::new() };

        if recursive_string.len() == 4 {
            recursive_string = recursive_string[1..2].to_string() + "*";
        }

        for &other_state in order[i+1..].iter() {  // replace edges leading to the state to be removed
            let prefixes = dfa.get(other_state)
                            .unwrap()
                            .iter()
                            .filter(|(_, value)| **value == state)
                            .map(|(key, _)| key.clone())
                            .collect::<Vec<String>>(); // yields exactly all keys leading to state

            if ! prefixes.is_empty() {
                for k in &prefixes {
                    dfa.get_mut(other_state).unwrap().remove(k); // prune the edges to be replaced, now that their keys are saved
                }
                for (key, value) in dfa.get(state).unwrap().clone() {
                    if value != state {
                        for prefix in &prefixes {
                            let new_edge = prefix.clone() + &recursive_string + &key;
                            dfa.get_mut(other_state).unwrap().insert(new_edge, value);
                        }
                    }
                }
            }
        }
    }

    if remainder != 0 {
        let accepting_state = dfa.remove(remainder);
        let starting_state = dfa.remove(0);
        let mut accepting_recursion = String::new();
        let mut starting_recursion = String::new();
        let mut start_to_accept = String::new();
        let mut accept_to_start = String::new();

        for (key, value) in accepting_state {
            if value == remainder {
                accepting_recursion = key.clone();
            } else {
                accept_to_start = key.clone();
            }
        }
        for (key, value) in starting_state {
            if value == remainder {
                start_to_accept = key.clone();
            } else {
                starting_recursion = key.clone();
            }
        }
        return format!("^({starting_recursion}|{start_to_accept}({accepting_recursion})*{accept_to_start})*{start_to_accept}({accepting_recursion})*$"); 
    }
    "^".to_string()
     + &dfa.get(remainder)
        .unwrap()
        .keys()
        .cloned()
        .collect::<Vec<String>>()
        .join("|") + "*$"
}

#[cfg(test)]
mod tests {
    use regex::Regex;
    use rand::{seq::IteratorRandom, rng};
    use super::*;

    #[test]
    fn test_zero_remainder() {
        for divisor in 3..9 {
            for base in [2, 8, 10, 16] {
                let re = Regex::new(&mod_regex(divisor, base, 0)).unwrap();
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
                let re = Regex::new(&mod_regex(divisor, base, remainder)).unwrap();
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
