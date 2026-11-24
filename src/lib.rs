//! Macaw's Doors for Sunrise: the doors are content in `data/` and `assets/`; this code runs
//! the garage doors and portcullises (`GarageDoor`) and the shoji sound of the Japanese and
//! stable doors (`JapaneseDoors`).
//!
//! | Moment | Where | What it does |
//! |---|---|---|
//! | `register` | [`declare`] | attaches the parts |
//! | `use_item_on`, `use_without_item` | [`Garage`] | opens or closes the whole door |
//! | `neighbor_changed` | [`Garage`] | redstone opens or closes the door |
//! | `is_pathfindable` | [`Garage`] | no path goes through a door |
//! | `state_for_placement` | [`Garage`] | a quarter clockwise from the placer |
//! | `on_destroyed_by_player` | [`Garage`] | the top segment drops the item |
//! | `use_without_item` | [`Shoji`] | a hand opens a Japanese or stable door with its own sound |

pub mod garage;
pub mod names;

use garage::{Cell, Seg, World};
use mod_api::{
    self, BlockPart, BlockUpdateFlags, Hooks, Host, InteractionResult, IsPathfindableEvent,
    ItemKey, ItemStack, LevelRef, LevelSide, LevelWriteHost, NeighborChangedEvent,
    OnDestroyedByPlayerEvent, Registrar, RegistrationContext, ServerHost, SoundSource,
    UseItemOnEvent, UseWithoutItemEvent,
};

const MOD: &str = "mcwdoors";

/// `SoundsInit.GARAGE`, `SoundsInit.SHOJI`.
const GARAGE_SOUND: &str = "mcwdoors:block.garage";
const SHOJI_SOUND: &str = "mcwdoors:block.shoji";

/// Whether a state string is one of the airs (`BlockState.isAir`).
fn is_air(text: &str) -> bool {
    matches!(
        text,
        "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"
    )
}

/// What a state string is to the door rules.
fn cell(text: &str) -> Cell {
    if is_air(text) {
        return Cell::Air;
    }
    Seg::parse(text)
        .filter(|s| s.block.starts_with("mcwdoors:"))
        .map_or(Cell::Other, Cell::Garage)
}

/// The text a cell writes, or None for a cell the rules never write.
fn cell_text(cell: Cell) -> Option<String> {
    match cell {
        Cell::Air => Some("minecraft:air".to_owned()),
        Cell::Garage(seg) => Some(seg.text()),
        Cell::Other => None,
    }
}

/// The level a neighbour update runs in.
struct Writer<'a> {
    level: &'a mut dyn LevelWriteHost,
}

impl World for Writer<'_> {
    fn at(&self, pos: [i32; 3]) -> Cell {
        self.level
            .block_state(pos)
            .and_then(|state| self.level.state_string(state))
            .map_or(Cell::Other, |text| cell(&text))
    }

    fn put(&mut self, pos: [i32; 3], cell: Cell) {
        if let Some(id) = cell_text(cell).and_then(|text| self.level.block_state_id(&text)) {
            self.level.set_block(
                pos,
                id,
                BlockUpdateFlags::NEIGHBORS | BlockUpdateFlags::CLIENTS,
            );
        }
    }
}

/// A level on the server, read and written through the server host.
struct Level<'a> {
    host: &'a mut dyn ServerHost,
    level: LevelRef,
}

impl World for Level<'_> {
    fn at(&self, pos: [i32; 3]) -> Cell {
        let Some(state) = self.level.block_state(&*self.host, pos) else {
            return Cell::Other;
        };
        self.host
            .block_state_string(state)
            .map_or(Cell::Other, |text| cell(&text))
    }

    fn put(&mut self, pos: [i32; 3], cell: Cell) {
        if let Some(id) = cell_text(cell).and_then(|text| self.host.block_state_id(&text)) {
            let _ = self.level.set_block(
                self.host,
                pos,
                id,
                BlockUpdateFlags::NEIGHBORS | BlockUpdateFlags::CLIENTS,
            );
        }
    }
}

