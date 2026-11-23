//! Macaw's Doors for Sunrise: the doors are content in `data/` and `assets/`; this code runs
//! the garage doors and portcullises (`GarageDoor`) and the shoji sound of the Japanese and
//! stable doors (`JapaneseDoors`).
//!
//! | Moment | Where | What it does |
//! |---|---|---|
//! | `register` | [`declare`] | attaches the parts |
//! | `use_item_on`, `use_without_item` | [`Garage`] | opens or closes the whole door |
//! | `neighbor_changed` | [`Garage`] | queues a door whose neighbour changed |
//! | `level_tick_post` | this module | redstone opens or closes each queued door |
//! | `state_for_placement` | [`Garage`] | a quarter clockwise from the placer |
//! | `on_destroyed_by_player` | [`Garage`] | the top segment drops the item |
//! | `use_without_item` | [`Shoji`] | a hand opens a Japanese or stable door with its own sound |

pub mod garage;
pub mod names;

use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Mutex, MutexGuard};

use garage::{Cell, Seg, World};
use mod_api::{
    self, BlockPart, BlockUpdateFlags, Hooks, HostSide, InteractionResult, ItemKey, ItemStack,
    LevelRef, LevelSide, LevelTickPostEvent, NeighborChangedEvent, OnDestroyedByPlayerEvent,
    Registrar, RegistrationContext, ServerHost, SoundSource, UseItemOnEvent, UseWithoutItemEvent,
};

const MOD: &str = "mcwdoors";

/// `SoundsInit.GARAGE`, `SoundsInit.SHOJI`.
const GARAGE_SOUND: &str = "mcwdoors:block.garage";
const SHOJI_SOUND: &str = "mcwdoors:block.shoji";

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
        let Some(text) = self.host.block_state_string(state) else {
            return Cell::Other;
        };
        if matches!(
            text.as_str(),
            "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"
        ) {
            return Cell::Air;
        }
        Seg::parse(&text)
            .filter(|s| s.block.starts_with("mcwdoors:"))
            .map_or(Cell::Other, Cell::Garage)
    }

    fn put(&mut self, pos: [i32; 3], cell: Cell) {
        let text = match cell {
            Cell::Air => "minecraft:air".to_owned(),
            Cell::Garage(seg) => seg.text(),
            Cell::Other => return,
        };
        if let Some(id) = self.host.block_state_id(&text) {
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
        host: &mut dyn ServerHost,
        event: &mut UseItemOnEvent<'_>,
    ) -> Option<InteractionResult> {
        let click = event.click;
        if click.stack.is_some_and(|s| s.item() == self.item) {
            return Some(InteractionResult::Pass);
        }
        Self::click(host, click)
    }

    fn use_without_item(
        &self,
        host: &mut dyn ServerHost,
        event: &mut UseWithoutItemEvent<'_>,
    ) -> Option<InteractionResult> {
        Self::click(host, event.click)
    }

    /// `GarageDoor.neighborChanged`: remember the door for this tick's redstone pass.
    fn neighbor_changed(
        &self,
        _level: &mut dyn mod_api::LevelTicksHost,
        event: &mut NeighborChangedEvent,
    ) {
        enqueue(
            event.dimension.key().id(),
            [event.pos.x, event.pos.y, event.pos.z],
        );
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
        let seg = Seg {
            block: block.to_owned(),
            facing: garage::placed_facing(event.context.yaw).to_owned(),
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

/// How many positions one level may keep queued: a `/fill` of doors queues one per block.
const MAX_QUEUED: usize = 8192;

fn queue() -> MutexGuard<'static, BTreeMap<&'static str, BTreeSet<[i32; 3]>>> {
    static QUEUE: Mutex<BTreeMap<&'static str, BTreeSet<[i32; 3]>>> = Mutex::new(BTreeMap::new());
    QUEUE
        .lock()
        .expect("the garage queue lock is never held across a panic")
}

fn enqueue(dimension: &'static str, pos: [i32; 3]) {
    let mut queue = queue();
    let waiting = queue.entry(dimension).or_default();
    if waiting.len() < MAX_QUEUED {
        waiting.insert(pos);
    }
}

/// `GarageDoor.neighborChanged` for the level that just ticked: a door whose power changed
/// opens (powered) or closes, and keeps the new `powered`.
pub fn on_level_tick_post(host: &mut dyn ServerHost, level: LevelRef) {
    let Some(dimension) = level.dimension().id(&*host) else {
        return;
    };
    let waiting = queue().remove(dimension).unwrap_or_default();
    for pos in waiting {
        let Some(state) = level.block_state(&*host, pos) else {
            continue;
        };
        let Some(seg) = host
            .block_state_string(state)
            .and_then(|t| Seg::parse(&t))
            .filter(|s| s.block.starts_with("mcwdoors:"))
        else {
            continue;
        };
        let signal = |at: [i32; 3]| level.best_neighbor_signal(&*host, at).unwrap_or(0) > 0;
        let powered = signal(pos) || signal([pos[0], pos[1] - 1, pos[2]]);
        if seg.powered == powered {
            continue;
        }
        garage::toggle(&mut Level { host, level }, pos, powered, &seg.facing);
        let Some(state) = level.block_state(&*host, pos) else {
            continue;
        };
        let Some(mut seg) = host
            .block_state_string(state)
            .and_then(|t| Seg::parse(&t))
            .filter(|s| s.block.starts_with("mcwdoors:"))
        else {
            continue;
        };
        seg.powered = powered;
        if let Some(id) = host.block_state_id(&seg.text()) {
            let _ = level.set_block(
                host,
                pos,
                id,
                BlockUpdateFlags::NEIGHBORS | BlockUpdateFlags::CLIENTS,
            );
        }
    }
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

    fn level_tick_post(&self, host: HostSide<'_>, event: &mut LevelTickPostEvent) {
        if let HostSide::Server(host) = host {
            on_level_tick_post(host, event.level);
        }
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
