# Macaw's Doors — a Horizon port

**Macaw's Doors** is a Minecraft mod. Source code for Macaw's Doors. This repository is a **port of it to [Horizon](https://github.com/horizon-rust/horizon)**, the Rust engine for Minecraft Java 26.3. It carries the complete source of the port plus the content files it ships, and installs as a single `.hmod` package.

Upstream: [sketchmacaw/MacawsDoors](https://github.com/sketchmacaw/MacawsDoors) — licence **MIT**.

| | |
|---|---|
| Upstream mod | Macaw's Doors ([source](https://github.com/sketchmacaw/MacawsDoors)) |
| Ported to | Horizon 0b14f8a, Minecraft Java 26.3, mod API `0.2` |
| Package | `mcwdoors.hmod` (install it with the Horizon launcher) |
| Licence | **MIT** — the upstream mod's licence; see [Licence and credit](#licence-and-credit) |

## What this port implements

- 239 Macaw's doors on vanilla templates, with the 12 door-print items.

## Not yet ported

- **Template numbers.** No NeoForge oracle here, so per-block hardness, blast
- **`JapaneseDoors` / `StableDoor` behaviour is not carried.** The content
- **Garage doors and portcullises** are not carried at all (gap 1 above): no
- **Door prints** are plain items with the `stick` template; upstream adds a
- **Fuel burn time** is the vanilla door's 200 ticks, not upstream's
- **`cherry_waffle_door`** is absent (incomplete assets upstream; see above);
- **Creative tab.** Upstream's `TabInit` is Java; since a data-tier port has no
- **Tags naming left-out ids.** `data/minecraft/tags/block/mineable/axe.json`

## Installing

1. Install the [Horizon launcher](https://github.com/horizon-rust/horizon/releases) (0.2.70 or newer).
2. Add `mcwdoors.hmod` to the instance's mods.
3. The package is pinned to a specific Horizon build; the launcher checks that pin.

## Licence and credit

Macaw's Doors was created by its upstream authors; the code, models and assets in the original mod
remain theirs and are used here under **MIT**. The Rust in this repository is a clean-room
rewrite for Horizon, not a translation of the original source.

- `LICENSE` — the upstream licence text.
- `LICENSE-NOTICE.md` — what this port carries and where it came from.
- The upstream project: https://github.com/sketchmacaw/MacawsDoors

## Technical details

```
horizon-mod.toml   package manifest (id, version, api, [native] pin)
Cargo.toml         the crate; depends on `horizon-mod-api` from the fetched SDK
src/               lib.rs (the one `impl Hooks`), runtime.rs (pure logic + tests), glue
assets/ data/      the content the package ships
tools/             content generators and the e2e scenario
```

- Build: `cargo horizon-mod build` (packages; it does not compile against the game).
- Test: `cargo test`, `cargo clippy --all-targets -- -D warnings`.
- In-game: `ports/tools/launcher-game.sh run mcwdoors` in the campaign tree, or install the
  `.hmod` and play; the harness scenario is `tools/e2e.sh`.

This repository is generated from the campaign's port at `ports/mcwdoors/`; the port's own
`notes.md` is the full record (decisions, traps, evidence).
