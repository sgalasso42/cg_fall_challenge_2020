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
    base_turn: i8,
    turn: i8,
    my_score: i8,
    opp_score: i8,
    ratio: f32,
    served: i8,
    inventory: [i8; 4],
    opp_inventory_score: i8,
    spells: Vec<Action>,
    book: Vec<Action>,
    orders: Vec<Action>
}

/* ------------------------------------------------------------ */
/* - Parsing -------------------------------------------------- */
/* ------------------------------------------------------------ */

fn get_turn_informations(turn: i8, served: i8) -> Game {
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
    let mut my_score: i8 = 0;
    let mut opp_score: i8 = 0;
    for i in 0..2 as usize {
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let inputs = input_line.split(" ").collect::<Vec<_>>();
        if i == 0 {
            inventory[0] = parse_input!(inputs[0], i8);
            inventory[1] = parse_input!(inputs[1], i8);
            inventory[2] = parse_input!(inputs[2], i8);
            inventory[3] = parse_input!(inputs[3], i8);
            my_score = parse_input!(inputs[4], i8); // amount of rupees
        } else {
            opp_inventory[0] = parse_input!(inputs[0], i8);
            opp_inventory[1] = parse_input!(inputs[1], i8);
            opp_inventory[2] = parse_input!(inputs[2], i8);
            opp_inventory[3] = parse_input!(inputs[3], i8);
            opp_score = parse_input!(inputs[4], i8); // amount of rupees
        }
    }
    return Game {
        base_turn: turn,
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

fn delta_add(a: &[i8; 4], b: &[i8; 4]) -> [i8; 4] {
    return [a[0] + b[0], a[1] + b[1], a[2] + b[2], a[3] + b[3]];
}

fn delta_mult(a: &[i8; 4], b: &[i8; 4]) -> [i8; 4] {
    return [a[0] * b[0], a[1] * b[1], a[2] * b[2], a[3] * b[3]];
}

fn inventory_final_score(state: [i8; 4]) -> i8 {
    return (state[1] + state[2] + state[3]) as i8;
}

fn get_available_brews(game: &Game) -> Vec<Action> {
    let mut available_brews: Vec<Action> = game.orders.iter().filter(|order| !delta_add(&game.inventory, &order.delta).iter().any(|el| *el < 0)).cloned().collect::<Vec<Action>>();
    return available_brews.iter().filter(|brew| game.served < 5 || game.my_score + brew.price + inventory_final_score(delta_add(&game.inventory, &brew.delta)) > game.opp_score + game.opp_inventory_score).cloned().collect::<Vec<Action>>();
}

fn get_available_spells(game: &Game) -> Vec<Action> {
    let mut possible_cast: Vec<Action> = Vec::new();
    for spell in game.spells.iter() {
        if spell.castable == 1 {
            let mut repeat_count: i8 = 0;
            loop {
                repeat_count += 1;
                let mut new_spell: Action = spell.clone();
                new_spell.delta = delta_mult(&spell.delta, &[repeat_count, repeat_count, repeat_count, repeat_count]);
                new_spell.repeat = repeat_count;
                let missing_table = delta_add(&game.inventory, &new_spell.delta);
                if !missing_table.iter().any(|el| *el < 0) && missing_table.iter().sum::<i8>() <= 10 {
                    possible_cast.push(new_spell);
                    if spell.repeatable == 0 { break; }
                } else { break; }
            }
        }
    }
    return possible_cast;
}

fn get_available_learns(game: &Game) -> Vec<Action> {
    let mut possible_learn: Vec<Action> = Vec::new();
    for learn in game.book.iter() { // if enought for tax : take pocket content -> learn spell -> pay tax
        if !delta_add(&game.inventory, &[-learn.tax, 0, 0, 0]).iter().any(|el| *el < 0) {
            if delta_add(&game.inventory, &[learn.pocket, 0, 0, 0]).iter().sum::<i8>() <= 10 {
                possible_learn.push(learn.clone());
            }
        }
    }
    return possible_learn;
}

fn get_neighbors(game: &Game) -> Vec<Action> {
    let mut neighbors: Vec<Action> = Vec::new();
    if game.turn < 100 && game.served < 6 {
        neighbors = [&get_available_spells(&game)[..], &get_available_learns(&game)[..], &get_available_brews(&game)[..]].concat();
        if game.spells.iter().any(|spell| spell.castable == 0) {
            neighbors.push(Action::new("REST"));
        }
    }
    return neighbors;
}

fn simulate(action: &Action, depth: i32, game: &Game) -> Game {
    let mut game_simulation: Game = game.clone();
    match &action.action[..] {
        "CAST" => {
            let mut delta: [i8; 4] = action.delta;
            if action.repeat > 1 {
                for el in delta.iter_mut() {
                    *el = *el / action.repeat;
                }
            }
            let spell_pos: usize = game_simulation.spells.iter().position(|spell| spell.delta == delta).expect("game_simulation.spells.iter().position(|spell| spell.delta == delta)") as usize;
            game_simulation.spells[spell_pos].castable = 0;
            game_simulation.inventory = delta_add(&game.inventory, &action.delta);
        },
        "LEARN" => {
            let new_s: [i8; 4] = delta_add(&delta_add(&game.inventory, &[action.pocket, 0, 0, 0]), &[-action.tax, 0, 0, 0]);
            let learn_index = &game_simulation.book.iter().position(|learn| learn.id == action.id).expect("&game_simulation.book.iter().position(|learn| learn.id == action.id)");
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
            game_simulation.inventory = new_s;
        },
        "BREW" => {
            game_simulation.served += 1;
            game_simulation.my_score += action.price;

            // let mut diff = (game.opp_score + game.opp_inventory_score) - (game.my_score + inventory_final_score(delta_add(&game.inventory, &action.delta)));
            // let scaled_diff = (diff as f32 - (-138.0)) / (20.0 - (-138.0));
            // game_simulation.ratio += action.price as f32 / (1.0 + ((depth as f32 - 1.0) / scaled_diff));
            // game_simulation.ratio += action.price as f32 / (1.0 + depth as f32) + delta_add(&game.inventory, &action.delta).iter().sum::<i8>() as f32;
            game_simulation.ratio += action.price as f32 - (2.0 * depth as f32) + delta_add(&game.inventory, &action.delta).iter().sum::<i8>() as f32;

            let brew_index = game_simulation.orders.iter().position(|brew| brew.id == action.id).expect("game_simulation.orders.iter().position(|brew| brew.id == action.id)");
            if brew_index == 0 && game_simulation.orders.len() > 1 {
                game_simulation.orders[1].price += 2;
            } else if brew_index == 1 && game_simulation.orders.len() > 2 {
                game_simulation.orders[2].price += 1;
            }
            game_simulation.orders.retain(|order| order.id != action.id);
            game_simulation.inventory = delta_add(&game.inventory, &action.delta);
        },
        "REST" => {
            for spell in game_simulation.spells.iter_mut() {
                spell.castable = 1;
            }
        },
        _ => {}
    };
    game_simulation.turn += 1;
    return game_simulation;
}

fn graph_search(solution: &mut Option<(Vec<Action>, f32)>, path: &mut Vec<Action>, bound: i8, game: &Game, explored_nodes: &mut usize, start_time: std::time::Instant) -> bool {
    *explored_nodes += 1;
    if game.ratio > solution.as_ref().unwrap().1 || (!solution.as_ref().unwrap().0.is_empty() && game.ratio == solution.as_ref().unwrap().1 && &(solution.as_ref().unwrap().0.first().unwrap().action)[..] == "CAST" && &(path.first().unwrap().action)[..] == "LEARN")  {
        *solution = Some((path.clone(), game.ratio));
        eprint!("[!] new best price: {:.3} path: ", game.ratio); for action in solution.as_ref().unwrap().0.iter() { eprint!("{}, ", action.id); } eprintln!("");
    }
    if path.len() > bound as usize {
        return true;
    }
    for action in get_neighbors(game).iter() {
        path.push(action.clone());
        if (game.base_turn == 1 && start_time.elapsed().as_millis() > 998) || (game.base_turn > 1 && start_time.elapsed().as_millis() > 48) {
            eprintln!("timeout at: {:.3?} bound: {} explored: {}", start_time.elapsed(), bound, explored_nodes);
            return false;
        }
        let simulation: Game = simulate(action, path.len() as i32, game);
        if !graph_search(solution, path,bound, &simulation, explored_nodes, start_time) {
            return false;
        }
        path.pop();
    }
    return true;
}

fn find_best_path(game: &Game, registered_path: &(Vec<Action>, f32), game_forecast: &Option<Game>, start_time: std::time::Instant) -> ((Vec<Action>, f32), String) {
    let mut solution: Option<(Vec<Action>, f32)> = Some((Vec::new(), game.ratio));
    let mut bound: i8 = 0;
    loop {
        let mut explored_nodes: usize = 0;
        let mut path: Vec<Action> = Vec::new();
        if !graph_search(&mut solution, &mut path, bound, &mut game.clone(), &mut explored_nodes, start_time) {
            break;
        }
        bound += 1;
    }
    if game_forecast.is_some() && !registered_path.0.is_empty() { // If book and orders has not been altered (if there is a registered_path) && different opp score
        let concerned_learn: Vec<Action> = game_forecast.as_ref().unwrap().book.iter().filter(|learn| registered_path.0.contains(learn)).cloned().collect::<Vec<Action>>();
        let concerned_orders: Vec<Action> = game_forecast.as_ref().unwrap().orders.iter().filter(|order| registered_path.0.contains(order)).cloned().collect::<Vec<Action>>();
        let is_opp_score_altered: bool = game.opp_score != game_forecast.as_ref().unwrap().opp_score;
        let is_concerned_learn_altered: bool = concerned_learn.iter().any(|learn| learn != game.book.iter().find(|book_learn| book_learn.id == learn.id).unwrap_or(&Action::new("")));
        let is_concerned_orders_altered: bool = concerned_orders.iter().any(|order| order != game.orders.iter().find(|book_order| book_order.id == order.id).unwrap_or(&Action::new("")));
        if !is_opp_score_altered && !is_concerned_learn_altered && !is_concerned_orders_altered && (solution.as_ref().unwrap().1 < 0.0 || solution.as_ref().unwrap().1 < registered_path.1) {
            return (registered_path.clone(), String::from("C"));
        }
    }
    if solution.as_ref().unwrap().1 > 0.0 { // Found a path
        if game.base_turn < 5 && solution.as_ref().unwrap().1 < 8.0 {
            return ((vec![game.book[0].clone()], 0.0), String::from("TS"));
        }
        return (solution.as_ref().unwrap().clone(), format!("F ({}) {:.3}", solution.as_ref().unwrap().0.len(), &solution.unwrap().1.to_string()));
    } else if game.inventory.iter().sum::<i8>() < 5 { // No path found and inventory is not filled enougth
        let possible_spells = game.spells.iter().filter(|spell| !delta_add(&spell.delta, &game.inventory).iter().any(|el| *el < 0)).cloned().collect::<Vec<Action>>();
        let available_fillers = possible_spells.iter().filter(|spell| delta_add(&spell.delta, &game.inventory).iter().sum::<i8>() > game.inventory.iter().sum::<i8>()).cloned().collect::<Vec<Action>>();
        if !available_fillers.is_empty() {
            let new_available_filters = available_fillers.iter().filter(|spell| spell.castable == 1).cloned().collect::<Vec<Action>>();
            if !new_available_filters.is_empty() {
                return ((vec![new_available_filters.iter().max_by_key(|spell| spell.delta.iter().sum::<i8>()).unwrap().clone()], 0.0), String::from("TI"));
            }
            return ((vec![Action::new("REST")], 0.0), String::from("TR"));
        }
    }
    if !game.book.is_empty() {
        return ((vec![game.book[0].clone()], 0.0), String::from("TL"));
    }
    return ((vec![Action::new("REST")], 0.0), String::from("Et merde !"));
}

/* ------------------------------------------------------------ */
/* - Main ----------------------------------------------------- */
/* ------------------------------------------------------------ */

fn main() {
    let mut turn: i8 = 0;
    let mut served: i8 = 0;
    let mut registered_path: (Vec<Action>, f32) = (Vec::new(), 0.0);
    let mut game_forecast: Option<Game> = None;
    loop {
        turn += 1;
        let mut game: Game = get_turn_informations(turn, served);
        let start_time = Instant::now();
        eprintln!("my: {} opp: {}", game.my_score, game.opp_score);
        let mut best_path: ((Vec<Action>, f32), String) = find_best_path(&game, &registered_path, &game_forecast, start_time);
        registered_path = best_path.0.clone();
        let action = registered_path.0.remove(0);
        game_forecast = Some(simulate(&action, 0, &game));
        eprintln!("graph search duration: {:.3?}", start_time.elapsed());
        eprintln!("action: {:?}", action);
        match &(action.action)[..] {
            "CAST" => {
                println!("CAST {} {} {}", action.id, action.repeat, best_path.1);
            },
            "LEARN" => {
                println!("LEARN {} {}", action.id, best_path.1);
            },
            "REST" => {
                println!("REST {}", best_path.1);
            },
            "BREW" => {
                println!("BREW {}", action.id);
                served += 1;
            },
            _ => {
                println!("REST [!] Who the fuck wrote this code [!]");
            }
        }
    }
}
