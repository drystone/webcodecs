use wasm_bindgen::prelude::*;

const VIDEO: &[u8] = include_bytes!("../hevc");

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn get_frames() -> Vec<js_sys::Uint8Array> {
    let frames = split_video(VIDEO);

    frames.into_iter().map(|f| f.into()).collect()
}

#[wasm_bindgen]
pub fn run_wasm() {
    let canvas = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .get_element_by_id("canvas")
        .unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .unwrap();

    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();

    let output_callback = {
        Closure::<dyn Fn(_)>::new(move |frame: web_sys::VideoFrame| {
            let _ = context.draw_image_with_video_frame_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                &frame,
                0.0,
                0.0,
                frame.coded_width() as f64,
                frame.coded_height() as f64,
                0.0,
                0.0,
                canvas.width() as f64,
                canvas.height() as f64,
            );
            frame.close();
        })
    };

    let error_callback = Closure::<dyn Fn(_)>::new(move |e: web_sys::ErrorEvent| {
        console_log!("decoder error: {e:?}");
    });

    let init = web_sys::VideoDecoderInit::new(
        error_callback.as_ref().unchecked_ref(),
        output_callback.as_ref().unchecked_ref(),
    );
    output_callback.forget();
    error_callback.forget();

    let decoder = web_sys::VideoDecoder::new(&init).unwrap();
    let mut config = web_sys::VideoDecoderConfig::new("hev1.1.2.L153.90");
    config.hardware_acceleration(web_sys::HardwareAcceleration::PreferHardware);
    decoder.configure(&config);

    let frames = get_frames();
    let mut frame_num = 0;
    let timer_callback = {
        Closure::<dyn FnMut()>::new(move || {
            let frame_type = if frame_num % frames.len() == 0 {
                web_sys::EncodedVideoChunkType::Key
            } else {
                web_sys::EncodedVideoChunkType::Delta
            };
            let frame_init = web_sys::EncodedVideoChunkInit::new(
                &frames[frame_num % frames.len()],
                0.0,
                frame_type,
            );
            let frame = web_sys::EncodedVideoChunk::new(&frame_init).unwrap();

            decoder.decode(&frame);
            frame_num += 1;
        })
    };
    let _ = web_sys::window()
        .unwrap()
        .set_interval_with_callback_and_timeout_and_arguments_0(
            timer_callback.as_ref().unchecked_ref(),
            50,
        );
    timer_callback.forget();
}

#[wasm_bindgen]
pub fn run_wasm_gl() {
    let canvas = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .get_element_by_id("canvas")
        .unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .unwrap();

    let context = canvas
        .get_context("webgl2")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::WebGl2RenderingContext>()
        .unwrap();

    prepare_gl_context(&context).unwrap();

    let output_callback = {
        Closure::<dyn Fn(_)>::new(move |frame: web_sys::VideoFrame| {
            context.viewport(
                -(canvas.width() as i32),
                -(canvas.height() as i32),
                canvas.width() as i32 * 2,
                canvas.height() as i32 * 2,
            );

            context
                .tex_image_2d_with_u32_and_u32_and_video_frame(
                    web_sys::WebGl2RenderingContext::TEXTURE_2D,
                    0,
                    web_sys::WebGl2RenderingContext::RGBA as i32,
                    web_sys::WebGl2RenderingContext::RGBA,
                    web_sys::WebGl2RenderingContext::UNSIGNED_BYTE,
                    &frame,
                )
                .unwrap();

            context.draw_arrays(web_sys::WebGl2RenderingContext::TRIANGLES, 0, 6);

            frame.close();
        })
    };

    let error_callback = Closure::<dyn Fn(_)>::new(move |e: web_sys::ErrorEvent| {
        console_log!("decoder error: {e:?}");
    });

    let init = web_sys::VideoDecoderInit::new(
        error_callback.as_ref().unchecked_ref(),
        output_callback.as_ref().unchecked_ref(),
    );
    output_callback.forget();
    error_callback.forget();

    let decoder = web_sys::VideoDecoder::new(&init).unwrap();
    let mut config = web_sys::VideoDecoderConfig::new("hev1.1.2.L153.90");
    config.hardware_acceleration(web_sys::HardwareAcceleration::PreferHardware);
    decoder.configure(&config);

    let frames = get_frames();
    let mut frame_num = 0;
    let timer_callback = {
        Closure::<dyn FnMut()>::new(move || {
            let frame_type = if frame_num % frames.len() == 0 {
                web_sys::EncodedVideoChunkType::Key
            } else {
                web_sys::EncodedVideoChunkType::Delta
            };
            let frame_init = web_sys::EncodedVideoChunkInit::new(
                &frames[frame_num % frames.len()],
                0.0,
                frame_type,
            );
            let frame = web_sys::EncodedVideoChunk::new(&frame_init).unwrap();

            decoder.decode(&frame);
            frame_num += 1;
        })
    };
    let _ = web_sys::window()
        .unwrap()
        .set_interval_with_callback_and_timeout_and_arguments_0(
            timer_callback.as_ref().unchecked_ref(),
            50,
        );
    timer_callback.forget();
}

