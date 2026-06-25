use agb::{
    display::{
        tile_data::TileData,
        tiled::{RegularBackground, TileSetting},
    },
    fixnum::Vector2D,
};

use crate::{BATTLEFIELD_TILE_SIZE, BattlefieldTileType, backgrounds};

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
    tile_width: i32,
    tile_height: i32,
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

pub fn set_battlefield_tile(
    background: &mut RegularBackground,
    pos: Vector2D<i32>,
    tile_type: BattlefieldTileType,
) {
    set_tiles(
        background,
        pos * BATTLEFIELD_TILE_SIZE,
        &backgrounds::BATTLEFIELD,
        tile_type as usize,
        BATTLEFIELD_TILE_SIZE as usize,
        BATTLEFIELD_TILE_SIZE as usize,
    );
}
