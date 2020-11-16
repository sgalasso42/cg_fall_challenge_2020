use std::io;
use std::time::{Instant};

macro_rules! parse_input {
    ($x:expr, $t:ident) => ($x.trim().parse::<$t>().unwrap())
}

#[derive(Clone, Debug, PartialEq)]
struct Action {
    id: i32,
    action: String,
    delta: [i32; 4],
    price: i32, // if order
    tax: i32, // if learn
    pocket: i32, // if learn, nb of type0 stored by tax
    repeatable: i32, // 1 if repeatable
    repeat: i32, // if cast, repeat instruction
    castable: i32 // 0 if castable cast
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
    fn from(id: i32, action: String, delta: [i32; 4]) -> Action {
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
    inventory: [i32; 4],
    spells: Vec<Action>,
    book: Vec<Action>,
    orders: Vec<Action>
}

/* ------------------------------------------------------------ */
/* - Parsing -------------------------------------------------- */
/* ------------------------------------------------------------ */

fn get_turn_informations() -> Game {
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
        action.delta[0] = parse_input!(inputs[2], i32); // tier-0 ingredient change
        action.delta[1]= parse_input!(inputs[3], i32); // tier-1 ingredient change
        action.delta[2] = parse_input!(inputs[4], i32); // tier-2 ingredient change
        action.delta[3] = parse_input!(inputs[5], i32); // tier-3 ingredient change
        action.price = parse_input!(inputs[6], i32); // the price in rupees if this is a potion
        action.tax = parse_input!(inputs[7], i32); // the index in the tome if this is a tome spell, equal to the read-ahead tax
        action.pocket = parse_input!(inputs[8], i32); // the amount of taxed tier-0 ingredients you gain from learning this spell
        action.castable = parse_input!(inputs[9], i32); // 1 if this is a castable player spell
        action.repeatable = parse_input!(inputs[10], i32); // 1 if this is a repeatable player spell
        match &(action.action)[..] {
            "BREW" => { orders.push(action); },
            "CAST" => { spells.push(action); },
            "LEARN" => { book.push(action); },
            _ => {}
        }
    }
    let mut inventory: [i32; 4] = [0, 0, 0, 0];
    for i in 0..2 as usize {
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let inputs = input_line.split(" ").collect::<Vec<_>>();
        if i == 0 {
            inventory[0] = parse_input!(inputs[0], i32);
            inventory[1] = parse_input!(inputs[1], i32);
            inventory[2] = parse_input!(inputs[2], i32);
            inventory[3] = parse_input!(inputs[3], i32);
        }
        let score = parse_input!(inputs[4], i32); // amount of rupees
    }
    return Game {
        inventory: inventory,
        spells: spells,
        book: book,
        orders: orders
    };
}

/* ------------------------------------------------------------ */
/* - Functions ------------------------------------------------ */
/* ------------------------------------------------------------ */

fn delta_add(a: [i32; 4], b: [i32; 4]) -> [i32; 4] {
    return [a[0] + b[0], a[1] + b[1], a[2] + b[2], a[3] + b[3]];
}

fn delta_mult(a: [i32; 4], b: [i32; 4]) -> [i32; 4] {
    return [a[0] * b[0], a[1] * b[1], a[2] * b[2], a[3] * b[3]];
}

fn find_solutions(state: [i32; 4], game: &Game) -> Vec<Action> {
    let mut possible_recipe: Vec<Action> = Vec::new();
    for order in game.orders.iter() {
        let missing_table = delta_add(state, order.delta);
        if !missing_table.iter().any(|el| *el < 0) {
            possible_recipe.push(order.clone());
        }
    }
    return possible_recipe;
}

fn get_available_spells(state: [i32; 4], spells: &Vec<Action>) -> Vec<Action> {
    let mut possible_cast: Vec<Action> = Vec::new();
    for spell in spells.iter() {
        if spell.castable == 1 {
            let mut repeat_count: i32 = 0;
            loop {
                repeat_count += 1;
                let mut new_spell: Action = spell.clone();
                new_spell.delta = delta_mult(spell.delta, [repeat_count, repeat_count, repeat_count, repeat_count]);
                new_spell.repeat = repeat_count;
                let missing_table = delta_add(state, new_spell.delta);
                // eprintln!("mt: {:?}", missing_table);
                let mut sum: i32 = 0;
                for el in missing_table.iter() {
                    sum += *el;
                }
                if sum <= 10 && !missing_table.iter().any(|el| *el < 0) {
                    possible_cast.push(new_spell);
                    if spell.repeatable == 0 { break; }
                } else { break; }
            }
        }
    }
    // eprintln!("possible_cast: {:?}", possible_cast);
    return possible_cast;
}

