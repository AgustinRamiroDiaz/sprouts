# Sprouts Game in Bevy Rust

https://en.wikipedia.org/wiki/Sprouts_(game)
http://www.papg.com/show?1TMQ

## Plan

Do something very similar to https://github.com/ChrisSmith2/Sprouts-Game

Game flow:

1. player puts down the initial nodes in the grid
1. player starts drawing the line between the nodes
1. a new node is automatically created in the middle when the line is connected to the other node
1. the graph automatically adjusts lengths using some physics gravitational simulation to look pretty

Physic simulation is key, and I hope that there's no edge overlapping
