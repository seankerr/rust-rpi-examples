extern crate videocore;

use videocore::bcm_host;
use videocore::dispmanx;

fn main() {
    // first thing to do is initialize the broadcom host (when doing any graphics on RPi)
    bcm_host::init();

    // open the display
    let display = dispmanx::display_open(0);

    // get update handle
    let update = dispmanx::update_start(0);

    // change background color
    dispmanx::display_set_background(update, display,
                                     255, // red
                                     0,   // green
                                     0);  // blue

    // submit changes
    dispmanx::update_submit_sync(update);
}