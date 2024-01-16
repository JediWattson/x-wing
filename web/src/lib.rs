use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{JsFuture, spawn_local};
use web_sys::{window, MediaStreamConstraints, MediaStream, MediaRecorder, RequestInit, console, BlobEvent};
use js_sys::{Array, JSON, Object};

#[wasm_bindgen]
pub async fn get_recorder() -> MediaRecorder {
    let mut constraints = MediaStreamConstraints::new();
    constraints.audio(&JsValue::TRUE);
    constraints.video(&JsValue::TRUE);
    let window = window().unwrap();
    let navigator = window.navigator();
    let media_devices = navigator.media_devices().unwrap(); 
    let promise = media_devices.get_user_media_with_constraints(&constraints).unwrap();
    let stream = MediaStream::from(JsFuture::from(promise).await.unwrap());    
    MediaRecorder::new_with_media_stream(&stream).unwrap()
}

#[wasm_bindgen]
pub async fn handle_req(ev_arr: Array) {
    let body_str =  JSON::stringify(&ev_arr).unwrap();
    let mut opts = RequestInit::new();
    opts.method("POST");
    opts.body(Some(&body_str));

    let window = window().unwrap();
    let request = window.fetch_with_str_and_init("http://127.0.0.1:4269/video_upload", &opts);
    JsFuture::from(request).await.unwrap();
}

struct MediaQueue {
    queue: Vec<BlobEvent>,
}

impl MediaQueue {
    fn new() -> MediaQueue {
        MediaQueue {
            queue: Vec::new(),
        }
    }

    fn push(&mut self, event: BlobEvent) {
        self.queue.push(event);
    }

    fn flush(&mut self) {
        if self.queue.len() == 0 {
            return;
        } 
        let ev_arr = Array::new();
        for i in self.queue.iter() {
            ev_arr.push(&i);
        }

        spawn_local(handle_req(ev_arr));
    }
}


#[wasm_bindgen]
pub async fn media_init() {
    let mut queue = MediaQueue::new();
    console::log_1(&"Recording init".into());
    
    let recorder = get_recorder().await;
    recorder.start_with_time_slice(10).unwrap();
    console::log_1(&"Recording started".into());

    let mut i = 0;
    let recorder_clone = recorder.clone();
    let closure = Closure::<dyn FnMut(BlobEvent)>::new(
        move |event: BlobEvent| {
            if event.data().unwrap().size() == 0.0 {
                return;
            } else if i == 100 {
                recorder_clone.stop().unwrap();
                console::log_1(&"Recording stopped".into());
                return
            } else if i % 10 == 0 {
                queue.flush();
            }

            queue.push(event);
            i += 1;       
        }
    );

    recorder.set_ondataavailable(Some(closure.as_ref().unchecked_ref()));
    closure.forget();

}
