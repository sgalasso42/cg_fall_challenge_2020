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
    price: i8,
    tax: i8,
    pocket: i8,
    repeatable: i8,
    repeat: i8,
    castable: i8
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

#[derive(Clone, Debug)]
struct Node {
    action: Action,
    state: Game,
    score: f32,
    n: u32,
}

impl Node {
    fn new(action: Action, state: Game) -> Node {
        return Node {
            action: action,
            state: state,
            score: 0.0,
            n: 0,
        }
    }
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
    return game.orders.iter().filter(|order| !delta_add(&game.inventory, &order.delta).iter().any(|el| *el < 0) && (game.served < 5 || game.my_score + order.price as i32 + inventory_final_score(delta_add(&game.inventory, &order.delta)) > game.opp_score + game.opp_inventory_score)).cloned().collect::<Vec<Action>>();
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
    // let calculated_turns_left: i32 = ((100 - game.calc_turn) as f32 * ((6 - cmp::max(game.served, game.opp_served)) as f32 / 6.0)) as i32;
    // let calculated_turns_left: i32 = ((100 - game.calc_turn) as f32 / (1.0 + cmp::max(game.served, game.opp_served) as f32)) as i32;
    if game.calc_turn < 100 && game.served < 6 {
        neighbors = [&get_available_spells(&game)[..], &get_available_learns(&game)[..], &get_available_brews(&game)[..]].concat();
        if game.spells.iter().any(|spell| spell.castable == 0) {
            neighbors.push(Action::new("REST"));
        }
    }
    return neighbors;
}

fn path_score(path: &Vec<Action>, game: &Game) -> f32 {
    let mut total_price: i8 = 0;
    let mut score: f32 = 0.0;
    let mut nb: i8 = 0;
    for (i, action) in path.iter().enumerate() {
        if &(action.action)[..] == "BREW" {
            // let turn = game.calc_turn + i as i32;
            // let coef: f32 = turn as f32 / 100.0;
            // score += action.price as f32 / (1.0 + (i as f32/* / (1.0 - coef)*/));
            // score += action.price as f32 - action.price as f32 * (i as f32 / 100.0);
            // score += ((100 - i) as f32 * ((action.price as f32 * 100.0) / 126.0));
            // total_price += action.price;
            // nb += 1;
            
            // let calculated_turns_left: i8 = ((100 - (game.turn + i as i32)) as f32 / (1.0 + cmp::max(game.served, game.opp_served) as f32)) as i8;
            // let calculated_turns_left: i8 = ((100 - game.turn) as f32 * ((6 - cmp::max(game.served, game.opp_served)) as f32 / 6.0)) as i8;
            // let calculated_turns_left: i8 = ((100 - game.turn) as f32 * (1.0 - (cmp::max(game.served, game.opp_served) as f32 / 7.0) as f32)) as i8;
            // score += action.price as f32 - action.price as f32 * (i as f32 / (100 - (game.turn + i as i32)) as f32); // RELEASE
            // score += action.price as f32 / (1.0 + i as f32);
            score += action.price as f32 / i as f32;
        }
    }
    // return score as f32 / path.len() as f32;
    // return score as f32 * (((100 - path.len()) as f32 * 126.0) / 100.0);
    // return score;
    return score as f32;
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
            let spell_pos: usize = game_simulation.spells.iter().position(|spell| spell.delta == delta).expect("game_simulation.spells.iter().position(|spell| spell.delta == delta)") as usize;
            game_simulation.spells[spell_pos].castable = 0;
            game_simulation.inventory = delta_add(&game.inventory, &action.delta);
        },
        "LEARN" => {
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
            game_simulation.inventory = delta_add(&delta_add(&game.inventory, &[action.pocket, 0, 0, 0]), &[-action.tax, 0, 0, 0]);
        },
        "BREW" => {
            game_simulation.served += 1;
            game_simulation.my_score += action.price as i32;
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
    game_simulation.calc_turn += 1;
    return game_simulation;
}

