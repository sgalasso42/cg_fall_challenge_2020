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
    fn new(action: &str) -> Action {
        return Action {
            id: 0,
            action: action.to_string(),
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
    my_score: i32,
    opp_score: i32,
    ratio: f64,
    served: i8,
    inventory: [i8; 4],
    opp_inventory_score: i32,
    spells: Vec<Action>,
    book: Vec<Action>,
    orders: Vec<Action>
}

/* ------------------------------------------------------------ */
/* - Parsing -------------------------------------------------- */
/* ------------------------------------------------------------ */

fn get_turn_informations(turn: i32, served: i8) -> Game {
    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();
    let action_count = parse_input!(input_line, i32); // the number of spells and recipes in play
    let mut orders: Vec<Action> = Vec::new();
    let mut spells: Vec<Action> = Vec::new();
    let mut book: Vec<Action> = Vec::new();
    for i in 0..action_count as usize {
        let mut action: Action = Action::new("");
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
    let mut opp_inventory: [i8; 4] = [0, 0, 0, 0];
    let mut my_score: i32 = 0;
    let mut opp_score: i32 = 0;
    for i in 0..2 as usize {
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let inputs = input_line.split(" ").collect::<Vec<_>>();
        if i == 0 {
            inventory[0] = parse_input!(inputs[0], i8);
            inventory[1] = parse_input!(inputs[1], i8);
            inventory[2] = parse_input!(inputs[2], i8);
            inventory[3] = parse_input!(inputs[3], i8);
            my_score = parse_input!(inputs[4], i32); // amount of rupees
        } else {
            opp_inventory[0] = parse_input!(inputs[0], i8);
            opp_inventory[1] = parse_input!(inputs[1], i8);
            opp_inventory[2] = parse_input!(inputs[2], i8);
            opp_inventory[3] = parse_input!(inputs[3], i8);
            opp_score = parse_input!(inputs[4], i32); // amount of rupees
        }
    }
    return Game {
        turn: turn,
        my_score: my_score,
        opp_score: opp_score,
        ratio: 0.0,
        served: served,
        inventory: inventory,
        opp_inventory_score: inventory_final_score(opp_inventory),
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

fn inventory_final_score(state: [i8; 4]) -> i32 {
    return (state[1] + state[2] + state[3]) as i32;
}

fn get_available_brews(state: [i8; 4], orders: &Vec<Action>, game: &Game) -> Vec<Action> {
    let mut available_brews: Vec<Action> = orders.iter().filter(|order| !delta_add(state, order.delta).iter().any(|el| *el < 0)).cloned().collect::<Vec<Action>>();
    return available_brews.iter().filter(|brew| game.served < 5 || game.my_score + brew.price as i32 + inventory_final_score(delta_add(state, brew.delta)) >= game.opp_score + game.opp_inventory_score).cloned().collect::<Vec<Action>>();
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
            let mut delta: [i8; 4] = action.delta;
            if action.repeat > 1 {
                for el in delta.iter_mut() {
                    *el = *el / action.repeat;
                }
            }
            let spell_pos: usize = game_simulation.spells.iter().position(|spell| spell.delta == delta).unwrap() as usize;
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
            game_simulation.served += 1;
            // eprintln!("brew: {}", action.id);
            game_simulation.my_score += action.price as i32;
            let mut diff = (game.opp_score + game.opp_inventory_score) - (game.my_score + inventory_final_score(delta_add(state, action.delta)));
            // if diff < -20 { diff = -20 }
            let scaled_diff = (diff as f64 - (-138.0)) / (20.0 - (-138.0));
            game_simulation.ratio += action.price as f64 / (1.0 + ((depth as f64 - 1.0) / scaled_diff));
            game_simulation.ratio += action.price as f64 / depth as f64;
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
    // si current_ratio > best_ratio && current_served == 6 && my_score + inventory_final_score > opp_score + game.opp_inventory_score {
    // if game_simulation.ratio > solution.as_ref().unwrap().1 && game_simulation.served == 6 && game_simulation.my_score + inventory_final_score(state) >= game_simulation.opp_score + game_simulation.opp_inventory_score {
    //     *solution = Some((path.clone(), game_simulation.ratio));
    //     return -2;
    // }
    // si current_ratio > best_ratio || (best exist && current_ratio == best_ratio && best_action == CAST && current_action == LEARN)
    if game_simulation.ratio > solution.as_ref().unwrap().1 || (!solution.as_ref().unwrap().0.is_empty() && game_simulation.ratio == solution.as_ref().unwrap().1 && &(solution.as_ref().unwrap().0.first().unwrap().action)[..] == "CAST" && &(path.first().unwrap().action)[..] == "LEARN")  {
        *solution = Some((path.clone(), game_simulation.ratio));
        eprint!("[!] new best price: {:.3} path: ", game_simulation.ratio); for action in solution.as_ref().unwrap().0.iter() { eprint!("{}, ", action.id); } eprintln!("");
    }

    let f = path.len() /*+ heuristic()*/;
    if f > bound as usize {
        return f as i8;
    }
    let mut neighbors: Vec<Action> = [&get_available_spells(state, &game_simulation.spells)[..], &get_available_learns(state, &game_simulation.book)[..], &get_available_brews(state, &game_simulation.orders, game_simulation)[..]].concat();
    if game_simulation.spells.iter().any(|spell| spell.castable == 0) {
        neighbors.push(Action::new("REST"));
    }
    let mut min: i8 = std::i8::MAX;
    for action in neighbors.iter() {
        path.push(action.clone());
        if (game_simulation.turn == 1 && start_time.elapsed().as_millis() > 998) || (game_simulation.turn > 1 && start_time.elapsed().as_millis() > 48) {
            eprintln!("timeout at: {:.3?} bound: {} explored: {}", start_time.elapsed(), bound, explored_nodes);
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

fn find_best_action(game: &Game, registered_path: &(Vec<Action>, f64), game_forecast: &Option<Game>) -> ((Vec<Action>, f64), String) {
    let start_time: std::time::Instant = Instant::now();
    let mut solution: Option<(Vec<Action>, f64)> = Some((Vec::new(), game.ratio));
    let mut bound: i8 = 0;
    let mut comment: String = String::from("");
    loop {
        // eprintln!(">> new bound: {}", bound);
        let mut explored_nodes: usize = 0;
        let mut path: Vec<Action> = Vec::new();
        match graph_search(&mut solution, &mut path, game.inventory.clone(), bound, &mut game.clone(), &mut explored_nodes, start_time) {
            x if x < 0 => {
                // if x == -2 {
                //     comment = String::from("S");
                // }
                // eprintln!("# last explored: {} time: {:.3?}", explored_nodes, start_time.elapsed());
                break;
            },
            res => {
                // eprintln!("== explored: {} time: {:.3?}\n---------------", explored_nodes, start_time.elapsed());
                bound = res;
            }
        }
    }
    if game_forecast.is_some() && !registered_path.0.is_empty() { // If book and orders has not been altered (if there is a registered_path) && different opp score
        let concerned_learn: Vec<Action> = game_forecast.as_ref().unwrap().book.iter().filter(|learn| registered_path.0.contains(learn)).cloned().collect::<Vec<Action>>();
        let concerned_orders: Vec<Action> = game_forecast.as_ref().unwrap().orders.iter().filter(|order| registered_path.0.contains(order)).cloned().collect::<Vec<Action>>();
        if game.opp_score == game_forecast.as_ref().unwrap().opp_score && !concerned_learn.iter().any(|learn| learn != game.book.iter().find(|book_learn| book_learn.id == learn.id).unwrap_or(&Action::new(""))) && !concerned_orders.iter().any(|order| order != game.orders.iter().find(|book_order| book_order.id == order.id).unwrap_or(&Action::new(""))) {

            if solution.as_ref().unwrap().1 > 0.0 { // If found a path
                if game.turn < 6 && solution.as_ref().unwrap().1 < 8.0 {
                    return ((vec![game.book[0].clone()], 0.0), String::from("TS"));
                }
                // eprintln!("F");
                if solution.as_ref().unwrap().1 > registered_path.1 {
                    return (solution.as_ref().unwrap().clone(), format!("F{} ({}) {:.3}", comment, solution.as_ref().unwrap().0.len(), &solution.unwrap().1.to_string()));
                }
            }
            // eprintln!("C");
            return (registered_path.clone(), String::from("C")); // [!] TODO only check alteration of elements of path
        }
    }
    if solution.as_ref().unwrap().1 > 0.0 { // If found a path
        if game.turn < 6 && solution.as_ref().unwrap().1 < 8.0 {
            return ((vec![game.book[0].clone()], 0.0), String::from("TS"));
        }
        // eprintln!("F");
        return (solution.as_ref().unwrap().clone(), format!("F{} ({}) {:.3}", comment, solution.as_ref().unwrap().0.len(), &solution.unwrap().1.to_string()));
    }
    if game.inventory.iter().sum::<i8>() < 5 { // If not found a path and inventory is not filled enougth
        let available_fillers = game.spells.iter().filter(|spell| !spell.delta.iter().any(|el| *el < 0)).cloned().collect::<Vec<Action>>();
        // [!] TODO: accept aswell any sort if the result is better than current nb of elements
        if !available_fillers.is_empty() {
            let new_available_filters = available_fillers.iter().filter(|spell| spell.castable == 1).cloned().collect::<Vec<Action>>();
            if !new_available_filters.is_empty() {
                // eprintln!("TI");
                return ((vec![new_available_filters.iter().max_by_key(|spell| spell.delta.iter().sum::<i8>()).unwrap().clone()], 0.0), String::from("TI"));
            }
            // eprintln!("TR");
            return ((vec![Action::new("REST")], 0.0), String::from("TR"));
        }
    }
    // eprintln!("TL");
    return ((vec![game.book[0].clone()], 0.0), String::from("TL")); // Default, free learning
}

/* ------------------------------------------------------------ */
/* - Main ----------------------------------------------------- */
/* ------------------------------------------------------------ */

fn main() {
    let mut turn: i32 = 0;
    let mut served: i8 = 0;
    let mut registered_path: (Vec<Action>, f64) = (Vec::new(), 0.0);
    let mut game_forecast: Option<Game> = None;
    loop {
        turn += 1;
        let mut game: Game = get_turn_informations(turn, served);
        eprintln!("my: {} opp: {}", game.my_score, game.opp_score);
        let start_time = Instant::now();
        let mut res: ((Vec<Action>, f64), String) = find_best_action(&game, &registered_path, &game_forecast);
        // eprint!("best path: "); for ac in (res.0).0.iter() { eprint!("{}, ", ac.id); } eprintln!("");
        registered_path = res.0.clone();
        let action = registered_path.0.remove(0);
        // eprint!("registered_path: "); for ac in registered_path.0.iter() { eprint!("{}, ", ac.id); } eprintln!("");
        // eprintln!("simulating: {} {}", action.action, action.id);
        game_forecast = Some(simulate(&action, game.inventory, 0, &game).1);
        // eprint!("fc orders: "); for order in game_forecast.as_ref().unwrap().orders.iter() { eprint!("{}, ", order.id); } eprintln!("");
        // eprint!("fc book: "); for learn in game_forecast.as_ref().unwrap().book.iter() { eprint!("{}, ", learn.id); } eprintln!("");
        eprintln!("graph search duration: {:.3?}", start_time.elapsed());
        match &(action.action)[..] {
            "CAST" => {
                println!("{} {} {} {}", action.action, action.id, action.repeat, res.1);
            },
            "LEARN" => {
                println!("{} {} {}", action.action, action.id, res.1);
            },
            "REST" => {
                println!("{} {}", action.action, res.1);
            },
            "BREW" => {
                println!("BREW {}", action.id);
                served += 1;
            },
            _ => {}
        }
    }
}