#[wasm_bindgen]
pub fn test_wasm_vr() -> js_sys::Promise {
    web_sys::window()
        .unwrap()
        .navigator()
        .xr()
        .is_session_supported(web_sys::XrSessionMode::ImmersiveVr)
}

#[wasm_bindgen]
pub async fn run_wasm_vr() {
    let canvas = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .get_element_by_id("canvas")
        .unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .unwrap();

    let context = canvas
        .get_context("webgl2")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::WebGl2RenderingContext>()
        .unwrap();

    prepare_gl_context(&context).unwrap();

    if let Err(e) = {
        let promise = context.make_xr_compatible();
        wasm_bindgen_futures::JsFuture::from(promise).await
    } {
        console_log!("not xr compatible: {e:?}");
        return;
    }

    let session: web_sys::XrSession = {
        let mut options = web_sys::XrSessionInit::new();
        options.required_features(
            &[JsValue::from_str("local-floor")]
                .iter()
                .collect::<js_sys::Array>(),
        );
        let promise = web_sys::window()
            .unwrap()
            .navigator()
            .xr()
            .request_session_with_options(web_sys::XrSessionMode::ImmersiveVr, &options);

        match wasm_bindgen_futures::JsFuture::from(promise).await {
            Ok(session) => session.dyn_into::<web_sys::XrSession>().unwrap(),
            Err(e) => {
                console_log!("request_session: {e:?}");
                return;
            }
        }
    };

    let gl_layer = web_sys::XrWebGlLayer::new_with_web_gl2_rendering_context(&session, &context)
        .map_err(|e| console_log!("gl_layer: {e:?}"))
        .unwrap();
    let mut render_state = web_sys::XrRenderStateInit::new();
    render_state.base_layer(Some(&gl_layer));
    session.update_render_state_with_state(&render_state);
    context.bind_framebuffer(
        web_sys::WebGl2RenderingContext::FRAMEBUFFER,
        gl_layer.framebuffer().as_ref(),
    );

    let output_callback = {
        let width = canvas.width() as i32;
        let height = canvas.height() as i32;
        Closure::<dyn Fn(_)>::new(move |frame: web_sys::VideoFrame| {
            console_log!("drawing frame");
            context.viewport(-width, -height, width * 2, height * 2);

            context
                .tex_image_2d_with_u32_and_u32_and_video_frame(
                    web_sys::WebGl2RenderingContext::TEXTURE_2D,
                    0,
                    web_sys::WebGl2RenderingContext::RGBA as i32,
                    web_sys::WebGl2RenderingContext::RGBA,
                    web_sys::WebGl2RenderingContext::UNSIGNED_BYTE,
                    &frame,
                )
                .unwrap();

            context.draw_arrays(web_sys::WebGl2RenderingContext::TRIANGLES, 0, 6);

            frame.close();
        })
    };

    let error_callback = Closure::<dyn Fn(_)>::new(move |e: web_sys::ErrorEvent| {
        console_log!("decoder error: {e:?}");
    });

    let init = web_sys::VideoDecoderInit::new(
        error_callback.as_ref().unchecked_ref(),
        output_callback.as_ref().unchecked_ref(),
    );
    let decoder = web_sys::VideoDecoder::new(&init).unwrap();
    let mut config = web_sys::VideoDecoderConfig::new("hev1.1.2.L153.90");
    config.hardware_acceleration(web_sys::HardwareAcceleration::PreferHardware);
    decoder.configure(&config);

    output_callback.forget();
    error_callback.forget();

    console_log!("decoder is {:?}", decoder.state());
    {
        let promise = js_sys::Promise::new(&mut |resolve, _| {
            web_sys::window()
                .unwrap()
                .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 1000)
                .unwrap();
        });
        let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
    }
    console_log!("decoder is {:?}", decoder.state());

    let frames = get_frames();
    let mut frame_num = 0;
    let timer_callback = {
        Closure::<dyn FnMut()>::new(move || {
            let frame_type = if frame_num % frames.len() == 0 {
                web_sys::EncodedVideoChunkType::Key
            } else {
                web_sys::EncodedVideoChunkType::Delta
            };
            let frame_init = web_sys::EncodedVideoChunkInit::new(
                &frames[frame_num % frames.len()],
                0.0,
                frame_type,
            );
            let frame = web_sys::EncodedVideoChunk::new(&frame_init).unwrap();

            decoder.decode(&frame);
            frame_num += 1;
        })
    };
    let _ = web_sys::window()
        .unwrap()
        .set_interval_with_callback_and_timeout_and_arguments_0(
            timer_callback.as_ref().unchecked_ref(),
            50,
        );
    timer_callback.forget();
}