fn get_available_learns(state: [i32; 4], book: &Vec<Action>) -> Vec<Action> {
    let mut possible_learn: Vec<Action> = Vec::new();
    for learn in book.iter() { // take pocket content -> learn spell -> pay tax
        let simulated_state: [i32; 4] = delta_add(state, [learn.pocket, 0, 0, 0]);
        let mut sum: i32 = 0;
        for el in simulated_state.iter() {
            sum += *el;
        }
        if sum <= 10 {
            let missing_table: [i32; 4] = delta_add(simulated_state, [-learn.tax, 0, 0, 0]);
            if !missing_table.iter().any(|el| *el < 0) {
                possible_learn.push(learn.clone());
            }
        }
    }
    // eprintln!("possible_learn: {:?}", possible_learn);
    return possible_learn;
}

/*fn heuristic() {

}*/

fn simulate(action: &Action, state: [i32; 4], game: &Game) -> ([i32; 4], Game) {
    let mut neighbour: [i32; 4] = state.clone();
    let mut game_simulation: Game = game.clone();
    match &action.action[..] {
        "CAST" => {
            let spell_pos: usize = game_simulation.spells.iter().position(|spell| spell.id == action.id).unwrap() as usize;
            game_simulation.spells[spell_pos].castable = 0;
            neighbour = delta_add(state, action.delta);
        },
        "LEARN" => { // take pocket content -> learn spell -> pay tax
            let mut new_state: [i32; 4] = delta_add(state, [action.pocket, 0, 0, 0]);
            new_state = delta_add(new_state, [-action.tax, 0, 0, 0]);
            game_simulation.book.retain(|spell| spell.id != action.id);
            game_simulation.spells.push(action.clone());
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

fn graph_search(state: [i32; 4], cost: i32, bound: i32, game_simulation: &Game, explored_nodes: &mut i32, start_time: std::time::Instant) -> Result<i32, i32> {
    if start_time.elapsed().as_millis() > 48 {
        return Err(-1);
    }
    *explored_nodes += 1;
    let f = cost /*+ heuristic()*/;
    let solutions = find_solutions(state, game_simulation);
    if !solutions.is_empty() {
        // eprint!("found lvl: {} for : ", cost); for el in solutions.iter() { eprint!("{} ({}), ", el.id, el.price); } eprintln!("");
        return Err(solutions.iter().max_by_key(|solution| solution.price).unwrap().price);
    } else if f > bound {
        return Ok(f);
    }
    let mut neighbors: Vec<Action> = [&get_available_spells(state, &game_simulation.spells)[..], &get_available_learns(state, &game_simulation.book)[..]].concat();
    if game_simulation.spells.iter().any(|spell| spell.castable == 0) {
        neighbors.push(Action::new(String::from("REST")));
    }
    // eprint!("neighbors: "); for el in neighbors.iter() { eprint!("{}, ", el.id); } eprintln!("");
    let mut min: i32 = std::i32::MAX;
    for action in neighbors.iter() {
        let simulation: ([i32; 4], Game) = simulate(action, state, game_simulation);
        match graph_search(simulation.0, cost + 1, bound, &simulation.1, explored_nodes, start_time) {
            Err(res) => return Err(res),
            Ok(res) => { min = res; }
        }
    }
    return Ok(min);
}

fn find_best_action(game: &Game) -> (Action, String) {
    let start_time: std::time::Instant = Instant::now();
    let mut bound: i32 = 0;
    loop {
        let mut explored_nodes: i32 = 0;
        let state: [i32; 4] = game.inventory.clone();
        let mut neighbors: Vec<Action> = [&get_available_spells(state, &game.spells)[..], &get_available_learns(state, &game.book)[..]].concat();
        if game.spells.iter().any(|spell| spell.castable == 0) {
            neighbors.push(Action::new(String::from("REST")));
        }
        let mut min: i32 = std::i32::MAX;
        let mut solution: Option<Action> = None;
        let mut first_action: Action = Action::new(String::from("CAST"));
        for action in neighbors.iter() {
            // eprintln!("try action: {}", action.id);
            let simulation: ([i32; 4], Game) = simulate(action, state, game);
            match graph_search(simulation.0, 1, bound, &simulation.1, &mut explored_nodes, start_time) {
                Err(res) => {
                    if res < 0 {
                        return (game.book[0].clone(), String::from("T"));
                    } else {
                        solution = Some(action.clone());
                    }
                },
                Ok(res) => { min = res; }
            }
        }
        eprintln!("# bound: {} explored: {} time: {:.3?}", bound, explored_nodes, start_time.elapsed());
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
    loop {
        let mut game: Game = get_turn_informations();
        let possible_recipe: Vec<Action> = find_solutions(game.inventory, &game);
        if possible_recipe.is_empty() {
            let start_time = Instant::now();
            let action: (Action, String) = find_best_action(&game);
            eprintln!("graph search duration: {:.3?}", start_time.elapsed());
            println!("{} {} {} {} {}", action.0.action, action.0.id, action.0.repeat, action.1, if action.0.repeat > 1 { "R" } else { "" });
        } else {
            let recipe_id: i32 = possible_recipe.iter().max_by_key(|order| order.price).unwrap().id;
            println!("BREW {}", recipe_id);
        }
    }
}
