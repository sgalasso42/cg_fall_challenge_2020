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
}

#[derive(Clone, Debug)]
struct Game {
    turn: i32,
    score: f64,
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
    let mut score: f64 = 0.0;
    for i in 0..2 as usize {
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let inputs = input_line.split(" ").collect::<Vec<_>>();
        if i == 0 {
            inventory[0] = parse_input!(inputs[0], i8);
            inventory[1] = parse_input!(inputs[1], i8);
            inventory[2] = parse_input!(inputs[2], i8);
            inventory[3] = parse_input!(inputs[3], i8);
            score = parse_input!(inputs[4], f64); // amount of rupees
        }
    }
    return Game {
        turn: turn,
        score: 0.0,
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

fn get_available_brews(state: [i8; 4], orders: &Vec<Action>) -> Vec<Action> {
    return orders.iter().filter(|order| !delta_add(state, order.delta).iter().any(|el| *el < 0)).cloned().collect::<Vec<Action>>();
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
    for learn in book.iter() { // if enought for tax : take pocket content -> learn spell -> pay tax
        if !delta_add(state, [-learn.tax, 0, 0, 0]).iter().any(|el| *el < 0) {
            if delta_add(state, [learn.pocket, 0, 0, 0]).iter().sum::<i8>() <= 10 {
                possible_learn.push(learn.clone());
            }
        }
    }
    // eprint!("possible_learn: "); for el in possible_learn.iter() { eprint!("{}, ", el.id); } eprintln!("");
    return possible_learn;
}

/*fn heuristic() {

}*/

fn simulate(action: &Action, state: [i8; 4], depth: i32, game: &Game) -> ([i8; 4], Game) {
    let mut neighbour: [i8; 4] = state.clone();
    let mut game_simulation: Game = game.clone();
    match &action.action[..] {
        "CAST" => {
            let spell_pos: usize = game_simulation.spells.iter().position(|spell| spell.id == action.id).unwrap() as usize;
            game_simulation.spells[spell_pos].castable = 0;
            neighbour = delta_add(state, action.delta);
        },
        "LEARN" => {
            let new_state: [i8; 4] = delta_add(delta_add(state, [action.pocket, 0, 0, 0]), [-action.tax, 0, 0, 0]);
            let learn_index = &game_simulation.book.iter().position(|learn| learn.id == action.id).unwrap();
            for (i, learn) in game_simulation.book.iter_mut().enumerate() {
                if learn.tax > 0 && i > *learn_index {
                    learn.tax -= 1;
                } else if i < *learn_index {
                    learn.pocket += 1;
                }
            }
            game_simulation.book.retain(|spell| spell.id != action.id);
            let mut new_spell = action.clone();
            new_spell.action = String::from("CAST");
            new_spell.castable = 1;
            game_simulation.spells.push(new_spell);
            neighbour = new_state;
        },
        "BREW" => {
            // eprintln!("brew: {}", action.id);
            game_simulation.score += (action.price as f64 * (1.0 - (depth as f64 / 10.0)));
            let brew_index = game_simulation.orders.iter().position(|brew| brew.id == action.id).unwrap();
            if brew_index == 0 && game_simulation.orders.len() > 1 {
                game_simulation.orders[1].price += 2;
            } else if brew_index == 1 && game_simulation.orders.len() > 2 {
                game_simulation.orders[2].price += 1;
            }
            game_simulation.orders.retain(|order| order.id != action.id);
            neighbour = delta_add(state, action.delta);
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

fn graph_search(solution: &mut Option<(Vec<Action>, f64)>, path: &mut Vec<Action>, state: [i8; 4], bound: i8, game_simulation: &Game, explored_nodes: &mut usize, start_time: std::time::Instant) -> i8 {
    *explored_nodes += 1;
    /*|| (ratio == solution.as_ref().unwrap().1 && &(solution.as_ref().unwrap().0.first().unwrap().action)[..] == "CAST" && &(path.first().unwrap().action)[..] == "LEARN")*/
    if game_simulation.score > solution.as_ref().unwrap().1  {
        *solution = Some((path.clone(), game_simulation.score));
        // eprint!("[!] new best price: {} path: ", game_simulation.score); for action in solution.as_ref().unwrap().0.iter() { eprint!("{}, ", action.id); } eprintln!("");
    }
    let f = path.len() /*+ heuristic()*/;
    if f > bound as usize {
        return f as i8;
    }
    let mut neighbors: Vec<Action> = [&get_available_spells(state, &game_simulation.spells)[..], &get_available_learns(state, &game_simulation.book)[..], &get_available_brews(state, &game_simulation.orders)[..]].concat();
    if game_simulation.spells.iter().any(|spell| spell.castable == 0) {
        neighbors.push(Action::new(String::from("REST")));
    }
    let mut min: i8 = std::i8::MAX;
    for action in neighbors.iter() {
        path.push(action.clone());
        if (game_simulation.turn == 1 && start_time.elapsed().as_millis() > 998) || (game_simulation.turn > 1 && start_time.elapsed().as_millis() > 45) {
            eprintln!("timeout at: {:.3?}", start_time.elapsed());
            return -1;
        }
        let simulation: ([i8; 4], Game) = simulate(action, state, path.len() as i32, game_simulation);
        // if bound < 2 {
        //     eprintln!("depth: {} action: {} {} inventory: {:?}", path.len(), action.action, action.id, simulation.0);
        //     eprint!("spells: "); for spell in simulation.1.spells.iter() { eprint!("({}, {}), ", spell.id, spell.castable); } eprintln!("");
        //     eprint!("learns: "); for learn in simulation.1.book.iter() { eprint!("({}, {}), ", learn.id, learn.tax); } eprintln!("");
        //     eprint!("orders: "); for order in simulation.1.orders.iter() { eprint!("({}, {}), ", order.id, order.price); } eprintln!("");
        // }
        match graph_search(solution, path, simulation.0, bound, &simulation.1, explored_nodes, start_time) {
            -1 => return -1,
            res if res < min => {
                min = res;
            },
            _ => {}
        }
        path.pop();
    }
    return min;
}

fn find_best_action(game: &Game) -> (Action, String) {
    let start_time: std::time::Instant = Instant::now();
    let mut solution: Option<(Vec<Action>, f64)> = Some((Vec::new(), game.score));
    let mut bound: i8 = 0;
    loop {
        // eprintln!(">> new bound: {}", bound);
        let mut explored_nodes: usize = 0;
        let mut path: Vec<Action> = Vec::new();
        match graph_search(&mut solution, &mut path, game.inventory.clone(), bound, &mut game.clone(), &mut explored_nodes, start_time) {
            -1 => {
                // eprintln!("# last explored: {} time: {:.3?}", explored_nodes, start_time.elapsed());
                break;
            },
            res => {
                // eprintln!("== explored: {} time: {:.3?}\n---------------", explored_nodes, start_time.elapsed());
                bound = res;
            }
        }
    }
    if solution.as_ref().unwrap().1 > 0.0 {
        return (solution.as_ref().unwrap().0.first().unwrap().clone(), format!("F {:.3}", &solution.unwrap().1.to_string()));
    }
    if !game.inventory.iter().any(|el| *el > 0) {
        let available_fillers = game.spells.iter().filter(|spell| spell.castable == 1 && !spell.delta.iter().any(|el| *el < 0)).cloned().collect::<Vec<Action>>();
        if !available_fillers.is_empty() {
            return (available_fillers.iter().max_by_key(|spell| spell.delta.iter().sum::<i8>()).unwrap().clone(), String::from("TI"));
        }
    }
    return (game.book[0].clone(), String::from("TL"));
}

/* ------------------------------------------------------------ */
/* - Main ----------------------------------------------------- */
/* ------------------------------------------------------------ */

fn main() {
    let mut turn: i32 = 0;
    loop {
        turn += 1;
        let mut game: Game = get_turn_informations(turn);
        match &get_available_brews(game.inventory, &game.orders)[..] {
            [] => {
                let start_time = Instant::now();
                let action: (Action, String) = find_best_action(&game);
                eprintln!("graph search duration: {:.3?}", start_time.elapsed());
                match &(action.0.action)[..] {
                    "CAST" => {
                        println!("{} {} {} {} {}", action.0.action, action.0.id, action.0.repeat, action.1, if action.0.repeat > 1 { "R" } else { "" });
                    },
                    "LEARN" => {
                        println!("{} {} {}", action.0.action, action.0.id, action.1);
                    },
                    "REST" => {
                        println!("{} {}", action.0.action, action.1);
                    },
                    _ => {}
                }
            },
            orders => {
                let recipe_id: i32 = orders.iter().max_by_key(|order| order.price).unwrap().id;
                println!("BREW {}", recipe_id);
            }
        }
    }
}
