# Macaw's Doors for Sunrise

239 doors in every wood and in glass, metal and themed styles, with door prints.

A port of [Macaw's Doors](https://github.com/sketchmacaw/MacawsDoors) by Sketch Macaw and Peachy Macaw to [Sunrise](https://playsunrise.net) (Minecraft Java 26.3), as a content mod: the original's doors, models, textures, recipes and strings, with no code.

## What it does

- 239 doors: barn, beach, cottage, classic, four-panel, glass, mystic, nether, paper, stable, swamp, tropical, waffle, western and more, in every wood.
- Metal, jail, store and sliding glass doors that open only with redstone, like iron doors.
- The 12 door prints used to craft them, the recipes, drops and creative tab.

## Differences from the original

- Garage doors and portcullises are not ported (they are Java blocks with no vanilla counterpart).
- Japanese and stable doors open as plain doors, not with their own two-part behaviour.

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
