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

extern crate alloc;

use agb::{
    display::{
        GraphicsFrame,
        tile_data::TileData,
        tiled::{TileFormat, TileSetting},
    },
    fixnum::Vector2D,
    input::ButtonController,
};
use alloc::vec;
use alloc::vec::Vec;

use agb::{
    display::{
        Priority,
        object::Object,
        tiled::{RegularBackground, RegularBackgroundSize},
    },
    include_aseprite, include_background_gfx,
};

include_aseprite!(
    mod shrooms,
    "gfx/sprites/shrooms.aseprite"
);

include_background_gfx!(
    mod backgrounds,
    "412d3d",
    BATTLEFIELD_BUILDINGS => "gfx/backgrounds/battlefield/buildings.aseprite",
    UI_CURSOR => "gfx/backgrounds/ui/cursor.aseprite",
    UI_BUTTONS => "gfx/backgrounds/ui/buttons.aseprite",
);

const SHROOM_WALK_SPRITES: &'static [usize] = &[0, 1, 0, 2];

/// Draws a group of tiles from a background tile set
/// Assumes that the imported aseprite image has logical groups of that size in a column
fn set_tiles(
    background: &mut RegularBackground,
    pos: impl Into<Vector2D<i32>> + Copy,
    tile_data: &TileData,
    tile_idx: usize,
    tile_width: usize,
    tile_height: usize,
) {
    let base_idx = tile_width * tile_height * tile_idx;
    for x in 0..tile_width {
        for y in 0..tile_height {
            background.set_tile(
                pos.into() + (x as i32, y as i32).into(),
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

pub struct Building {
    building_type: BuildingType,
    pos: Vector2D<i32>,
}

pub struct Battlefield {
    buildings: Vec<Building>,
    cursor_pos: Vector2D<i32>,
    previous_cursor_pos: Vector2D<i32>,
}

impl Battlefield {
    fn new() -> Self {
        Self {
            buildings: vec![Building {
                building_type: BuildingType::Mountain,
                pos: (0, 0).into(),
            }],
            cursor_pos: (2, 2).into(),
            previous_cursor_pos: (64, 64).into(),
        }
    }
    fn cursor_pos(&self) -> Vector2D<i32> {
        self.cursor_pos
    }
    fn set_cursor_pos(&mut self, pos: Vector2D<i32>) {
        self.previous_cursor_pos = self.cursor_pos;
        self.cursor_pos = pos;
    }
    fn show(
        &mut self,
        frame: &mut GraphicsFrame,
        frame_count: usize,
        battlefield_background: &mut RegularBackground,
        ui_background: &mut RegularBackground,
    ) -> &mut Self {
        for building in &self.buildings {
            set_tiles(
                battlefield_background,
                building.pos * 2,
                &backgrounds::BATTLEFIELD_BUILDINGS,
                building.building_type as usize,
                2,
                2,
            );
        }
        if self.previous_cursor_pos != self.cursor_pos {
            blank_tiles(
                ui_background,
                self.previous_cursor_pos * 2,
                &backgrounds::UI_CURSOR,
                2,
                2,
            );
            set_tiles(
                ui_background,
                self.cursor_pos * 2,
                &backgrounds::UI_CURSOR,
                frame_count / 32,
                2,
                2,
            );
        }
        battlefield_background.show(frame);
        ui_background.show(frame);
        self
    }
}

pub fn main(mut gba: agb::Gba) -> ! {
    // Get the graphics manager, responsible for all the graphics
    let mut gfx = gba.graphics.get();

    // Set the palettes to the import, otherwise nothing will show (all 0 palette)
    gfx.set_background_palettes(backgrounds::PALETTES);

    // Make the backgrounds we'll need
    let mut battlefield_background = RegularBackground::new(
        Priority::P3,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );
    // Make the backgrounds we'll need
    let mut ui_background = RegularBackground::new(
        Priority::P0,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );

    // Make the battlefield
    let mut battlefield = Battlefield::new();

    // Create an objects with the sprites
    let mut shroom = Object::new(shrooms::SHROOM.sprite(0));
    let mut shroom_fast = Object::new(shrooms::SHROOM_FAST.sprite(0));

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
        if input.just_pressed_vector::<i32>() != (0, 0).into() {
            battlefield.set_cursor_pos(battlefield.cursor_pos() + input.just_pressed_vector());
        }

        // count the frames
        frame_count = (frame_count + 1) % 64;

        // Start a frame
        let mut frame = gfx.frame();

        // Show the bg
        battlefield.show(
            &mut frame,
            frame_count,
            &mut battlefield_background,
            &mut ui_background,
        );

        if frame_count % 8 == 0 {
            // Set the object sprites based on the frame count
            shroom_animation_idx = (shroom_animation_idx + 1) % SHROOM_WALK_SPRITES.len();
            shroom.set_sprite(shrooms::SHROOM.sprite(SHROOM_WALK_SPRITES[shroom_animation_idx]));
            shroom_fast
                .set_sprite(shrooms::SHROOM_FAST.sprite(SHROOM_WALK_SPRITES[shroom_animation_idx]));
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
