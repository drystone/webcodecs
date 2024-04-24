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