fn split_video(video: &'static [u8]) -> Vec<&'static [u8]> {
    let nalu_offsets = video
        .windows(4)
        .enumerate()
        .filter_map(|(i, bs)| (bs == [0x00, 0x00, 0x00, 0x01]).then_some(i))
        .collect::<Vec<_>>();

    let chunk_offsets = nalu_offsets
        .split_inclusive(|offset| video[offset + 4] == 2 || video[offset + 4] == 38)
        .map(|nal_offsets| nal_offsets[0])
        .collect::<Vec<_>>();

    chunk_offsets
        .iter()
        .zip(chunk_offsets[1..].iter().chain(Some(video.len()).iter()))
        .map(|(s, e)| &video[*s..*e])
        .collect::<Vec<_>>()
}

fn prepare_gl_context(context: &web_sys::WebGl2RenderingContext) -> Result<(), String> {
    let vert_shader = compile_shader(
        context,
        web_sys::WebGl2RenderingContext::VERTEX_SHADER,
        r##"#version 300 es
        in vec4 position;
        out vec2 v_uv;

        void main() {
            gl_Position = position;
            v_uv = position.xy;
        }
        "##,
    )?;

    let frag_shader = compile_shader(
        context,
        web_sys::WebGl2RenderingContext::FRAGMENT_SHADER,
        r##"#version 300 es
        precision mediump float;

        uniform sampler2D u_texture;
        in vec2 v_uv;
        out vec4 outColor;

        void main() {
            outColor = texture(u_texture, vec2(v_uv.x, 1.0 - v_uv.y));
        }
        "##,
    )?;
    let program = link_program(context, &vert_shader, &frag_shader)?;
    context.use_program(Some(&program));

    let vertices: [f32; 12] = [0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0, 1.0];

    let buffer = context.create_buffer().ok_or("failed to create buffer")?;
    context.bind_buffer(web_sys::WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));

    // Ensuring no memory allocations before `Float32Array::view` is dropped.
    unsafe {
        let positions_array_buf_view = js_sys::Float32Array::view(&vertices);
        context.buffer_data_with_array_buffer_view(
            web_sys::WebGl2RenderingContext::ARRAY_BUFFER,
            &positions_array_buf_view,
            web_sys::WebGl2RenderingContext::STATIC_DRAW,
        );
    }

    let vao = context
        .create_vertex_array()
        .ok_or("Could not create vertex array object")?;
    context.bind_vertex_array(Some(&vao));

    let position_attribute_location = context.get_attrib_location(&program, "position");
    context.vertex_attrib_pointer_with_i32(
        position_attribute_location as u32,
        2,
        web_sys::WebGl2RenderingContext::FLOAT,
        false,
        0,
        0,
    );
    context.enable_vertex_attrib_array(position_attribute_location as u32);
    context.bind_vertex_array(Some(&vao));

    let texture = context.create_texture().unwrap();
    context.bind_texture(web_sys::WebGl2RenderingContext::TEXTURE_2D, Some(&texture));

    let texture_location = context.get_uniform_location(&program, "u_texture").unwrap();
    context.uniform1i(Some(&texture_location), 0);

    context.tex_parameteri(
        web_sys::WebGl2RenderingContext::TEXTURE_2D,
        web_sys::WebGl2RenderingContext::TEXTURE_WRAP_S,
        web_sys::WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
    );
    context.tex_parameteri(
        web_sys::WebGl2RenderingContext::TEXTURE_2D,
        web_sys::WebGl2RenderingContext::TEXTURE_WRAP_T,
        web_sys::WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
    );
    context.tex_parameteri(
        web_sys::WebGl2RenderingContext::TEXTURE_2D,
        web_sys::WebGl2RenderingContext::TEXTURE_MIN_FILTER,
        web_sys::WebGl2RenderingContext::LINEAR as i32,
    );
    Ok(())
}

fn compile_shader(
    context: &web_sys::WebGl2RenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<web_sys::WebGlShader, String> {
    let shader = context
        .create_shader(shader_type)
        .ok_or_else(|| String::from("failed to create shader"))?;
    context.shader_source(&shader, source);
    context.compile_shader(&shader);

    if context
        .get_shader_parameter(&shader, web_sys::WebGl2RenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(context
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| String::from("failed to create shader")))
    }
}

fn link_program(
    context: &web_sys::WebGl2RenderingContext,
    vert_shader: &web_sys::WebGlShader,
    frag_shader: &web_sys::WebGlShader,
) -> Result<web_sys::WebGlProgram, String> {
    let program = context
        .create_program()
        .ok_or_else(|| String::from("failed to create program"))?;

    context.attach_shader(&program, vert_shader);
    context.attach_shader(&program, frag_shader);
    context.link_program(&program);

    if context
        .get_program_parameter(&program, web_sys::WebGl2RenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(context
            .get_program_info_log(&program)
            .unwrap_or_else(|| String::from("failed to create program")))
    }
}

#[cfg(test)]
mod tests {
    const VIDEO: &[u8] = &[0, 0, 0, 1, 25, 0, 0, 0, 1, 38, 0, 0, 0, 1, 2];
    #[test]
    fn test_split_video() {
        let frames = super::split_video(VIDEO);
        assert_eq!(2, frames.len());
        assert_eq!(&[0, 0, 0, 1, 25, 0, 0, 0, 1, 38], frames[0]);
        assert_eq!(&[0, 0, 0, 1, 2], frames[1]);
    }
}
