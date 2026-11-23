# Macaw's Doors for Sunrise

294 doors in every wood and in glass, metal and themed styles, with door prints.

A port of [Macaw's Doors](https://github.com/sketchmacaw/MacawsDoors) by Sketch Macaw and Peachy Macaw to [Sunrise](https://playsunrise.net) (Minecraft Java 26.3), as a content mod with a native part: the original's doors, models, textures, recipes and strings, plus its garage doors and portcullises.

## What it does

- 294 doors: chevron, barn, beach, cottage, classic, four-panel, glass, mystic, nether, paper, stable, swamp, tropical, waffle, western and more, in every wood, pale oak and poplar included.
- Metal, jail, store and sliding glass doors that open only with redstone, like iron doors.
- Four garage doors and two portcullises: place one on the ceiling and click it to roll it down to the floor; click again to roll it up. Doors of the same kind side by side move together, and redstone power opens and closes them. Breaking the top piece gives the item back.
- Japanese and stable doors keep their thin shapes, let you through when open and open by hand with the shoji sound.
- The 12 door prints say "Crafting ingredient" and burn for 300 ticks; the metal doors, garage doors and portcullises carry the original's tooltips.
- The recipes, drops and creative tab.

## Differences from the original

- Redstone power moves a garage door one tick late, and a Japanese or stable door opened by redstone or an explosion plays the ordinary door sound, not the shoji one.
- The stable doors' shapes are trimmed to the block's edge (the original overhangs by 0.02 of a pixel).
- Garage doors and portcullises are not blocked for mobs' pathfinding.

## Building

With the Sunrise mod tool (`cargo sunrise-mod`, from the Sunrise SDK):

```sh
cargo sunrise-mod check
cargo sunrise-mod build    # target/sunrise-mod/mcwdoors.smod
```

This version is made for the Sunrise Dev build `dev-2026-11-22-7cfdf81`.

## Licence and credits

Macaw's Doors is by Sketch Macaw and Peachy Macaw (https://github.com/sketchmacaw/MacawsDoors). Its textures, models, recipes and strings are Sketch Macaw and Peachy Macaw's and are used here under the **MIT licence**, the original's; `LICENSE` holds the text and its copyright notice, and `LICENSE-NOTICE.md` says what this port carries.

Carried to Minecraft 26.3 by AdLf0208 ([AdLf0208/macaws_doors](https://github.com/AdLf0208/macaws_doors)), and ported to Sunrise by arismoko. This port is distributed under the same licence.
