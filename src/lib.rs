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

// module declarations
mod cursor;
mod gfx;
mod math;

extern crate alloc;

use agb::{
    display::{
        self,
        Priority::{P2, P3},
        tiled::{RegularBackground, RegularBackgroundSize, TileFormat::FourBpp},
    },
    fixnum::{Rect, Vector2D, vec2},
    include_aseprite, include_background_gfx,
    input::{Button, ButtonController},
};

use crate::{
    cursor::CursorHighlight,
    math::{bound, clamp},
};

// graphics includes
include_aseprite!(
    mod sprites,
    "gfx/sprites/shrooms.aseprite",
    "gfx/sprites/ui/cursor.aseprite"
);

include_background_gfx!(
    mod backgrounds,
    "412d3d",
    BATTLEFIELD => "gfx/backgrounds/battlefield.aseprite",
    MENU => "gfx/backgrounds/ui/menu.aseprite",
    UI_PLAY_BUTTON => "gfx/backgrounds/ui/play_button.aseprite",
);

// System Constants
const TILE_PIXEL_SIZE: i32 = 8;

// Battlefield constants
const BATTLEFIELD_TILE_SIZE: i32 = 2;
const BATTLEFIELD_PIXEL_SIZE: i32 = BATTLEFIELD_TILE_SIZE * TILE_PIXEL_SIZE;
const BATTLEFIELD_WIDTH: i32 = display::WIDTH / BATTLEFIELD_PIXEL_SIZE;
const BATTLEFIELD_HEIGHT: i32 = display::HEIGHT / BATTLEFIELD_PIXEL_SIZE;
const BATTLEFIELD_IDX_BOUNDS: Rect<i32> = Rect::new(
    vec2(0, 0),
    vec2(BATTLEFIELD_WIDTH - 1, BATTLEFIELD_HEIGHT - 1),
);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BattlefieldTileType {
    Empty,
    Tower,
    Cannon,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum MenuState {
    Closed,
    Open(MenuType),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum MenuType {
    EmptyTile,
    OccupiedTile,
}

pub fn main(mut gba: agb::Gba) -> ! {
    // ###### Game state ###### //

    // We'll store the current menu state to drive what UI is shown and how inputs are handled
    let mut menu_state = MenuState::Closed;
    // The tile the cursor is on on the battlefield
    let mut battlefield_cursor_pos: Vector2D<i32> = vec2(0, 0);
    // The actual tiles on the battlefield
    let mut battlefield =
        [[BattlefieldTileType::Empty; BATTLEFIELD_HEIGHT as usize]; BATTLEFIELD_WIDTH as usize];

    // ###### System Initializations ###### //

    // Get the graphics manager, responsible for all the graphics
    let mut gfx = gba.graphics.get();

    // We want to Count frames later for animation timing
    let mut frame_count: usize = 0;

    // Set the palettes to the import, otherwise nothing will show (all 0 palette)
    gfx.set_background_palettes(backgrounds::PALETTES);

    // Two backgrounds, one for the battlefield, one for the ui
    let mut battlefield_background =
        RegularBackground::new(P3, RegularBackgroundSize::Background32x32, FourBpp);
    let mut ui_background =
        RegularBackground::new(P2, RegularBackgroundSize::Background32x32, FourBpp);

    // Fill the battlefield with grass
    for x in 0..BATTLEFIELD_WIDTH as usize {
        for y in 0..BATTLEFIELD_HEIGHT as usize {
            gfx::set_battlefield_tile(
                &mut battlefield_background,
                vec2(x as i32, y as i32),
                battlefield[x][y],
            );
        }
    }

    // The cursor highlight will move around wherever we tell it to, with a nice animation
    let mut cursor_highlight = CursorHighlight::new(
        vec2(0, 0),
        vec2(BATTLEFIELD_PIXEL_SIZE, BATTLEFIELD_PIXEL_SIZE),
    );

    // Input handler
    let mut input = ButtonController::new();

    // Game loop
    loop {
        // Poll the inputs at the top of the frame
        input.update();

        // Start a frame
        let mut frame = gfx.frame();
        frame_count += 1;

        // game logic
        match menu_state {
            // If the menu is closed, controls move the cursor around the battlefield
            MenuState::Closed => {
                // Move the cursor in the direction selected, clamped to the valid area
                let new_battlefield_cursor_pos =
                    battlefield_cursor_pos + input.just_pressed_vector();
                battlefield_cursor_pos = bound(new_battlefield_cursor_pos, BATTLEFIELD_IDX_BOUNDS);

                // Highlight the newly selected square
                let mut cursor_highlight_pos = battlefield_cursor_pos * BATTLEFIELD_PIXEL_SIZE;
                // If the move was unsucessful, move the cursor_highlight in that direction some
                if new_battlefield_cursor_pos != battlefield_cursor_pos {
                    cursor_highlight_pos += input.vector::<i32>() * TILE_PIXEL_SIZE;
                }
                // Get that cursor moving there!
                cursor_highlight.set_target(
                    cursor_highlight_pos,
                    vec2(BATTLEFIELD_PIXEL_SIZE, BATTLEFIELD_PIXEL_SIZE),
                );

                // If A is pressed, open the menu
                if input.is_just_released(Button::A) {
                    if battlefield[battlefield_cursor_pos.x as usize][battlefield_cursor_pos.y as usize] == BattlefieldTileType::Empty {
                        menu_state = MenuState::Open(MenuType::EmptyTile);
                    } else {
                        menu_state = MenuState::Open(MenuType::OccupiedTile)
                    }
                }
            }
            // If the menu is open, controls navigate the cursor around the menu
            MenuState::Open(menu_type) => {
                // TO-DO draw the menu or somethink idfk it's late...
            }
        }

        // Show the backgrounds
        battlefield_background.show(&mut frame);
        ui_background.show(&mut frame);

        // move the cursor and show it
        cursor_highlight.update();
        cursor_highlight.show(&mut frame, frame_count);

        frame.commit();
    }
}
