# CG 2020 Fall Challenge PostMortem

## Stats
* Overall rank: 389 out of 7011 players
* Team rank (42 school): 4 out of 197 players

## The challenge
Basically, we control a witch that can prepare and sell potions, <a href=https://www.codingame.com/challengereport/28507351a92c5398f4545e49406f995619744234>official rules here</a> 
<div><img src="battle_demo.gif" width="300"/></div>

## My Progression (Lazy to read ? Pseudocode at the end)
### Wood1 League (Day 1)
#### Rules
* We have an inventory of some elements tier0, tier1, tier2 and tier3.
* There is a set of orders that each need differents elements to be prepared.
* We can use one BREW action per turn to serve an order, the player earn the price of the order.
* The first player who served 2 orders end the battle.
* The winner is the one with the best score.
#### My strategy
So I basically made a function to choose the two best potions I could do with my inventory, and that was enougth to pass Wood2.
### Wood2 League (Day 1)
#### Rules
* We now have the 4 base spells available:
```
[ 2,  0,  0,  0], // make 2 tier0
[-1,  1,  0,  0], // make 1 tier1 from 1 tier0
[ 0, -1,  1,  0], // make 1 tier2 from 1 tier1
[ 0,  0, -1,  1]  // make 1 tier3 from 1 tier2
```
* We can use only one spell per turn.
* Every time we use a spell, we have to REST to use it again, the REST action make all the used spells usable again, but it cost a turn.
#### My strategy
* There is the interesting feature of the challenge, the goal will be to arrange the inventory by using spells too have the necessary elements to make the potions. I started to imagine how to make a Graph search but I guessed that it was not necessary to go to Bronze league, so I just calculated the time necessary to make every potions, choose the fastests and apply spells.<br/>
Submit -> Bronze :ok_hand:
### Bronze League (Day 2 to 5)
#### Rules
* We can now buy differents spells from a mutual book, once a sepll learn it disapear from the book,
* Some spells from the book are repeatable, it means they are usable several times on the same turn.
* To buy a spell, we have to pay a tax of tier0 elements that correspond to it's index on the list and dispose it on every book spells that are below iton the list.
* When you buy a spell you earn the tier0 elements that correspond to that was add to this spell if someone buy spells above it.
* And finally, now the two firsts orders give bonus score, the first one 3 and the second 1, theses bonus are applies only 4 times each during the game.
* The game finish after 100 turn or if a play made 6 potions.
#### My strategy
From that moment I started to think Graph trasversal, rules were more complex but we could expose the problem in a data tree. I choosed to start with a basic DFS because I knew that it would be useless to go to far on the search, with the opponents actions the game become chaotic after some turns and a simulation would become useless ! Moreover, the DFS allow to perform a search without storing too much data, a BFS or equivalent could be faster but there should be too much nodes  to store and and I prefered to optimize the time complexity than the size complexity !

I made the basics, a function to make neighbors for each nodes, SPELL, LEARN and REST actions were handled at this moment, I made my recursive function to iterate over the graph, and a simulation system. At this moment I realized that my search would not be well distributed on every starting nodes, so I updated it to make an IDDFS instead (I could do IDA* but I had no clue of what heuristics I could use), with that I explore every nodes of depth and then iterate that depth and start again, not very fast but enougth for now.

Basically, what I was doing at this moment was, iterate over graph and stop as soon as it found a solution, then execute the first action of the path.

I did some improvements as searching for multiple paths to find the best one during the 50 available ms, handling the 1000ms available on the first turn, and prioritize path that started by LEARN to avoid path destruction in case of the opponent take it. That was enougth to be more or less 60th on the scoreboard and pass to the Iron league when it oppened :v: !
### Iron League (Day 5 to 12)
* No new rules, the Bronze ones where the last to change
#### My strategy
I needed to improve my search, to pass Gold I had to stay not far from the top ranking, firstly I made corrections, I found a lot of errors on my simulation, theses errors were quite hard to detect actually, but by observing passed battle against other I managed to fix the most !

Then I added a new possible action on my Graph: BREW, it seems obvious but I wasn't handling that one before, I was stopping the depth search when I found one.. with tmy update I then tried to find the path with the most BREWs on it, I also tried to get the ones where there are fast to do !
At this moment my scoring function was someting like that :
```
for (i, action) in path.iter().enumerate() {
  score += action.price / (1.0 + i as f32);
}
```
I also defined a default action in case of no found path, it was to LEARN the first spell of the book, it was free and could unstuck the situation. Problem though: more actions equal more neighbors equal less time to explore depth. I also added a condition in case of I had an empty inventory: fill the half of it.

Gold league open and... I didn't pass ! They took only 10% of the Iron one :scream:

So I tried features as adding a memory function to keep passed found path for some situations and others. I updated my scoring function taking care of the number of passed turns and the probable number of turns remaining, the score difference between players and the numbers of orders every players has served... I also made a lot of optimisations, code refactoring and data structure compressions but nothing pushed me on the Gold league !

So I changed my strategy, I made a MCTS with one neighbors depth, and tried to run a maximum of simulations from every first depth nodes and compute a score from it, made optimisations for it and after hours of coding got the result: still stuck in Iron ! Arg !

At this moment I knew that the problem was my scoring functions, my features were logic but didn't pushed me upper on the scoreboard, so the last night I just tried everything to make this &%#!!$$$% of scoring function doing something right.

4am: I gave up the MCTS, my IDDFS were pusing me upper on the ranking, tried my best to optimize it to finish not bad on the Iron league scoreboard.

8am: I checked the scoreboard, and see NicolaH_42 with who I was working, and who was stuck as me on the scoring function in Iron, passing Gold, he finally found a good one, well quite simple but fitting on my program:
```
for (i, action) in path.iter().enumerate() {
  score += action.price - (2.0 * depth as f32) + inventory_nb_of_elements(&game);
}
```
Still have 2 hours to pass Gold, I tried that one, submitted, waited 1 hour because everyone were submitting at the same moment... and finally passed Gold :clap:

### Gold League (Day 12)
One hour left to do something and try to gain some rank before the end of the contest, and I seriously need to sleep..
I try to apply the function on the MCTS to see if it make something magic... Nope, even by normalizing the (2 * i), no time to try something else, I submit the IDDFS and it's 10am the challenge end !

## Final Pseudocode
```
find_neighbors() {
  return possible REST, CAST, LEARN and BREW;
}

fn simulate(game) {
  match action {
    "LEARN" => simulate taxes price + taxes payment & reception + book state + spells state,
    "CAST" => simulate spells state + inventory state,
    "REST" => simulate spells state,
    "BREW" => simulate game score + orders state + inventory state
  }
  return simulation;
}

fn graph_search(cost, bound, game) -> bool {
  if game.path_score > best_path.score => best_path = game.path_score;
  if cost > bound => return true;
  for neighbour in find_neighbors().iter() {
    if timeout => return false;
    simulation = simulate(game);
    graph_search(cost + 1, bound, simulation)
  }
}

fn find_best_path(game) {
  loop {
    if !graph_search(0, bound, game) => break;
    bound += 1;
  }
  if game_not_altered && stored_path.score > path_score => return stored_path;
  if path any found => return path; // LEARN first cast of book in case of first 5 turns
  if inventory.is_empty() => return CAST to fill it; // REST in case of no possible actions
  return LEARN first cast of book;
}

fn main() {
  loop {
    parse_game();
    best_path = find_best_path(game);
    stored_path = best_path;
    println!(best_path.first());
  }
}
```

