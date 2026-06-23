// Games made using `agb` are no_std which means you don't have access to the standard
// rust library. This is because the game boy advance doesn't really have an operating
// system, so most of the content of the standard library doesn't apply.
//
// Provided you haven't disabled it, agb does provide an allocator, so it is possible
// to use both the `core` and the `alloc` built in crates.
#![no_std]
// `agb` defines its own `main` function, so you must declare your game's main function
// using the #[agb::entry] proc macro. Failing to do so will cause failure in linking
// which won't be a particularly clear error message.
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]
#![feature(const_destruct)]

use core::{
    cmp::{max, min},
    marker::Destruct,
};

use agb::{
    display::{
        GraphicsFrame,
        tile_data::TileData,
        tiled::{TileFormat, TileSetting},
    },
    fixnum::Vector2D,
    input::{Button::A, ButtonController},
};

use agb::{
    display::{
        Priority,
        object::Object,
        tiled::{RegularBackground, RegularBackgroundSize},
    },
    include_aseprite, include_background_gfx,
};

include_aseprite!(
    mod sprites,
    "gfx/sprites/shrooms.aseprite",
    "gfx/sprites/ui/cursor.aseprite"
);

include_background_gfx!(
    mod backgrounds,
    "412d3d",
    BATTLEFIELD_BUILDINGS => "gfx/backgrounds/battlefield/buildings.aseprite",
    UI_BUTTONS => "gfx/backgrounds/ui/buttons.aseprite",
);

const SHROOM_WALK_SPRITES: &'static [usize] = &[0, 1, 0, 2];

/// Given an input, a min, and a max, clamps the input between the two values (inclusive)
fn clamp<T>(value: T, min_value: T, max_value: T) -> T
where
    T: Ord + Destruct,
{
    return min(max_value, max(min_value, value));
}

/// Draws a group of tiles from a background tile set
/// Assumes that the imported aseprite image has logical groups of that size in a column
fn set_tiles(
    background: &mut RegularBackground,
    pos: Vector2D<i32>,
    tile_data: &TileData,
    tile_idx: usize,
    tile_width: usize,
    tile_height: usize,
) {
    let base_idx = tile_width * tile_height * tile_idx;
    for x in 0..tile_width {
        for y in 0..tile_height {
            let pos = pos + (x as i32, y as i32).into();
            background.set_tile(
                pos,
                &tile_data.tiles,
                tile_data.tile_settings[base_idx + x + (y * tile_width)],
            );
        }
    }
}
/// Draws a group of blank tiles from a background tile set
fn blank_tiles(
    background: &mut RegularBackground,
    pos: impl Into<Vector2D<i32>> + Copy,
    tile_data: &TileData,
    tile_width: usize,
    tile_height: usize,
) {
    for x in 0..tile_width {
        for y in 0..tile_height {
            background.set_tile(
                pos.into() + (x as i32, y as i32).into(),
                &tile_data.tiles,
                TileSetting::BLANK,
            );
        }
    }
}

// Manages the tile battlefield state and display of said state
#[derive(Copy, Clone)]
enum BuildingType {
    Grass = 0,
    Mountain = 1,
    Archer = 2,
    Canon = 3,
}

const BATTLEFIELD_WIDTH: usize = 14;
const BATTLEFIELD_HEIGHT: usize = 9;
const BATTLEFIELD_X_OFFSET: i32 = 0;
const BATTLEFIELD_Y_OFFSET: i32 = 1;

pub struct Battlefield {
    battlefield: RegularBackground,
    cursor: Object,
    buildings: [[BuildingType; BATTLEFIELD_HEIGHT]; BATTLEFIELD_WIDTH],
    cursor_pos: Vector2D<usize>,
}

fn battlefield_pos_to_offset_pos(pos: Vector2D<usize>) -> Vector2D<i32> {
    (
        pos.x as i32 + BATTLEFIELD_X_OFFSET,
        pos.y as i32 + BATTLEFIELD_Y_OFFSET,
    )
        .into()
}

fn battlefield_pos_to_tile_pos(pos: Vector2D<usize>) -> Vector2D<i32> {
    battlefield_pos_to_offset_pos(pos) * 2
}

fn battlefield_pos_to_screen_pos(pos: Vector2D<usize>) -> Vector2D<i32> {
    battlefield_pos_to_offset_pos(pos) * 16
}

