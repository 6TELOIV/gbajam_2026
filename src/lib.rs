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

use core::cmp::{max, min};

use agb::{
    display::{
        GraphicsFrame,
        tile_data::TileData,
        tiled::{TileFormat, TileSetting},
    },
    fixnum::Vector2D,
    input::{
        Button::{self, A},
        ButtonController,
    },
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
    UI_PLAY_BUTTON => "gfx/backgrounds/ui/play_button.aseprite",
);

/// Given an input, a min, and a max, clamps the input between the two values (inclusive)
fn clamp<T>(value: T, min_value: T, max_value: T) -> T
where
    T: Ord,
{
    min(max_value, max(min_value, value))
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

// Constants and helpers for dealing with coordinates on the battlefield
const BATTLEFIELD_WIDTH: usize = 14;
const BATTLEFIELD_HEIGHT: usize = 9;
const BATTLEFIELD_X_OFFSET: i32 = 0;
const BATTLEFIELD_Y_OFFSET: i32 = 1;

#[derive(Clone, Copy)]
pub struct BattlefieldCoordinate(Vector2D<usize>);

impl BattlefieldCoordinate {
    pub fn new(pos: impl Into<Vector2D<usize>>) -> Self {
        Self(pos.into())
    }
    fn to_offset_pos(&self) -> Vector2D<i32> {
        (
            self.0.x as i32 + BATTLEFIELD_X_OFFSET,
            self.0.y as i32 + BATTLEFIELD_Y_OFFSET,
        )
            .into()
    }
    pub fn to_tile_pos(&self) -> Vector2D<i32> {
        self.to_offset_pos() * 2
    }
    pub fn to_screen_pos(&self) -> Vector2D<i32> {
        self.to_offset_pos() * 16
    }
    /// doesn't modify the value; returns a new coordinate with the added value.
    pub fn add_vector_clamped(&self, vector: Vector2D<i32>) -> Self {
        (
            clamp(self.0.x as i32 + vector.x, 0, BATTLEFIELD_WIDTH as i32 - 1) as usize,
            clamp(self.0.y as i32 + vector.y, 0, BATTLEFIELD_HEIGHT as i32 - 1) as usize,
        )
            .into()
    }
}

impl From<(usize, usize)> for BattlefieldCoordinate {
    fn from(pos: (usize, usize)) -> Self {
        Self(pos.into())
    }
}

const BUTTONS_LEN: usize = 3;
const BUTTONS_SIZE: usize = BUTTONS_LEN * 2 * 8;

#[derive(Clone, Copy)]
enum CursorPosition {
    Battlefield(BattlefieldCoordinate),
    Buttons(usize),
}

pub struct Cursor {
    object_left: Object,
    object_right: Object,
    pos: CursorPosition,
}

impl Cursor {
    pub fn new() -> Self {
        let mut object_right = Object::new(sprites::CURSOR.sprite(0));
        object_right.set_hflip(true);
        Self {
            object_left: Object::new(sprites::CURSOR.sprite(0)),
            object_right,
            pos: CursorPosition::Battlefield(BattlefieldCoordinate::new((0usize, 0usize))),
        }
    }
    pub fn pos(&self) -> CursorPosition {
        self.pos
    }
    pub fn update(&mut self, input: &ButtonController) {
        let pressed_vector: Vector2D<i32> = input.just_pressed_vector();
        match self.pos {
            CursorPosition::Battlefield(battlefield_pos) => {
                if battlefield_pos.0.y == 0 && pressed_vector.y == -1 {
                    // If we're at the top of the battlefield, jump into the buttons area
                    self.pos = CursorPosition::Buttons(0);
                } else {
                    // Otherwise, move around the battlefield
                    self.pos = CursorPosition::Battlefield(
                        battlefield_pos.add_vector_clamped(pressed_vector),
                    );
                }
            }
            CursorPosition::Buttons(button_index) => {
                if pressed_vector.y == 1 {
                    // If the down arrow is pressed, go down to the battlefield.
                    self.pos = CursorPosition::Battlefield((0, 0).into());
                } else {
                    // otherwise, move the button we're focused on
                    self.pos = CursorPosition::Buttons(clamp(
                        button_index as i32 + pressed_vector.x,
                        0,
                        BUTTONS_LEN as i32,
                    ) as usize);
                }
            }
        }
    }
    pub fn show(&mut self, frame: &mut GraphicsFrame, frame_count: usize) {
        let sprite = sprites::CURSOR.animation_sprite(frame_count / 32);
        match self.pos {
            CursorPosition::Battlefield(battlefield_pos) => {
                self.object_left.set_sprite(sprite);
                self.object_right.set_sprite(sprite);
                self.object_left.set_pos(battlefield_pos.to_screen_pos());
                self.object_right
                    .set_pos(battlefield_pos.to_screen_pos() + (8, 0).into());
            }
            CursorPosition::Buttons(button_index) => {
                self.object_left
                    .set_sprite(sprites::CURSOR.animation_sprite(frame_count / 32));
                self.object_right
                    .set_sprite(sprites::CURSOR.animation_sprite(frame_count / 32));
                self.object_left
                    .set_pos(((button_index * BUTTONS_SIZE) as i32, 0));
                self.object_right
                    .set_pos((((button_index + 1) * BUTTONS_SIZE - 8) as i32, 0));
            }
        }
        self.object_left.show(frame);
        self.object_right.show(frame);
    }
}

#[derive(Copy, Clone)]
enum BuildingType {
    Grass = 0,
    Mountain = 1,
    Archer = 2,
    Canon = 3,
}

pub struct Buildings {
    background: RegularBackground,
    buildings: [[BuildingType; BATTLEFIELD_HEIGHT]; BATTLEFIELD_WIDTH],
}

impl Buildings {
    fn new() -> Self {
        let mut background = RegularBackground::new(
            Priority::P3,
            RegularBackgroundSize::Background32x32,
            TileFormat::FourBpp,
        );
        let buildings = [[BuildingType::Grass; BATTLEFIELD_HEIGHT]; BATTLEFIELD_WIDTH];

        // initialzie the battlefield background with grass
        for x in 0..BATTLEFIELD_WIDTH {
            for y in 0..BATTLEFIELD_HEIGHT {
                set_tiles(
                    &mut background,
                    BattlefieldCoordinate::new((x, y)).to_tile_pos(),
                    &backgrounds::BATTLEFIELD_BUILDINGS,
                    BuildingType::Grass as usize,
                    2,
                    2,
                );
            }
        }
        Self {
            background,
            buildings,
        }
    }
    fn get_building(&self, pos: BattlefieldCoordinate) -> BuildingType {
        self.buildings[pos.0.x][pos.0.y]
    }
    fn set_building(&mut self, pos: BattlefieldCoordinate, building_type: BuildingType) {
        self.buildings[pos.0.x][pos.0.y] = building_type;
        set_tiles(
            &mut self.background,
            pos.to_tile_pos(),
            &backgrounds::BATTLEFIELD_BUILDINGS,
            building_type as usize,
            2,
            2,
        );
    }
    fn show(&mut self, frame: &mut GraphicsFrame) {
        self.background.show(frame);
    }
}

pub struct UserInterface {
    background: RegularBackground,
}

impl UserInterface {
    pub fn new() -> Self {
        let mut background = RegularBackground::new(
            Priority::P3,
            RegularBackgroundSize::Background32x32,
            TileFormat::FourBpp,
        );
        Self { background }
    }
}

pub fn main(mut gba: agb::Gba) -> ! {
    // Get the graphics manager, responsible for all the graphics
    let mut gfx = gba.graphics.get();

    // Set the palettes to the import, otherwise nothing will show (all 0 palette)
    gfx.set_background_palettes(backgrounds::PALETTES);

    // Make the battlefield
    let mut buildings = Buildings::new();
    let mut cursor = Cursor::new();

    // Count frames for animation timing
    let mut frame_count: usize = 0;

    // Get inputs
    let mut input = ButtonController::new();

    loop {
        // get inputs
        input.update();

        // move the cursor
        cursor.update(&input);

        // count the frames
        frame_count += 1;

        // Start a frame
        let mut frame = gfx.frame();

        // Show the bgs
        buildings.show(&mut frame);
        cursor.show(&mut frame, frame_count);

        // Until the call to `frame.commit()`, nothing will be displayed
        frame.commit();
    }
}
