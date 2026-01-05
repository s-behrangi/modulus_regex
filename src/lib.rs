use wasm_bindgen::prelude::wasm_bindgen;
use std::collections::HashMap;

#[wasm_bindgen]
pub fn mod_regex(divisor : usize, base : usize, remainder: usize) -> String {
    /* We stuff everything in one function to make WebAssembly easier */
    
    /* Handle a degenerate case: */
    if base == 0 {
        if remainder == 0 {
            return format!("^(0{{{divisor}}})*$");
        } else {
            return format!("^0{{{remainder}}}(0{{{divisor}}})*$");
        }
    }
    
    /* Initialize a DFA to reduce by elimination
    Invariant: a state's index in the DFA vector corresponds
    to the remainder it represents. Later on, "removing" a 
    state just curresponds to rearranging edges so none from
    the remaining states reach the state being removed */

    let base_encoding = "0123456789abcdef";
    let mut dfa: Vec<HashMap::<String, usize>> = (0..divisor).map(|_| HashMap::<String, usize>::new()).collect();

    for (state, node) in dfa.iter_mut().enumerate() {
        for digit in 0..base {
            let r = (base * state  + digit) % divisor;
            node.insert(base_encoding[digit..digit+1].to_string(), r);
        }
    }

    // pseudocode idea before i forget:
    // order is maybe significant
    // propagate from state 0 outward
    // find minimal distance from 0 to each state
    // order should put furthest first
    // OR: those with fewest edges (lowest degree) firsta

    let order : Vec<usize> = if remainder == 0{
        (remainder..divisor).rev().collect::<Vec<usize>>()
        //(1..divisor).chain(0..1).collect::<Vec<usize>>()
    } else { //behaviour is slightly different if the accepting state isn't 0
        (remainder + 1..divisor).chain(1..remainder).rev().chain([remainder, 0]).collect::<Vec<usize>>()
    };

    for (i, &state) in order.iter().enumerate() { // we proceed in "backwards" looping order down to the accepting state
        /* Collapse parallel edges by merging them with disjunction */
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
            continue; //necessary for remainder != 0 
        }

        /* Get the transitions from the state to itself */
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

        /* Excise a state by replacing all edges leading to it */
        for &other_state in order[i+1..].iter() {
            let prefixes = dfa.get(other_state)
                            .unwrap()
                            .iter()
                            .filter(|(_, value)| **value == state)
                            .map(|(key, _)| key.clone())
                            .collect::<Vec<String>>(); // yields exactly all keys leading from other_state to state

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
        /* This section is bulky because HashMaps are designed for
        the opposite operation, but it is mechanically straighforward */
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
        .join("|") + "*$" //note that because of the parallel merge stage, this won't need any additional parentheses
}

/* The version of the function below is abbreviated
because it uses nested vectors instead of HashMaps,
which simplifies certain operations. However, it 
produces bulkier regex, so I'm not using it. I'm 
not certain why this is the case, because the two 
functions should be algorithmically equivalent. I 
think it's possible that the HashMaps are merging keys
and saving space that way, because the effect doesn't
appear until larger regular expressions, i.e. it is
not a mere difference of redundant parentheses or 
something like that. */

pub fn _mod_regex(divisor : usize, base : usize, remainder : usize) -> String {
    let encoding = "0123456789abcdef";
    let mut dfa = vec![vec![String::new(); divisor]; divisor];
    for (r, state) in dfa.iter_mut().enumerate() {
        for digit in 0..base {
            let r_next = (base * r + digit) % divisor;
            state[r_next] = if state[r_next].is_empty() {
                encoding[digit..digit + 1].to_string()
            } else {
                state[r_next].clone() + "|" + &encoding[digit..digit + 1]
            };
        } 
    }
    
    let order : Vec<usize> = if remainder == 0{
        (0..divisor).rev().collect::<Vec<usize>>()
    } else { //behaviour is slightly different if the accepting state isn't 0
        (remainder + 1..divisor).chain(1..remainder).rev().chain([remainder, 0]).collect::<Vec<usize>>()
    };

    for (i, &state) in order.iter().enumerate() {
        if state == remainder || state == 0{
            continue;
        }

        let recursive_transition = if dfa[state][state].is_empty() {
            String::new()
        } else if dfa[state][state].len() == 1 || is_bracketed_unit(&dfa[state][state]){
                dfa[state][state].clone() + "*"
        } else {
            "(".to_string() + &dfa[state][state] + ")*"
        };

        for &start_state in order[i + 1..].iter(){
            
            if ! dfa[start_state][state].is_empty() {
                let start_to_mid = if !is_unbracketed_disjunction(&dfa[start_state][state]){//dfa[start_state][state].contains(&"|") {
                    dfa[start_state][state].clone()
                } else {
                    "(".to_string() + &dfa[start_state][state] + ")"
                };

                for &end_state in order[i + 1..].iter() {//0..divisor { //to order[i+1..]?
                    if ! dfa[state][end_state].is_empty() {
                        let mid_to_end = if !is_unbracketed_disjunction(&dfa[state][end_state]){//dfa[state][end_state].contains(&"|") {
                            dfa[state][end_state].clone()
                        } else {
                            "(".to_string() + &dfa[state][end_state] + ")"
                        };

                        if dfa[start_state][end_state].is_empty() {
                            dfa[start_state][end_state] = start_to_mid.clone() + &recursive_transition + &mid_to_end; //dfa[start_state][state].clone() + &recursive_transition + &dfa[state][end_state];
                        } else {
                            let start_to_end = if !is_unbracketed_disjunction(&dfa[start_state][end_state]){//dfa[start_state][end_state].contains(&"|") {
                                dfa[start_state][end_state].clone()
                            } else {
                                "(".to_string() + &dfa[start_state][end_state] + ")"
                            };

                            dfa[start_state][end_state] = start_to_end + "|" + &start_to_mid + &recursive_transition + &mid_to_end; //dfa[start_state][end_state].clone() + "|" + &dfa[start_state][state] + &recursive_transition + &dfa[state][end_state];
                        }
                    }
                }

                dfa[start_state][state] = String::new();
            }

        }
    }

    if remainder != 0 {
        let (a, b, c, d) = (dfa[0][0].clone(), dfa[0][remainder].clone(), dfa[remainder][0].clone(), dfa[remainder][remainder].clone());
        return format!("^(({a})|(({b})({d})*({c})))*({b})({d})*$");
    }

    "^(".to_string() + &dfa[0][0] + ")*$"
}

fn is_unbracketed_disjunction(word: &str) -> bool {
    word.contains('|') && ! is_bracketed_unit(word)
}

fn is_bracketed_unit(word: &str) -> bool {
    /* Assumes string is well-formed! */
    let is_bracketed = word.starts_with('(') && word.ends_with(')');
    let is_unit = ! word[..word.len()-1].chars()
        .scan(0, |acc, c| {
            match c {
                '(' => {*acc += 1; Some(*acc)},
                ')' => {*acc -=1; Some(*acc)},
                _ => Some(*acc)
            }
        })
        .any(|x| x == 0);
        is_bracketed && is_unit
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
                for n in 0..10000 {
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
