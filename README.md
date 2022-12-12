# Vexation

Vexation is a game very similar to [Aggravation](https://en.wikipedia.org/wiki/Aggravation_(board_game)) but with a few tweaks.

> It was originally inspired by https://github.com/voxelv/marbles by @voxelv! Thanks dude!

![vexation screenshot](https://github.com/Zooce/vexation/blob/main/extra/images/Vexation.png?raw=true)

## The Objective

The first player to move all 5 of their marbles clockwise around the board and into their "home" row wins.

## The Board

> TODO: annotated picture

1. Base
    * This is where marbles start and go back to if they are captured.
    * The base with the dice indicates the current player.
    * The base that is outlined indicates the human player.
2. Starting space
    * Must roll a 1 or a 6 to exit the base and land on this space.
3. Home row
    * Only the matching color can enter their home row.
4. Center space
    * Can only enter with an exact roll using **both** dice. Ex: Marble at starting space and rolled a total of 6 (e.g, 2+4, 3+3, etc.).
    * Can only enter from a corner where the arrow point out does **NOT** match your color.
    * Must roll a 1 to exit the center space.
    * Can only exit in the direction indicated by the arrow matching your color.

## Setup

The human player picks their color by clicking on the "base" of the color they want. Then a random player is chosen to go first.

## Player Turn

First, the dice are automatically rolled for the current player. The player can then use the values of the dice individually to move one or two marbles, or use the sum of the dice to move one marble. 

## Marble Movement and Captures

Movement is always clockwise around the board. A player's marble can hop over opponents' marbles but cannot hop over their own marbles. If a marble lands exactly on an opponent's marble the opponent's marble is considered "captured" and is moved back to its base.

---

## Running the game

Make sure you have [Git LFS](https://git-lfs.github.com/) installed. This is used to store all of the image assets.

Download or clone this repository, install [Rust](https://rust-lang.org), and run `cargo run` at the root of the directory. Optionally use the `--features bevy/dynamic` flag to speed things up a bit if you have nightly installed.

---

# Credits

Made by me, [Zooce](https://github.com/Zooce).

Inspired by [marbles.world](https://github.com/voxelv/marbles) by [voxelv](https://github.com/voxelv).

Some assets used/inspired from the very generous [Kenney](https://kenney.nl/).
