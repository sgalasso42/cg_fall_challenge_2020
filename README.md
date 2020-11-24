# CG 2020 Fall Challenge Postmortem

## Stats
<img alt="Rust" src="https://img.shields.io/badge/Rust-orange?logo=rust"/>

## The challenge
<img src="battle_demo.gif" width="300"/>

## Day 1 : Wood 1&2
### My strategy
#### Wood 1 : Simple choice comparaison function
For the first Wood league, we got simplified rules, we have a filled inventory and two spells to do.
There is a set of simple potions to do, and our inventory is filled with some elements, so basically I just made a function to choose the two best potions I can do with my inventory, and that was enougth to pass Wood2.
#### Wood 2 : Scoring spells sequence
New rules has been added on the second league. We now have the 4 base spells available, we can use only one spell per turn. Every time we use a spell, we have to rest to use it again, the rest action make all the used spells usable again, but it cost a turn.
```
Spells:
[ 2,  0,  0,  0], // make 2 tier0
[-1,  1,  0,  0], // make 1 tier1 from 1 tier0
[ 0, -1,  1,  0], // make 1 tier2 from 1 tier1
[ 0,  0, -1,  1]  // make 1 tier3 from 1 tier2
```
Ok we now see the interesting feature of the challenge, the goal will be to arrange the inventory by using spells too have the necessary elements to make the potions. I started to imagine how to make a Graph search but actually, I guessed that it was not necessary to go to Bronze league, so I just calculated the time necessary to make every potions, choose the fastests and apply spells.
## Day 2 to 4 : Bronze
New rules: We now can buy differents spells from a mutual book
### My strategy

## Day 5 to 11 : Iron

## Day 12 : Gold
