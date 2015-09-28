extern crate egl;
extern crate opengles;
extern crate videocore;

use std::ptr;

use egl::{ EGLConfig,
           EGLContext,
           EGLDisplay,
           EGLNativeDisplayType,
           EGLSurface };

use opengles::glesv2 as gl;

use videocore::bcm_host;
use videocore::dispmanx;
use videocore::dispmanx::{ FlagsAlpha,
                           Transform,
                           VCAlpha,
                           Window };
use videocore::image::Rect;

// -------------------------------------------------------------------------------------------------
// STRUCTS
// -------------------------------------------------------------------------------------------------

pub struct Context {
    pub config:  EGLConfig,
    pub context: EGLContext,
    pub display: EGLDisplay,
    pub surface: EGLSurface
}

// -------------------------------------------------------------------------------------------------
// FUNCTIONS
// -------------------------------------------------------------------------------------------------

fn gl_loop(context: Context) {
    // get screen resolution
    let dimensions = bcm_host::graphics_get_display_size(0).unwrap();

    gl::viewport(0, 0, dimensions.width as i32, dimensions.height as i32);

    loop {
        gl::clear_color(1.0, 0.0, 1.0, 1.0);
        gl::clear(gl::GL_COLOR_BUFFER_BIT);

        egl::swap_buffers(context.display, context.surface);
    }
}

fn init_egl(window: &mut Window) -> Context {

    let context_attr = [egl::EGL_CONTEXT_CLIENT_VERSION, 2,
                        egl::EGL_NONE];

    let config_attr = [egl::EGL_RED_SIZE,     8,
                       egl::EGL_GREEN_SIZE,   8,
                       egl::EGL_BLUE_SIZE,    8,
                       egl::EGL_ALPHA_SIZE,   8,
                       egl::EGL_SURFACE_TYPE, egl::EGL_WINDOW_BIT,
                       egl::EGL_NONE];

    // get display
    let egl_display = match egl::get_display(egl::EGL_DEFAULT_DISPLAY) {
        Some(x) => x,
        None    => panic!("Failed to get EGL display")
    };

    // init display
    if !egl::initialize(egl_display, &mut 0i32, &mut 0i32) {
        panic!("Failed to initialize EGL");
    }

    // choose first available configuration
    let egl_config = match egl::choose_config(egl_display, &config_attr, 1) {
        Some(x) => x,
        None    => panic!("Failed to get EGL configuration")
    };

    // bind opengl es api
    if !egl::bind_api(egl::EGL_OPENGL_ES_API) {
        panic!("Failed to bind EGL OpenGL ES API");
    }

    // create egl context
    let egl_context = match egl::create_context(egl_display, egl_config, egl::EGL_NO_CONTEXT,
                                                &context_attr) {
        Some(c) => c,
        None    => panic!("Failed to create EGL context")
    };

    // create surface
    let egl_surface = match egl::create_window_surface(egl_display, egl_config,
                                                       window as *mut _ as EGLNativeDisplayType,
                                                       &[]) {
        Some(s) => s,
        None    => panic!("Failed to create EGL surface")
    };

    // set current context
    if !egl::make_current(egl_display, egl_surface, egl_surface, egl_context) {
        panic!("Failed to make EGL current context");
    }

    Context{ config:  egl_config,
             context: egl_context,
             display: egl_display,
             surface: egl_surface }
}

fn main() {
    // first thing to do is initialize the broadcom host (when doing any graphics on RPi)
    bcm_host::init();

    // open the display
    let display = dispmanx::display_open(0);

    // get update handle
    let update = dispmanx::update_start(0);

    // get screen resolution (same display number as display_open()
    let dimensions = match bcm_host::graphics_get_display_size(0) {
        Some(x) => x,
        None    => panic!("Must call bcm_host::init() prior to any display operation on RPi")
    };

    println!("Display size: {}x{}", dimensions.width, dimensions.height);

    // setup the destination rectangle where opengl will be drawing
    let mut dest_rect = Rect{ x:      0,
                              y:      0,
                              width:  dimensions.width as i32,
                              height: dimensions.height as i32 };

    // setup the source rectangle where opengl will be drawing
    let mut src_rect = Rect{ x:      0,
                             y:      0,
                             width:  0,
                             height: 0 };

    // draw opengl context on a clean background (cleared by the clear color)
    let mut alpha = VCAlpha{ flags:   FlagsAlpha::FIXED_ALL_PIXELS,
                             opacity: 255,
                             mask:    0 };

    // draw opengl context on top of whatever is running behind it
    // note: changing the layer for the dispmanx element will also adjust where it's drawn, if
    //       there are other graphical applications running

    //let mut alpha = VCAlpha{ flags:   FlagsAlpha::FROM_SOURCE,
    //                         opacity: 0,
    //                         mask:    0 };

    // create our dispmanx element upon which we'll draw opengl using EGL
    let element = dispmanx::element_add(update, display,
                                        3, // layer upon which to draw
                                        &mut dest_rect,
                                        0,
                                        &mut src_rect,
                                        dispmanx::DISPMANX_PROTECTION_NONE,
                                        &mut alpha,
                                        ptr::null_mut(),
                                        Transform::NO_ROTATE);

    // submit changes
    dispmanx::update_submit_sync(update);

    // create window to hold element, width, height
    let mut window = Window{ element: element,
                             width:   dimensions.width as i32,
                             height:  dimensions.height as i32 };

    // init egl
    let context = init_egl(&mut window);

    gl_loop(context);
}