use std::io;
use std::time::{Instant};

macro_rules! parse_input {
    ($x:expr, $t:ident) => ($x.trim().parse::<$t>().unwrap())
}

#[derive(Clone, Debug, PartialEq)]
struct Action {
    id: i32,
    action: String,
    delta: [i8; 4],
    price: i8, // if order
    tax: i8, // if learn
    pocket: i8, // if learn, nb of type0 stored by tax
    repeatable: i8, // 1 if repeatable
    repeat: i8, // if cast, repeat instruction
    castable: i8 // 0 if castable cast
}

impl Action {
    fn new(action: String) -> Action {
        return Action {
            id: 0,
            action: action,
            delta: [0, 0, 0, 0],
            price: 0,
            tax: 0,
            pocket: 0,
            repeatable: 0,
            repeat: 1,
            castable: 0
        }
    }
    fn from(id: i32, action: String, delta: [i8; 4]) -> Action {
        return Action {
            id: id,
            action: action,
            delta: delta,
            price: 0,
            tax: 0,
            pocket: 0,
            repeatable: 0,
            repeat: 1,
            castable: 0
        }
    }
}

#[derive(Clone, Debug)]
struct Game {
    turn: i32,
    inventory: [i8; 4],
    spells: Vec<Action>,
    book: Vec<Action>,
    orders: Vec<Action>
}

/* ------------------------------------------------------------ */
/* - Parsing -------------------------------------------------- */
/* ------------------------------------------------------------ */

fn get_turn_informations(turn: i32) -> Game {
    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();
    let action_count = parse_input!(input_line, i32); // the number of spells and recipes in play
    let mut orders: Vec<Action> = Vec::new();
    let mut spells: Vec<Action> = Vec::new();
    let mut book: Vec<Action> = Vec::new();
    for i in 0..action_count as usize {
        let mut action: Action = Action::new(String::from(""));
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let inputs = input_line.split(" ").collect::<Vec<_>>();
        action.id = parse_input!(inputs[0], i32); // the unique ID of this spell or recipe
        action.action = inputs[1].trim().to_string(); // later: CAST, OPPONENT_CAST, LEARN, BREW
        action.delta[0] = parse_input!(inputs[2], i8); // tier-0 ingredient change
        action.delta[1]= parse_input!(inputs[3], i8); // tier-1 ingredient change
        action.delta[2] = parse_input!(inputs[4], i8); // tier-2 ingredient change
        action.delta[3] = parse_input!(inputs[5], i8); // tier-3 ingredient change
        action.price = parse_input!(inputs[6], i8); // the price in rupees if this is a potion
        action.tax = parse_input!(inputs[7], i8); // the index in the tome if this is a tome spell, equal to the read-ahead tax
        action.pocket = parse_input!(inputs[8], i8); // the amount of taxed tier-0 ingredients you gain from learning this spell
        action.castable = parse_input!(inputs[9], i8); // 1 if this is a castable player spell
        action.repeatable = parse_input!(inputs[10], i8); // 1 if this is a repeatable player spell
        match &(action.action)[..] {
            "BREW" => { orders.push(action); },
            "CAST" => { spells.push(action); },
            "LEARN" => { book.push(action); },
            _ => {}
        }
    }
    let mut inventory: [i8; 4] = [0, 0, 0, 0];
    for i in 0..2 as usize {
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let inputs = input_line.split(" ").collect::<Vec<_>>();
        if i == 0 {
            inventory[0] = parse_input!(inputs[0], i8);
            inventory[1] = parse_input!(inputs[1], i8);
            inventory[2] = parse_input!(inputs[2], i8);
            inventory[3] = parse_input!(inputs[3], i8);
        }
        let score = parse_input!(inputs[4], i32); // amount of rupees
    }
    return Game {
        turn: turn,
        inventory: inventory,
        spells: spells,
        book: book,
        orders: orders
    };
}

/* ------------------------------------------------------------ */
/* - Functions ------------------------------------------------ */
/* ------------------------------------------------------------ */

fn delta_add(a: [i8; 4], b: [i8; 4]) -> [i8; 4] {
    return [a[0] + b[0], a[1] + b[1], a[2] + b[2], a[3] + b[3]];
}

