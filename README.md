# MTG Engine

A Magic: The Gathering rules engine and simulator built in Rust, targeting Modern format.

## Goal

A fully rules-enforced MTG simulator where you can load Modern decklists and play games — starting with Amulet Titan and Splinter Twin.

## Architecture

```
src/
  card/       — Card types, supertypes, keywords, card data
  mana/       — Mana costs, mana pools, color system
  zone/       — Zone management (library, hand, battlefield, graveyard, exile, stack)
  player/     — Player state, life totals, land drops
  game/       — Game state, turn structure, phases and steps
  stack/      — The stack, priority, spell/ability resolution
  ability/    — Triggered, activated, and static abilities
  combat/     — Combat phases, attackers, blockers, damage
  effect/     — Permanent state, counters, continuous effects
  data/       — Card builders and sample decklists
```

## Building

```
cargo build --release
cargo run
```

## Roadmap

1. **Vanilla Magic** — lands, mana, vanilla creatures, combat, life totals, full turn loop
2. **Keywords** — flying, trample, deathtouch, first strike, lifelink, etc.
3. **Instants and sorceries** — targeting, stack resolution, counterspells
4. **Triggered and activated abilities** — ETB effects, mana abilities, death triggers
5. **Modern staples** — implement cards needed for target decklists
6. **Decklist loader** — import from Scryfall/MTGO format
7. **TUI** — terminal-based interactive play interface
