use std::io;
use std::time::{Instant};
use rand::Rng;
use std::cmp;

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
    calc_turn: i32,
    my_score: i32,
    opp_score: i32,
    served: i8,
    opp_served: i8,
    inventory: [i8; 4],
    opp_inventory_score: i32,
    spells: Vec<Action>,
    book: Vec<Action>,
    orders: Vec<Action>
}

/* ------------------------------------------------------------ */
/* - Parsing -------------------------------------------------- */
/* ------------------------------------------------------------ */

fn get_turn_informations(turn: i32, served: i8, previous_opp_score: i32, mut opp_served: i8) -> Game {
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
    if previous_opp_score != opp_score {
        opp_served += 1;
    }
    return Game {
        turn: turn,
        calc_turn: turn,
        my_score: my_score,
        opp_score: opp_score,
        served: served,
        opp_served: opp_served,
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

fn inventory_final_score(state: [i8; 4]) -> i32 {
    return (state[1] + state[2] + state[3]) as i32;
}

fn get_available_brews(game: &Game) -> Vec<Action> {
    if game.served > 6 {
        return Vec::new();
    }
    let mut available_brews: Vec<Action> = game.orders.iter().filter(|order| !delta_add(&game.inventory, &order.delta).iter().any(|el| *el < 0)).cloned().collect::<Vec<Action>>();
    return available_brews.iter().filter(|brew| game.served < 5 || game.my_score + brew.price as i32 + inventory_final_score(delta_add(&game.inventory, &brew.delta)) - 1 /* -1 ARBITRAIRE SERT DE SECURITEE, A REFACTOR VOIE BLOC NOTE */ > game.opp_score + game.opp_inventory_score).cloned().collect::<Vec<Action>>();
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
                // eprintln!("mt: {:?}", missing_table);
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
    let mut neighbors: Vec<Action> = [&get_available_spells(&game)[..], &get_available_learns(&game)[..], &get_available_brews(&game)[..]].concat();
    if game.spells.iter().any(|spell| spell.castable == 0) {
        neighbors.push(Action::new("REST"));
    }
    return neighbors;
}

fn path_score(path: &Vec<Action>, game: &Game) -> f64 {
    let mut total_price: i8 = 0;
    let mut score: f64 = 0.0;
    for (i, action) in path.iter().enumerate() {
        if &(action.action)[..] == "BREW" {
            let turn = game.turn + i as i32;
            let coef: f64 = turn as f64 / 100.0;
            // score += action.price as f64 / (1.0 + (i as f64/* / (1.0 - coef)*/)); // RELEASE
            // score += action.price as f64 - action.price as f64 * (i as f64 / 100.0); // DEBUG
            score += action.price as f64;
        }
    }
    return score as f64;
}

fn simulate(action: &Action, game: &Game) -> Game {
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
            game_simulation.inventory = delta_add(&game.inventory, &action.delta);
        },
        "LEARN" => {
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
            game_simulation.inventory = delta_add(&delta_add(&game.inventory, &[action.pocket, 0, 0, 0]), &[-action.tax, 0, 0, 0]);
        },
        "BREW" => {
            game_simulation.served += 1;
            game_simulation.my_score += action.price as i32;
            let brew_index = game_simulation.orders.iter().position(|brew| brew.id == action.id).unwrap();
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
    game_simulation.calc_turn += 1;
    return game_simulation;
}

fn playout(path: &mut Vec<Action>, game_simulation: &Game, explored_nodes: &mut usize, start_time: std::time::Instant) -> bool {
    *explored_nodes += 1;
    if (game_simulation.turn == 1 && start_time.elapsed().as_millis() > 998) || (game_simulation.turn > 1 && start_time.elapsed().as_millis() > 48) {
        eprintln!("timeout at: {:.3?} explored_nodes: {}", start_time.elapsed(), explored_nodes);
        return false;
    }
    if path.len() as i32 > (((100.0 - game_simulation.calc_turn as f64) * ((6.0 - cmp::max(game_simulation.served, game_simulation.opp_served) as f64) / 6.0)) / 2.0) as i32 {
        return true;
    }
    let neighbors: Vec<Action> = get_neighbors(&game_simulation);
    if neighbors.len() > 0 {
        let action: Action = neighbors[rand::thread_rng().gen_range(0, &neighbors.len())].clone();
        path.push(action.clone());
        let simulation: Game = simulate(&action, &game_simulation);
        if playout(path, &simulation, explored_nodes, start_time) == false {
            return false;
        }
    }
    return true;
}

fn find_best_action(game: &Game) -> ((Action, f64), String) {
    let start_time: std::time::Instant = Instant::now();
    let mut explored_nodes: usize = 0;
    let mut projections: i32 = 0;
    let neighbors: Vec<Action> = get_neighbors(&game);
    let mut scored_neighbors: Vec<(&Action, f64)> = neighbors.iter().map(|neighbour| (neighbour, 0.0)).collect();
    eprintln!("bound: {}", (((100.0 - game.calc_turn as f64) * ((6.0 - cmp::max(game.served, game.opp_served) as f64) / 6.0)) / 2.0) as i32);
    loop {
        projections += 1;
        for neighbour in scored_neighbors.iter_mut() {
            let mut path: Vec<Action> = vec![neighbour.0.clone()];
            let simulation: Game = simulate(neighbour.0, game);
            if !playout(&mut path, &simulation, &mut explored_nodes, start_time) {
                if game.turn > 1 { eprintln!("time average per projection: {:.3}", 50.0 / projections as f64); }
                if !scored_neighbors.iter().any(|n| n.1 > 0.0) {
                    return ((game.book[0].clone(), 0.0), String::from("Et merde !"));
                }
                let mut max: (Action, f64) = (Action::new(""), 0.0);
                for n in scored_neighbors.iter() { // because I can't use max_by_key
                    let ratio = n.1 ;/// projections as f64; // TODO: prendre en compte la derniere projection probalbement non terminee
                    eprintln!("{:<5} {:<5} {:<5.3}", n.0.action, n.0.id, ratio);
                    if ratio > max.1 {
                        max = (n.0.clone(), ratio);
                    }
                }
                return (max, String::from(format!("{}, {}", projections, explored_nodes)));
            }
            neighbour.1 += path_score(&path, game);
        }
    }
}

/* - MCTS ----------------------------------------------------- */
// use std::collections::VecDeque;

// struct Node {
//     state: &Game,
//     score: f64,
//     n: f64,
//     neighbors: VecDeque<Node>,
// }

// fn ucb1(node: &Node, parent: &Node) -> f64 {
//     return (node.score / node.n) + 2 * sqrt(ln(parent.n) / node.n);
// }

// fn get_neighbors(node: &Node) -> {
//     //
// }

// fn expand_neighbors(node: &Node) -> {
//     //
// }

// fn playout(node: &Node) -> Node{
//     let mut current: Node = node.clone();
//     let mut neighbors: Vec<Node> = get_neighbors(&current);
//     while neighbors.len() > 0 {
//         let action: Action = neighbors[rand::thread_rng().gen_range(0, &neighbors.len())].clone();
//         current = simulate(&action, &current);
//         neighbors: Vec<Node> = get_neighbors(&current);
//     }
//     return current;
// }

// fn mcts() {
//     let start_time: std::time::Instant = Instant::now();
//     let mut tree: VecDeque<Node> = VecDeque::new();
//     loop {
//         if (game_simulation.turn == 1 && start_time.elapsed().as_millis() > 998) || (game_simulation.turn > 1 && start_time.elapsed().as_millis() > 48) {
//             eprintln!("timeout at: {:.3?}", start_time.elapsed());
//             return max(/* first actions in tree */);
//         }
//         let node = /* initial game state */;
//         while node /* is not a leaf */ {
//             let neighbors = get_neighbors(node);
//             get_neighbors(node).iter().map(|neighbour| neighbour.ucb1 = ucb1(neighbour, neighbour.parent));
//             node = neighbors.iter().max_by_key(|neighbour| neighbour.ucb1);
//         }
//         if node.nb_projections == 0 {
//             let res = graph_search(node);
//             backpropagate(res);
//         } else {
//             expand_tree(node);
//             let neighbors = get_neighbors(node);
//             graph_search(neighbors.first());
//             backpropagate(res);
//         }
//     }
// }

/* ------------------------------------------------------------ */
/* - Main ----------------------------------------------------- */
/* ------------------------------------------------------------ */

fn main() {
    let mut turn: i32 = 0;
    let mut served: i8 = 0;
    let mut opp_score: i32 = 0;
    let mut opp_served: i8 = 0;
    // let mut game_forecast: Option<Game> = None;
    loop {
        turn += 1;
        let mut game: Game = get_turn_informations(turn, served, opp_score, opp_served);
        opp_score = game.opp_score;
        opp_served = game.opp_served;
        eprintln!("my_score: {} opp_score: {}", game.my_score, game.opp_score);
        eprintln!("my_served: {} opp_served: {}", game.served, game.opp_served);
        let start_time = Instant::now();
        if game.book.is_empty() {
            println!("REST Oups !");
        } else {
            let mut res: ((Action, f64), String) = find_best_action(&game);
            eprintln!("graph search duration: {:.3?}", start_time.elapsed());
            let action = (res.0).0.clone();
            match &(action.action)[..] {
                "CAST" => {
                    println!("CAST {} {} {}", action.id, action.repeat, res.1);
                },
                "LEARN" => {
                    println!("LEARN {} {}", action.id, res.1);
                },
                "REST" => {
                    println!("REST {}", res.1);
                },
                "BREW" => {
                    println!("BREW {} {}", action.id, res.1);
                    served += 1;
                },
                _ => {
                    println!("REST [!] Who the fuck wrote this code [!]");
                }
            }
        }
    }
}

// - C, registered_path, game_forecast
