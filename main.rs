use std::io;

macro_rules! parse_input {
    ($x:expr, $t:ident) => ($x.trim().parse::<$t>().unwrap())
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct Action {
    id: i32,
    delta: [i32; 4],
    price: i32,
    used: i32
}

impl Action {
    fn new() -> Action {
        return Action {
            id: 0,
            delta: [0, 0, 0, 0],
            price: 0, // if this is an order
            used: 0 // if this is a cast
        }
    }
}

struct Game {
    inventory: [i32; 4],
    spells: Vec<Action>,
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
    for i in 0..action_count as usize {
        let mut action: Action = Action::new();
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let inputs = input_line.split(" ").collect::<Vec<_>>();
        action.id = parse_input!(inputs[0], i32); // the unique ID of this spell or recipe
        let action_type = inputs[1].trim().to_string(); // in the first league: BREW; later: CAST, OPPONENT_CAST, LEARN, BREW
        action.delta[0] = parse_input!(inputs[2], i32); // tier-0 ingredient change
        action.delta[1]= parse_input!(inputs[3], i32); // tier-1 ingredient change
        action.delta[2] = parse_input!(inputs[4], i32); // tier-2 ingredient change
        action.delta[3] = parse_input!(inputs[5], i32); // tier-3 ingredient change
        action.price = parse_input!(inputs[6], i32); // the price in rupees if this is a potion
        let tome_index = parse_input!(inputs[7], i32); // in the first two leagues: always 0; later: the index in the tome if this is a tome spell, equal to the read-ahead tax
        let tax_count = parse_input!(inputs[8], i32); // in the first two leagues: always 0; later: the amount of taxed tier-0 ingredients you gain from learning this spell
        action.used = parse_input!(inputs[9], i32); // in the first league: always 0; later: 1 if this is a castable player spell
        let repeatable = parse_input!(inputs[10], i32); // for the first two leagues: always 0; later: 1 if this is a repeatable player spell
        if action_type == "BREW" {
            orders.push(action);
        } else if action_type == "CAST" {
            spells.push(action);
        }
    }
    let mut inventory: [i32; 4] = [0, 0, 0, 0];
    for i in 0..2 as usize {
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let inputs = input_line.split(" ").collect::<Vec<_>>();
        if i == 0 {
            inventory[0] = parse_input!(inputs[0], i32); // tier-0 ingredients in inventory
            inventory[1] = parse_input!(inputs[1], i32);
            inventory[2] = parse_input!(inputs[2], i32);
            inventory[3] = parse_input!(inputs[3], i32);
        }
        let score = parse_input!(inputs[4], i32); // amount of rupees
    }
    return Game {
        inventory: inventory,
        spells: spells,
        orders: orders
    };
}

/* ------------------------------------------------------------ */
/* - Functions ------------------------------------------------ */
/* ------------------------------------------------------------ */

fn delta_add(a: [i32; 4], b: [i32; 4]) -> [i32; 4] {
    return [a[0] + b[0], a[1] + b[1], a[2] + b[2], a[3] + b[3]];
}

fn get_possible_recipe(game: &Game) -> Vec<Action> {
    let mut possible_recipe: Vec<Action> = Vec::new();
    for order in game.orders.iter() {
        let missing_table = delta_add(game.inventory, order.delta);
        // eprintln!("inv: {:?} order: {:?}", game.inventory, order.delta);
        // eprintln!("order: {} mt: {:?}", order.id, missing_table);
        if !missing_table.iter().any(|el| *el < 0) {
            possible_recipe.push(*order);
        }
    }
    return possible_recipe;
}

fn get_possible_cast(game: &Game) -> Vec<Action> {
    let mut possible_recipe: Vec<Action> = Vec::new();
    for spell in game.spells.iter() {
        let missing_table = delta_add(game.inventory, spell.delta);
        // eprintln!("mt: {:?}", missing_table);
        let mut sum: i32 = 0;
        for el in delta_add(game.inventory, spell.delta).iter() {
            sum += *el;
        }
        if sum <= 10 && spell.used == 1 && !missing_table.iter().any(|el| *el < 0) {
            possible_recipe.push(*spell);
        }
    }
    return possible_recipe;
}

fn preparation_time_for(delta: [i32; 4], inventory: [i32; 4]) -> i32 {
    let mut time: i32 = 0;
    let new_delta = delta_add(delta, inventory);
    for (i, el) in new_delta.iter().enumerate() {
        if *el < 0 {
            time += (el.abs() * (i + 1) as i32);
        }
    }
    return time;
}

// fn graph_search(cast: Action, path: Vec<Action>, graph) -> bool {
//     if !missing_element_table.iter().any(|&el| el < 0) {
//         return true;
//     }
//     for cast in game.spells {
//         if graph_search(cast, visited, graph) == true {
//             return true;
//         }
//     }
// }

// fn find_fastest_preparation(order: Order, game: &Game) -> Vec<Action> {
//     // let missing_element_table: [i32; 4] = delta_add(game.inventory.delta, order.delta);
    
//     let path: Vec<Action> = Vec::new();
//     for cast in game.spells {
//         if !visited.contains(cast) {
//             path.push(cast);
//             if graph_search(cast, path, graph) == true {
//                 return nb_turn;
//             }
//             path.pop();
//         }
//     }
//     return path;
// }

fn find_best_cast_for(order: Action, game: &Game) -> i32 {
    let possible_cast: Vec<Action> = get_possible_cast(&game);
    let mut best_spell_id: i32 = 0;
    let mut best: i32 =  preparation_time_for(order.delta, game.inventory);
    eprintln!("inv time: {}", best);
    for spell in possible_cast.iter() {
        let casted_inventory = delta_add(game.inventory, spell.delta);
        eprintln!("cast {} casted inv: {:?}", spell.id, casted_inventory);
        let time: i32 = preparation_time_for(order.delta, casted_inventory);
        eprintln!("time: {}", time);
        if time <= best {
            best = time;
            best_spell_id = spell.id;
        }
    }
    eprintln!("best_spell_id: {}", best_spell_id);
    return best_spell_id;
}

/* ------------------------------------------------------------ */
/* - Main ----------------------------------------------------- */
/* ------------------------------------------------------------ */

fn main() {
    loop {
        let mut game: Game = get_turn_informations();
        let possible_recipe: Vec<Action> = get_possible_recipe(&game);
        if possible_recipe.is_empty() {
            if game.spells.iter().any(|spell| spell.used == 1) {
                let mut best_order: (Action, i32) = (Action::new(), std::i32::MAX);
                for order in game.orders.iter() {
                    let time: i32 = preparation_time_for(order.delta, game.inventory);
                    // eprintln!("order: {} time: {}", order.id, time);
                    if time < best_order.1 {
                        best_order = (*order, time);
                    }
                    // let sequence: Vec<Action> = find_fastest_preparation(order, &game);
                    // if sequence.len() < best_order.1 {
                    //     best_order = (order, sequence.len());
                    // }
                }
                eprintln!("closest: {}", best_order.0.id);
                let cast_id = find_best_cast_for(best_order.0, &game);
                if cast_id != 0 {
                    println!("CAST {}", cast_id);
                } else {
                    println!("REST");
                }
            } else {
                println!("REST");
            }
        } else {
            let recipe_id: i32 = possible_recipe.iter().max_by_key(|order| order.price).unwrap().id;
            println!("BREW {}", recipe_id);
        }
    }
}

