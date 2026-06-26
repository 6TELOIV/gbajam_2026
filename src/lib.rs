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
        self, Blend, Priority::{P2, P3}, tiled::{RegularBackground, RegularBackgroundSize, TileFormat::FourBpp},
    }, fixnum::{Num, Rect, Vector2D, vec2}, include_aseprite, include_background_gfx, input::{Button, ButtonController},
};
use alloc::vec;

use crate::{cursor::CursorHighlight, gfx::{blank_background, draw_many_tile_data, set_battlefield_tile}, math::bound};

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
    MENU_HEADER => "gfx/backgrounds/ui/menu/_header.aseprite",
    MENU_FOOTER => "gfx/backgrounds/ui/menu/_footer.aseprite",
    MENU_BUILD => "gfx/backgrounds/ui/menu/build.aseprite",
    MENU_SELL => "gfx/backgrounds/ui/menu/sell.aseprite",
    MENU_INFO => "gfx/backgrounds/ui/menu/info.aseprite",
    UI_PLAY_BUTTON => "gfx/backgrounds/ui/play_button.aseprite",
);

// System Constants
const TILE_PIXEL_SIZE: i32 = 8;
const DISPLAY_TILE_WIDTH: i32 = display::WIDTH / TILE_PIXEL_SIZE;
const DISPLAY_TILE_HEIGHT: i32 = display::HEIGHT / TILE_PIXEL_SIZE;

// Battlefield Constants
const BATTLEFIELD_TILE_SIZE: i32 = 2;
const BATTLEFIELD_PIXEL_SIZE: i32 = BATTLEFIELD_TILE_SIZE * TILE_PIXEL_SIZE;
const BATTLEFIELD_WIDTH: i32 = DISPLAY_TILE_WIDTH / BATTLEFIELD_TILE_SIZE;
const BATTLEFIELD_X_MIDPOINT: i32 = BATTLEFIELD_WIDTH / 2;
const BATTLEFIELD_HEIGHT: i32 = DISPLAY_TILE_HEIGHT / BATTLEFIELD_TILE_SIZE;
// const BATTLEFIELD_Y_MIDPOINT: i32 = BATTLEFIELD_HEIGHT / 2;
const BATTLEFIELD_IDX_BOUNDS: Rect<i32> = Rect::new(
    vec2(0, 0),
    vec2(BATTLEFIELD_WIDTH - 1, BATTLEFIELD_HEIGHT - 1),
);

// UI Constants
const MENU_LEFT_X: i32 = 1;
const MENU_RIGHT_X: i32 = DISPLAY_TILE_WIDTH - backgrounds::MENU_HEADER.width as i32 - 1;
const MENU_Y: i32 = 1;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BattlefieldTileType {
    Empty,
    Tower,
    Cannon,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MenuState {
    Closed,
    Open(MenuType, MenuSide),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MenuType {
    EmptyTile,
    OccupiedTile,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MenuSide {
    Left,
    Right,
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
            set_battlefield_tile(
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
                    // Open the menu
                    let menu_type = match battlefield[battlefield_cursor_pos.x as usize]
                        [battlefield_cursor_pos.y as usize]
                    {
                        BattlefieldTileType::Empty => MenuType::EmptyTile,
                        _ => MenuType::OccupiedTile,
                    };
                    // Menu goes on the opposite side of the selected building
                    let menu_side = match battlefield_cursor_pos.x {
                        0..BATTLEFIELD_X_MIDPOINT => MenuSide::Right,
                        _ => MenuSide::Left,
                    };
                    menu_state = MenuState::Open(menu_type, menu_side);
                    // Display the menu
                    let menu_pos = (
                        match menu_side {
                            MenuSide::Left => MENU_LEFT_X,
                            MenuSide::Right => MENU_RIGHT_X,
                        },
                        MENU_Y,
                    )
                        .into();
                    draw_many_tile_data(
                        &mut ui_background,
                        menu_pos,
                        &vec![
                            &backgrounds::MENU_HEADER,
                            &backgrounds::MENU_BUILD,
                            &backgrounds::MENU_INFO,
                            &backgrounds::MENU_FOOTER,
                        ],
                    );
                    // Move the cursor
                    cursor_highlight.set_target(
                        ((menu_pos + vec2(0, backgrounds::MENU_HEADER.height as i32))
                            * TILE_PIXEL_SIZE)
                            + vec2(1, 0),
                        (vec2(
                            backgrounds::MENU_BUILD.width as i32,
                            (backgrounds::MENU_BUILD.height) as i32,
                        ) * TILE_PIXEL_SIZE)
                            - vec2(2, 0),
                    );
                    // Draw the battlefield tile on the ui layer
                    // TO-DO: get the actual tile; just drawing grass for now to get the visual down.
                    set_battlefield_tile(&mut ui_background, battlefield_cursor_pos, BattlefieldTileType::Empty);
                }
            }
            // If the menu is open, controls navigate the cursor around the menu
            MenuState::Open(menu_type, menu_side) => {
                // B button closes menu
                if input.is_just_released(Button::B) {
                    menu_state = MenuState::Closed;
                    blank_background(&mut ui_background, &backgrounds::MENU_HEADER);
                }

            }
        }

        // Show the backgrounds
        let battlefield_background_id = battlefield_background.show(&mut frame);
        if let MenuState::Open(..) = menu_state {
            frame.blend().darken(Num::from_f32(0.5)).enable_background(battlefield_background_id).enable_backdrop();
        }
        ui_background.show(&mut frame);

        // move the cursor and show it
        cursor_highlight.update();
        cursor_highlight.show(&mut frame, frame_count);

        frame.commit();
    }
}