fn play(
    host: &mut dyn ServerHost,
    level: LevelRef,
    sound: &str,
    pos: [i32; 3],
    volume: f32,
    base: f32,
) {
    let unit = level
        .with_random(host, |random| random.next_float())
        .unwrap_or(0.5);
    let [x, y, z] = pos;
    let _ = level.play_sound(
        host,
        sound,
        [f64::from(x) + 0.5, f64::from(y) + 0.5, f64::from(z) + 0.5],
        SoundSource::Blocks,
        volume,
        unit * 0.1 + base,
        None,
    );
}

/// The garage doors and portcullises.
pub struct Garage {
    item: ItemKey,
}

impl Garage {
    /// `GarageDoor.useItemOn`: flip the door, as the clicked block's state says.
    fn click(
        host: &mut dyn ServerHost,
        click: &mod_api::BlockClick<'_>,
    ) -> Option<InteractionResult> {
        let level = host.level(click.dimension.key().id())?;
        let state = level.block_state(&*host, click.pos)?;
        let seg = Seg::parse(&host.block_state_string(state)?)?;
        garage::toggle(
            &mut Level { host, level },
            click.pos,
            !seg.open,
            &seg.facing,
        );
        play(host, level, GARAGE_SOUND, click.pos, 0.5, 0.8);
        Some(InteractionResult::Success)
    }
}

#[mod_api::sunrise_part]
impl BlockPart for Garage {
    fn use_item_on(
        &self,
        _host: &mut dyn ServerHost,
        event: &mut UseItemOnEvent<'_>,
    ) -> Option<InteractionResult> {
        let click = event.click;
        if click.stack.is_some_and(|s| s.item() == self.item) {
            return Some(InteractionResult::Pass);
        }
        // Any other item tries the empty hand, which runs for the main hand only, so a click that
        // reaches the server for both hands moves the door once.
        Some(InteractionResult::TryWithEmptyHand)
    }

    fn use_without_item(
        &self,
        host: &mut dyn ServerHost,
        event: &mut UseWithoutItemEvent<'_>,
    ) -> Option<InteractionResult> {
        Self::click(host, event.click)
    }

    /// `GarageDoor.neighborChanged`: power at the segment or the block below it opens the door,
    /// losing it closes the door, and then the segment is written back as it was before with the
    /// new `powered`, as the original writes it. No sound.
    fn neighbor_changed(&self, level: &mut dyn LevelWriteHost, event: &mut NeighborChangedEvent) {
        let pos = [event.pos.x, event.pos.y, event.pos.z];
        let Some(mut seg) = level.state_string(event.state).and_then(|t| Seg::parse(&t)) else {
            return;
        };
        let signal = |at: [i32; 3]| level.has_neighbor_signal(at).unwrap_or(false);
        let powered = signal(pos) || signal([pos[0], pos[1] - 1, pos[2]]);
        if seg.powered == powered {
            return;
        }
        garage::toggle(
            &mut Writer { level: &mut *level },
            pos,
            powered,
            &seg.facing,
        );
        seg.powered = powered;
        if let Some(id) = level.block_state_id(&seg.text()) {
            level.set_block(
                pos,
                id,
                BlockUpdateFlags::NEIGHBORS | BlockUpdateFlags::CLIENTS,
            );
        }
    }

    /// `GarageDoor.isPathfindable`: no path of any kind goes through a door.
    fn is_pathfindable(&self, _host: &dyn Host, event: &mut IsPathfindableEvent) {
        event.pathfindable = false;
    }