fn delta_mult(a: [i8; 4], b: [i8; 4]) -> [i8; 4] {
    return [a[0] * b[0], a[1] * b[1], a[2] * b[2], a[3] * b[3]];
}

fn find_solutions(state: [i8; 4], game: &Game) -> Vec<&Action> {
    return game.orders.iter().filter(|order| !delta_add(state, order.delta).iter().any(|el| *el < 0)).collect::<Vec<&Action>>();
}

fn get_available_spells(state: [i8; 4], spells: &Vec<Action>) -> Vec<Action> {
    let mut possible_cast: Vec<Action> = Vec::new();
    for spell in spells.iter() {
        if spell.castable == 1 {
            let mut repeat_count: i8 = 0;
            loop {
                repeat_count += 1;
                let mut new_spell: Action = spell.clone();
                new_spell.delta = delta_mult(spell.delta, [repeat_count, repeat_count, repeat_count, repeat_count]);
                new_spell.repeat = repeat_count;
                let missing_table = delta_add(state, new_spell.delta);
                // eprintln!("mt: {:?}", missing_table);
                if !missing_table.iter().any(|el| *el < 0) && missing_table.iter().sum::<i8>() <= 10 {
                    possible_cast.push(new_spell);
                    if spell.repeatable == 0 { break; }
                } else { break; }
            }
        }
    }
    // eprint!("possible_cast: "); for el in possible_cast.iter() { eprint!("{}, ", el.id); } eprintln!("");
    return possible_cast;
}

fn get_available_learns(state: [i8; 4], book: &Vec<Action>) -> Vec<Action> {
    let mut possible_learn: Vec<Action> = Vec::new();
    for learn in book.iter() { // take pocket content -> learn spell -> pay tax
        let simulated_state: [i8; 4] = delta_add(state, [learn.pocket, 0, 0, 0]);
        if simulated_state.iter().sum::<i8>() <= 10 {
            let missing_table: [i8; 4] = delta_add(simulated_state, [-learn.tax, 0, 0, 0]);
            if !missing_table.iter().any(|el| *el < 0) {
                possible_learn.push(learn.clone());
            }
        }
    }
    // eprint!("possible_learn: "); for el in possible_learn.iter() { eprint!("{}, ", el.id); } eprintln!("");
    return possible_learn;
}

/*fn heuristic() {

}*/

fn simulate(action: &Action, state: [i8; 4], game: &Game) -> ([i8; 4], Game) {
    let mut neighbour: [i8; 4] = state.clone();
    let mut game_simulation: Game = game.clone();
    match &action.action[..] {
        "CAST" => {
            let spell_pos: usize = game_simulation.spells.iter().position(|spell| spell.id == action.id).unwrap() as usize;
            game_simulation.spells[spell_pos].castable = 0;
            neighbour = delta_add(state, action.delta);
        },
        "LEARN" => { // take pocket content -> learn spell -> pay tax
            let new_state: [i8; 4] = delta_add(delta_add(state, [action.pocket, 0, 0, 0]), [-action.tax, 0, 0, 0]);
            game_simulation.book.retain(|spell| spell.id != action.id);
            for learn in game_simulation.book.iter_mut() {
                learn.tax -= 1;
            }
            let mut new_spell = action.clone();
            new_spell.action = String::from("CAST");
            new_spell.castable = 1;
            game_simulation.spells.push(new_spell);
            neighbour = new_state;
        },
        "REST" => {
            for spell in game_simulation.spells.iter_mut() {
                spell.castable = 1;
            }
        },
        _ => {}
    };
    return (neighbour, game_simulation)
}

