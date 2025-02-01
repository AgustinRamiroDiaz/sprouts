# Sprouts Game in Bevy Rust

https://en.wikipedia.org/wiki/Sprouts_(game)
http://www.papg.com/show?1TMQ

# Running

```
trunk serve
```

or

```
cargo run
```

## Plan

Do something very similar to https://github.com/ChrisSmith2/Sprouts-Game

Game flow:

1. player puts down the initial nodes in the grid
1. player starts drawing the line between the nodes
1. a new node is automatically created in the middle when the line is connected to the other node
1. the graph automatically adjusts lengths using some physics gravitational simulation to look pretty

Physic simulation is key, and I hope that there's no edge overlapping

Physics simulation ideas:

- edges will actually be chains like https://github.com/Jondolf/avian/blob/main/crates/avian2d/examples/chain_2d.rs
- when the user draws the edge, internally they'll be links from the chain
- the chain will be a physics body
- nodes and chains will try to repel each other