    /// `GarageDoor.getStateForPlacement`: the default state, turned a quarter clockwise from
    /// where the placer looks.
    fn state_for_placement(
        &self,
        level: LevelSide,
        event: &mut mod_api::StateForPlacementEvent<'_>,
    ) -> Option<Option<mod_api::BlockState>> {
        let (LevelSide::Server(level) | LevelSide::Client(level)) = level;
        let block = level.state_block(event.state)?;
        let facing = match event.context.source {
            mod_api::PlacementSource::Player { yaw, .. } => garage::placed_facing(yaw),
            // A quarter clockwise from `DirectionalPlaceContext.getHorizontalDirection`: its own
            // direction, north when it is vertical.
            mod_api::PlacementSource::Directional { direction } => match direction {
                mod_api::Direction::East => "south",
                mod_api::Direction::South => "west",
                mod_api::Direction::West => "north",
                _ => "east",
            },
            _ => return None,
        };
        let seg = Seg {
            block: block.to_owned(),
            facing: facing.to_owned(),
            open: true,
            powered: false,
            part: garage::Part::Top,
        };
        Some(level.block_state_id(&seg.text()))
    }

    /// `GarageDoor.playerWillDestroy`: a survival player breaking the top segment gets the item.
    fn on_destroyed_by_player(
        &self,
        host: &mut dyn ServerHost,
        event: &mut OnDestroyedByPlayerEvent<'_>,
    ) -> Option<bool> {
        let seg = Seg::parse(&host.block_state_string(event.state)?)?;
        let player = host.player(event.player)?;
        if seg.part == garage::Part::Top && !player.is_creative(&*host).unwrap_or(false) {
            let level = player.entity().level(&*host)?;
            let _ = level.pop_resource(host, event.pos, ItemStack::of(self.item));
        }
        None
    }
}

/// `JapaneseDoors.useWithoutItem`: a hand opens it with the shoji sound.
pub struct Shoji;

#[mod_api::sunrise_part]
impl BlockPart for Shoji {
    fn use_without_item(
        &self,
        host: &mut dyn ServerHost,
        event: &mut UseWithoutItemEvent<'_>,
    ) -> Option<InteractionResult> {
        let click = event.click;
        let next = flip_open(&host.block_state_string(event.state)?)?;
        let next = host.block_state_id(&next)?;
        let level = host.level(click.dimension.key().id())?;
        let _ = level.set_block(host, click.pos, next, BlockUpdateFlags::CLIENTS);
        play(host, level, SHOJI_SOUND, click.pos, 1.0, 0.9);
        Some(InteractionResult::Success)
    }
}

/// `state.cycle(OPEN)` on a door's state string.
pub fn flip_open(state: &str) -> Option<String> {
    let (block, props) = state.strip_suffix(']')?.split_once('[')?;
    let mut found = false;
    let props: Vec<String> = props
        .split(',')
        .map(|pair| match pair {
            "open=true" => {
                found = true;
                "open=false".to_owned()
            }
            "open=false" => {
                found = true;
                "open=true".to_owned()
            }
            other => other.to_owned(),
        })
        .collect();
    found.then(|| format!("{block}[{}]", props.join(",")))
}

/// Attach the parts to the blocks the package declares.
pub fn declare(r: &mut dyn Registrar) {
    for name in names::GARAGE {
        let block = r.block_key(&format!("{MOD}:{name}"));
        let item = r.item_key(&format!("{MOD}:{name}"));
        r.block_part(block, Garage { item });
    }
    for name in names::SHOJI {
        let block = r.block_key(&format!("{MOD}:{name}"));
        r.block_part(block, Shoji);
    }
}

#[derive(Default)]
struct Mod;

#[mod_api::sunrise_mod]
impl Hooks for Mod {
    fn register(&self, r: &mut dyn Registrar, _: &RegistrationContext) {
        declare(r);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_hand_flips_open_and_nothing_else() {
        assert_eq!(
            flip_open("mcwdoors:oak_japanese_door[facing=north,half=lower,hinge=left,open=false,powered=false]").as_deref(),
            Some("mcwdoors:oak_japanese_door[facing=north,half=lower,hinge=left,open=true,powered=false]")
        );
        assert_eq!(flip_open("minecraft:stone"), None);
    }

    #[test]
    fn every_listed_block_is_declared() {
        let dir =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("data/mcwdoors/sunrise/block");
        for name in names::GARAGE.iter().chain(names::SHOJI.iter()) {
            assert!(dir.join(format!("{name}.json")).exists(), "{name}");
        }
    }
}
