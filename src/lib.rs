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
        tiled::{TileSetting},
    },
    fixnum::Vector2D, input::ButtonController,
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
    "gfx/shrooms.aseprite"
);

include_background_gfx!(
    mod backgrounds,
    "412d3d",
    BATTLEFIELD => "gfx/battlefield.aseprite",
    BATTLEFIELD_UI => "gfx/battlefield_ui.aseprite",
);

const SHROOM_WALK_SPRITES: &'static [usize] = &[0, 1, 0, 2];

/// Helper function to draw a 16x16 meta tile to a position based on a 16x16 grid
fn set_meta_tile(
    background: &mut RegularBackground,
    pos: impl Into<Vector2D<i32>>,
    tile_data: &TileData,
    meta_tile_idx: usize,
) -> () {
    let base_pos = pos.into() * 2;
    let base_tile_settings_idx = meta_tile_idx * 2;
    let row_two_offset = tile_data.tile_settings.len() / 2;
    background.set_tile(
        base_pos,
        &tile_data.tiles,
        tile_data.tile_settings[base_tile_settings_idx],
    );
    background.set_tile(
        base_pos + (1, 0).into(),
        &tile_data.tiles,
        tile_data.tile_settings[base_tile_settings_idx + 1],
    );
    background.set_tile(
        base_pos + (0, 1).into(),
        &tile_data.tiles,
        tile_data.tile_settings[base_tile_settings_idx + row_two_offset],
    );
    background.set_tile(
        base_pos + (1, 1).into(),
        &tile_data.tiles,
        tile_data.tile_settings[base_tile_settings_idx + 1 + row_two_offset],
    );
}
/// helper function to empty a 16x16 meta tile to a position based on a 16x16 grid
fn blank_meta_tile(
    background: &mut RegularBackground,
    pos: impl Into<Vector2D<i32>>,
    tile_data: &TileData,
) -> () {
    let pos = pos.into() * 2;
    background.set_tile(
        pos,
        &tile_data.tiles,
        TileSetting::BLANK
    );
    background.set_tile(
        pos + (1, 0).into(),
        &tile_data.tiles,
        TileSetting::BLANK
    );
    background.set_tile(
        pos + (0, 1).into(),
        &tile_data.tiles,
        TileSetting::BLANK
    );
    background.set_tile(
        pos + (1, 1).into(),
        &tile_data.tiles,
        TileSetting::BLANK
    );
}

// Manages the tile battlefield state and display of said state
#[derive(Copy, Clone)]
enum BuildingType {
    // Grass = 0,
    Mountain = 1,
    // Archer = 2,
    // Canon = 3,
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
                pos: (2, 2).into(),
            }],
            cursor_pos: (2,2).into(),
            previous_cursor_pos: (64,64).into(),
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
            set_meta_tile(
                battlefield_background,
                building.pos,
                &backgrounds::BATTLEFIELD,
                building.building_type as usize,
            );
        }
        if self.previous_cursor_pos != self.cursor_pos {
            blank_meta_tile(ui_background, self.previous_cursor_pos, &backgrounds::BATTLEFIELD_UI);
            set_meta_tile(ui_background, self.cursor_pos, &backgrounds::BATTLEFIELD_UI, frame_count / 32);
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
        backgrounds::BATTLEFIELD.tiles.format(),
    );
    // Make the backgrounds we'll need
    let mut ui_background = RegularBackground::new(
        Priority::P0,
        RegularBackgroundSize::Background32x32,
        backgrounds::BATTLEFIELD.tiles.format(),
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
        if input.just_pressed_vector::<i32>() != (0,0).into() {
            battlefield.set_cursor_pos(battlefield.cursor_pos() + input.just_pressed_vector());
        }

        // count the frames
        frame_count = (frame_count + 1) % 64;

        // Start a frame
        let mut frame = gfx.frame();

        // Show the bg
        battlefield.show(&mut frame, frame_count, &mut battlefield_background, &mut ui_background);

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