fn playout(current: &Node, explored_nodes: &mut u32, start_time: std::time::Instant) -> Option<Vec<Action>> {
    let mut path: Vec<Action> = vec![current.action.clone()];
    let mut simulation: Game = current.state.clone();
    loop {
        *explored_nodes += 1;
        match get_neighbors(&simulation) {
            _ if (simulation.turn == 1 && start_time.elapsed().as_millis() > 998) || (simulation.turn > 1 && start_time.elapsed().as_millis() > 48) => {
                eprintln!("timeout at: {:.3?} explored_nodes: {}", start_time.elapsed(), explored_nodes);
                return None;
            },
            ref neighbors if neighbors.len() > 0 => {
                let action: Action = neighbors[rand::thread_rng().gen_range(0, &neighbors.len())].clone();
                path.push(action.clone());
                simulation = simulate(&action, &simulation);
            },
            _ => {
                return Some(path);
            }
        }
    }
}

fn uct(neighbors: &Vec<Node>, parent_n: u32) -> Option<usize> {
    let mut max: (Option<usize>, f32) = (None, std::f32::NEG_INFINITY);
    for (i, neighbour) in neighbors.iter().enumerate() {
        if neighbour.n == 0 {
            return Some(i);
        }
        let val: f32 = (neighbour.score / neighbour.n as f32) + (2 as f32).sqrt() * ((parent_n as f32).ln() / neighbour.n as f32).sqrt();
        if val > max.1 {
            max = (Some(i), val);
        }
    }
    return max.0;
}

fn find_best_action(game: &Game, start_time: std::time::Instant) -> ((Action, f32), String) {
    let mut explored_nodes: u32 = 0;
    let mut n: u32 = 0;
    let mut neighbors: Vec<Node> = get_neighbors(&game).iter().map(|neighbour| Node::new(neighbour.clone(), simulate(neighbour, game))).collect();
    loop {
        match uct(&neighbors, n) {
            Some(current) => {
                match playout(&neighbors[current], &mut explored_nodes, start_time) {
                    Some(path) => {
                        neighbors[current].score += path_score(&path, game);
                        neighbors[current].n += 1;
                        n += 1;
                    },
                    _ => {
                        if !neighbors.iter().any(|neighbour| neighbour.score > 0.0) {
                            if game.book.len() > 0 {
                                return ((game.book[0].clone(), 0.0), String::from("No path found"));
                            }
                            return ((Action::new("REST"), 0.0), String::from("No path found & no book left)"));
                        }
                        let mut max: (Action, f32) = (Action::new(""), 0.0);
                        for neighbour in neighbors.iter() {
                            let ratio = neighbour.score / neighbour.n as f32;
                            eprintln!("{:<5} {:<5} score: {:<9.3} n: {:<5} ratio: {:<5.3}", neighbour.action.action, neighbour.action.id, neighbour.score, neighbour.n, ratio);
                            if ratio > max.1 {
                                max = (neighbour.action.clone(), ratio);
                            }
                        }
                        return (max, String::from(format!("{}, {}", n, explored_nodes)));
                    }
                }
            },
            _ => {
                if game.book.len() > 0 {
                    return ((game.book[0].clone(), 0.0), String::from("No path with score > 0"));
                }
                return ((Action::new("REST"), 0.0), String::from("No path found & no book left)"));
            }
        }
    }
}

/* ------------------------------------------------------------ */
/* - Main ----------------------------------------------------- */
/* ------------------------------------------------------------ */

fn main() {
    let mut turn: i32 = 0;
    let mut served: i8 = 0;
    let mut opp_score: i32 = 0;
    let mut opp_served: i8 = 0;
    loop {
        turn += 1;
        let mut game: Game = get_turn_informations(turn, served, opp_score, opp_served);
        let start_time: std::time::Instant = Instant::now();
        opp_score = game.opp_score;
        opp_served = game.opp_served;
        eprintln!("my_score: {} opp_score: {}", game.my_score, game.opp_score);
        eprintln!("my_served: {} opp_served: {}", game.served, game.opp_served);
        // eprintln!("calculated turns left: {}", ((100 - game.turn) as f32 * ((6 - cmp::max(game.served, game.opp_served)) as f32 / 6.0)) as i8);
        eprintln!("calculated turns left: {}", ((100 - game.turn) as f32 * (1.0 - (cmp::max(game.served, game.opp_served) as f32 / 7.0) as f32)) as i8);
        let mut res: ((Action, f32), String) = find_best_action(&game, start_time);
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
