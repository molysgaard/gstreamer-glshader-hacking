use std::{str::FromStr, time::Duration};

use anyhow::Result;
use gst::prelude::*;
use gstreamer as gst;
use gstreamer_gl as gstgl;

const img_width: usize = 800;
const img_height: usize = 800;

const ball_shader: &'static str = "\
// For GStreamer glshader
#version 100
#ifdef GL_ES
precision mediump float;
#endif
varying vec2 v_texcoord;
uniform sampler2D tex; // Output image, unknown dimesion, [0,1]x[0,1]
uniform float time;
uniform float width; // Output image width pixels
uniform float height; // Output image height pixels
uniform float cx;
uniform float cy;

void main() {
    vec2 c = vec2(cx, cy);
    float r = 0.1;
    vec2 p = vec2(gl_FragCoord.x/width, gl_FragCoord.y/height);

    vec2 err = p-c;
    float errnorm = length(err);

    vec4 color;
    if (errnorm < r) {
        color = vec4(1.0, 1.0, 1.0, 1.0);
    }
    else {
        color = vec4(0.0, 0.0, 0.0, 1.0);
    }
    gl_FragColor = color;
}
";

fn main() -> Result<()> {
    gst::init()?;

    let pipeline = gst::Pipeline::new(None);

    let videotestsrc = gst::ElementFactory::make("videotestsrc").build()?;
    let videoconvert_0 = gst::ElementFactory::make("videoconvert").build()?;
    let glupload = gst::ElementFactory::make("glupload").build()?;
    let glshader = gst::ElementFactory::make("glshader").build()?;
    glshader.set_property("fragment", ball_shader);
    let glimagesink = gst::ElementFactory::make("glimagesink").build()?;

    pipeline.add(&videotestsrc)?;
    pipeline.add(&videoconvert_0)?;
    pipeline.add(&glupload)?;
    pipeline.add(&glshader)?;
    pipeline.add(&glimagesink)?;

    videotestsrc.link(&videoconvert_0)?;
    let caps = gst::Caps::from_str(&format!(
        "video/x-raw,format=RGBA,width={img_width},height={img_height}"
    ))
    .unwrap();
    videoconvert_0.link_filtered(&glupload, &caps)?;
    glupload.link(&glshader)?;
    glshader.link(&glimagesink)?;

    let bus = pipeline.bus().unwrap();
    pipeline.set_state(gst::State::Playing)?;

    // Here we would like to update the position of the ball live, `[cx, cy]`
    for _i in 0..50 {
        if glshader.has_property("shader", None) {
            println!("shader prop type: {:?}", glshader.property_type("shader").unwrap());
            let sh: gstgl::GLShader = glshader.property::<gstgl::GLShader>("shader");
            println!("shader: {:?}", sh);
            sh.set_uniform_1f("cx", 0.5f32);
            sh.set_uniform_1f("cy", 0.5f32);
        }
        std::thread::sleep(Duration::from_secs_f64(0.1));
    }

    println!("Sending EOS");
    pipeline.send_event(gst::event::Eos::new());
    println!("Waiting for EOS");
    bus.timed_pop_filtered(gst::ClockTime::NONE, &[gst::MessageType::Eos]);
    pipeline.set_state(gst::State::Null)?;

    Ok(())
}
