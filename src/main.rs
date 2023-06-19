use std::{str::FromStr, time::Duration};

use anyhow::Result;
use gst::prelude::*;
use gstreamer as gst;
use gstreamer_gl as gstgl;

const IMG_WIDTH: usize = 800;
const IMG_HEIGHT: usize = 800;

const BALL_SHADER: &'static str = "\
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
    //vec2 c = (vec2(cos(time), sin(time))+1.0) / 2.0;
    vec2 c = vec2(cx,cy);
    float r = 0.1;
    vec2 p = vec2(gl_FragCoord.x/width, gl_FragCoord.y/height);

    vec2 err = p-c;
    float errnorm = length(err);

    vec4 color;
    if (errnorm < r) {
        color = vec4(1.0, 1.0, 1.0, 1.0);
    }
    else {
        color = vec4(0.2, 0.2, 0.2, 1.0);
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
    glshader.set_property("fragment", BALL_SHADER);
    let glimagesink = gst::ElementFactory::make("glimagesink").build()?;

    pipeline.add(&videotestsrc)?;
    pipeline.add(&videoconvert_0)?;
    pipeline.add(&glupload)?;
    pipeline.add(&glshader)?;
    pipeline.add(&glimagesink)?;

    videotestsrc.link(&videoconvert_0)?;
    let caps = gst::Caps::from_str(&format!(
        "video/x-raw,format=RGBA,width={IMG_WIDTH},height={IMG_HEIGHT}"
    ))
    .unwrap();
    videoconvert_0.link_filtered(&glupload, &caps)?;
    glupload.link(&glshader)?;
    glshader.link(&glimagesink)?;

    let bus = pipeline.bus().unwrap();
    pipeline.set_state(gst::State::Playing)?;

    // Here we would like to update the position of the ball live, `[cx, cy]`
    for i in 0..50 {
        let time = 0.1 * i as f32;
        let sh: Option<gstgl::GLShader> = glshader.property("shader");
        if let Some(sh) = sh {
            println!("shader: {:?}", sh);
            let x = time.cos();
            let y = time.sin();
            sh.set_uniform_1f("cx", x);
            sh.set_uniform_1f("cy", y);
            glshader.set_property("update-shader", true);
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
