# Vexation

Vexation is a game very similar to [Aggravation](https://en.wikipedia.org/wiki/Aggravation_(board_game)) but with a few tweaks.

## The Objective

The first player to move all 5 of their marbles clockwise around the board and into their "home" row wins.

## The Board

> TODO: include an annotated picture

1. Base
    * This is where marbles start and go back to if they are captured.
    * The base with the dice indicates the current player.
    * The base highlighed indicates the human player.
2. Starting space
    * Must roll a 1 to exit the base and land on this space.
3. Home row
    * Only the matching color can enter their home row.
4. Shortcut entrance
    * The 3 colors in the corner indicate which players **can** enter from this space.
    * The colored arrow indicates which player **cannot** enter from this space.
    * Can only enter with an exact roll of **both** dice. Ex: Marble at starting space and rolled a total of 6 (e.g, 2+4, 3+3, etc.).
5. Center
    * Must roll a 1 to exit.
    * Can only exit in the direction matching your color.

## Setup

The human player picks their color by clicking on the "base" of the color they want. Then random player is chosen to go first.

## Player Turn

First, the dice are automatically rolled for the current player. The player can then use the values of the dice individually to move two marbles or use the sum of the dice to move one marble clockwise around the board.

## Marble Movement and Captures

A player's marble can hop over opponents marbles but cannot hop over their own marbles. If a marble lands exacly on an opponent's marble the opponent's marble is considered "captured" and is moved back to its base.

---

## Running the game

Download or clone this repository, install [Rust](https://rust-lang.org), and running `cargo run` at the root of the directory.
