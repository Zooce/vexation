# The Aggravation game with Bevy

This is my implementation of Aggravation with the Bevy engine!

> Important: Remove the "dynamic" feature from our Bevy dep before release.

> Note: I'm following the Bevy book to get this set up.

## TODOs (in no particular order)

Drawing Systems:
- [ ] Draw the board
    - [ ] 4x (5x5) player tiles (red, yellow, green, and blue)
    - [ ] 69x game tiles
        - [ ] Player start tiles (1 per player - should have an arrow showing the movment direction)
        - [ ] "Safe" tiles (should have a different color)
        - [ ] "Home" tiles (5 per player)
        - [ ] "Center" tile (should have a different color)
    - [ ] Human player indicator
- [ ] Draw the dice
    - [ ] Orientation is based on current player
    - Note: the player with the dice is the current player
- [ ] Draw the marbles (5 per player)
    - [ ] "Jailed" marbles' orientation is based on current player
- [ ] Animate marble movement

User Input:
- [ ] Allow user to chose their move (2-step: choose marble > choose destination)

Other Game Systems:
- [ ] Setup (resources, system registration, spawn entities, etc.)
- [ ] Roll dice for the current player
- [ ] Choose next player
- [ ] Make computer player moves
- [ ] Choose the player who goes first
- [ ] Player picks their color