impl Battlefield {
    fn new() -> Self {
        let mut cursor = Object::new(sprites::CURSOR.sprite(0));
        let cursor_pos = (0usize, 0usize).into();
        let mut battlefield = RegularBackground::new(
            Priority::P3,
            RegularBackgroundSize::Background32x32,
            TileFormat::FourBpp,
        );
        // initialzie the battlefield background with grass
        for x in 0..BATTLEFIELD_WIDTH {
            for y in 0..BATTLEFIELD_HEIGHT {
                set_tiles(
                    &mut battlefield,
                    battlefield_pos_to_tile_pos((x, y).into()),
                    &backgrounds::BATTLEFIELD_BUILDINGS,
                    BuildingType::Grass as usize,
                    2,
                    2,
                );
            }
        }
        // initialize the cursor at the right position
        cursor.set_pos(battlefield_pos_to_screen_pos(cursor_pos));
        Self {
            battlefield,
            cursor,
            buildings: [[BuildingType::Grass; BATTLEFIELD_HEIGHT]; BATTLEFIELD_WIDTH],
            cursor_pos,
        }
    }
    fn cursor_pos(&self) -> Vector2D<usize> {
        self.cursor_pos
    }
    fn set_cursor_pos(&mut self, pos: Vector2D<i32>) {
        self.cursor_pos = (
            clamp(pos.x, 0, BATTLEFIELD_WIDTH as i32 - 1) as usize,
            clamp(pos.y, 0, BATTLEFIELD_HEIGHT as i32 - 1) as usize,
        )
            .into();
        self.cursor
            .set_pos(battlefield_pos_to_screen_pos(self.cursor_pos));
    }
    fn set_building(&mut self, pos: Vector2D<usize>, building_type: BuildingType) {
        self.buildings[pos.x][pos.y] = building_type;
        set_tiles(
            &mut self.battlefield,
            battlefield_pos_to_tile_pos(pos),
            &backgrounds::BATTLEFIELD_BUILDINGS,
            building_type as usize,
            2,
            2,
        );
    }
    fn show(&mut self, frame: &mut GraphicsFrame, frame_count: usize) -> &mut Self {
        // draw the buildings
        // place and animate the cursor
        self.cursor
            .set_sprite(sprites::CURSOR.animation_sprite(frame_count / 32));

        // show them graphics!
        self.cursor.show(frame);
        self.battlefield.show(frame);
        self
    }
}

pub fn main(mut gba: agb::Gba) -> ! {
    // Get the graphics manager, responsible for all the graphics
    let mut gfx = gba.graphics.get();

    // Set the palettes to the import, otherwise nothing will show (all 0 palette)
    gfx.set_background_palettes(backgrounds::PALETTES);

    // Make the battlefield
    let mut battlefield = Battlefield::new();

    // Create objects with the sprites
    let mut shroom = Object::new(sprites::SHROOM.sprite(0));
    let mut shroom_fast = Object::new(sprites::SHROOM_FAST.sprite(0));

    // Place them at some points on the screen
    shroom.set_pos((48, 48));
    shroom_fast.set_pos((64, 64));

    // Count frames for animation timing
    let mut frame_count: usize = 0;
    let mut shroom_animation_idx = 0;

    // Get inputs
    let mut input = ButtonController::new();

    loop {
        // get inputs
        input.update();

        // move the cursor
        let pressed_vector = input.just_pressed_vector::<i32>();
        if pressed_vector.x != 0 || pressed_vector.y != 0 {
            let cursor_pos = battlefield.cursor_pos();
            battlefield
                .set_cursor_pos(pressed_vector + (cursor_pos.x as i32, cursor_pos.y as i32).into());
        }

        if input.is_just_pressed(A) {
            battlefield.set_building(battlefield.cursor_pos(), BuildingType::Archer);
        }

        // count the frames
        frame_count += 1;

        // Start a frame
        let mut frame = gfx.frame();

        // Show the bg
        battlefield.show(&mut frame, frame_count);

        if frame_count % 8 == 0 {
            // Set the object sprites based on the frame count
            shroom_animation_idx = (shroom_animation_idx + 1) % SHROOM_WALK_SPRITES.len();
            shroom.set_sprite(sprites::SHROOM.sprite(SHROOM_WALK_SPRITES[shroom_animation_idx]));
            shroom_fast
                .set_sprite(sprites::SHROOM_FAST.sprite(SHROOM_WALK_SPRITES[shroom_animation_idx]));
        }

        if frame_count % 16 == 0 {
            // Move the objects
            shroom.set_pos(shroom.pos() + (1, 0).into());
            shroom_fast.set_pos(shroom_fast.pos() + (1, 0).into());
        }

        // Actually show these objects on the screen
        shroom.show(&mut frame);
        shroom_fast.show(&mut frame);

        // Until the call to `frame.commit()`, nothing will be displayed
        frame.commit();
    }
}
