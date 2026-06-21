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

use agb::{display::object::Object, include_aseprite};

include_aseprite!(
    mod shrooms,
    "gfx/shrooms.aseprite"
);
const SHROOM_WALK_SPRITES: &'static [usize] = &[0, 1, 0, 2];

pub fn main(mut gba: agb::Gba) -> ! {
    // Get the graphics manager, responsible for all the graphics
    let mut gfx = gba.graphics.get(); 

    // Create an objects with the sprites
    let mut shroom = Object::new(shrooms::SHROOM.sprite(0));
    let mut shroom_fast: Object = Object::new(shrooms::SHROOM_FAST.sprite(0));

    // Place them at some points on the screen
    shroom.set_pos((48, 48));
    shroom_fast.set_pos((64,64));

    // Count frames for animation timing
    let mut frame_count: u32 = 0;
    let mut shroom_animation_idx = 0;

    loop {
        // count the frames
        frame_count = (frame_count + 1) % 64;

        // Start a frame
        let mut frame = gfx.frame();

        if frame_count % 8 == 0 {
            // Set the object sprites based on the frame count
            shroom_animation_idx = (shroom_animation_idx + 1) % SHROOM_WALK_SPRITES.len();
            shroom.set_sprite(shrooms::SHROOM.sprite(SHROOM_WALK_SPRITES[shroom_animation_idx]));
            shroom_fast.set_sprite(shrooms::SHROOM_FAST.sprite(SHROOM_WALK_SPRITES[shroom_animation_idx]));
            
        }
        if frame_count % 16 == 0 {
            // Move the objects
            shroom.set_pos(shroom.pos() + (1, 0).into());
            shroom_fast.set_pos(shroom_fast.pos() + (1,0).into());
        }

        // Actually show these objects on the screen
        shroom.show(&mut frame);
        shroom_fast.show(&mut frame);

        // Until the call to `frame.commit()`, nothing will be displayed
        frame.commit();
    }
}