fn graph_search(/*debug_path: &mut Vec<Action>, */state: [i8; 4], cost: i32, bound: i32, game_simulation: &Game, explored_nodes: &mut i32, start_time: std::time::Instant) -> Result<i32, i8> {
    *explored_nodes += 1;
    let f = cost /*+ heuristic()*/;
    match &find_solutions(state, game_simulation)[..] {
        solutions if !solutions.is_empty() => {
            // eprint!("[!] found depth: {} for: ", cost - 1); for el in solutions.iter() { eprint!("{} price: {}, ", el.id, el.price); } eprintln!("");
            // eprint!("path: "); for el in debug_path.iter() { eprint!("{}, ", el.id); } eprintln!("");
            return Err(solutions.iter().max_by_key(|solution| solution.price).unwrap().price);
        },
        solutions if f > bound => { return Ok(f); },
        _ => {}
    }
    let mut neighbors: Vec<Action> = [&get_available_spells(state, &game_simulation.spells)[..], &get_available_learns(state, &game_simulation.book)[..]].concat();
    if game_simulation.spells.iter().any(|spell| spell.castable == 0) {
        neighbors.push(Action::new(String::from("REST")));
    }
    let mut min: i32 = std::i32::MAX;
    for action in neighbors.iter() {
        // debug_path.push(action.clone());
        if (game_simulation.turn == 1 && start_time.elapsed().as_millis() > 998) || (game_simulation.turn > 1 && start_time.elapsed().as_millis() > 48) {
            // eprintln!("timeout at: {:.3?}", start_time.elapsed());
            return Err(-1);
        }
        let simulation: ([i8; 4], Game) = simulate(action, state, game_simulation);
        match graph_search(/*debug_path, */simulation.0, cost + 1, bound, &simulation.1, explored_nodes, start_time) {
            Err(res) => return Err(res),
            Ok(res) if res < min => {
                min = res;
            },
            _ => {}
        }
        // debug_path.pop();
    }
    return Ok(min);
}

fn find_best_action(game: &Game) -> (Action, String) {
    let start_time: std::time::Instant = Instant::now();
    let mut bound: i32 = 0;
    loop {
        // eprintln!(">> new bound: {}", bound);
        let mut explored_nodes: i32 = 0;
        let state: [i8; 4] = game.inventory.clone();
        let mut neighbors: Vec<Action> = [&get_available_spells(state, &game.spells)[..], &get_available_learns(state, &game.book)[..]].concat();
        if game.spells.iter().any(|spell| spell.castable == 0) {
            neighbors.push(Action::new(String::from("REST")));
        }
        let mut min: i32 = std::i32::MAX;
        let mut best_score: i8 = 0;
        let mut solution: Option<Action> = None;
        for action in neighbors.iter() {
            // let mut debug_path: Vec<Action> = Vec::new();
            // debug_path.push(action.clone());
            let simulation: ([i8; 4], Game) = simulate(action, state, game);
            match graph_search(/*&mut debug_path, */simulation.0, 1, bound, &simulation.1, &mut explored_nodes, start_time) {
                Err(res) => {
                    if res < 0 {
                        if solution.is_some() {
                            break ;
                        }
                        // eprintln!("# last explored: {} time: {:.3?}", explored_nodes, start_time.elapsed());
                        return (game.book[0].clone(), String::from("T"));
                    } else if res == best_score && &(solution.as_ref().unwrap().action)[..] == "CAST" && &(action.action)[..] == "LEARN" {
                        solution = Some(action.clone());
                    } else if res > best_score {
                        best_score = res;
                        solution = Some(action.clone());
                    }
                },
                Ok(res) => { min = res; }
            }
            /*debug_path.pop();*/
        }
        // eprintln!("== explored: {} time: {:.3?}\n---------------", explored_nodes, start_time.elapsed());
        if solution.is_some() {
            return (solution.unwrap(), String::from("F"));
        }
        bound = min;
    }
    return (game.book[0].clone(), String::from("N"));
}

/* ------------------------------------------------------------ */
/* - Main ----------------------------------------------------- */
/* ------------------------------------------------------------ */

fn main() {
    let mut turn: i32 = 0;
    loop {
        turn += 1;
        let mut game: Game = get_turn_informations(turn);
        match &find_solutions(game.inventory, &game)[..] {
            orders if orders.is_empty() => {
                let start_time = Instant::now();
                let action: (Action, String) = find_best_action(&game);
                // eprintln!("graph search duration: {:.3?}", start_time.elapsed());
                println!("{} {} {} {} {}", action.0.action, action.0.id, action.0.repeat, action.1, if action.0.repeat > 1 { "R" } else { "" });
            },
            orders => {
                let recipe_id: i32 = orders.iter().max_by_key(|order| order.price).unwrap().id;
                println!("BREW {}", recipe_id);
            }
        }
    }
}

