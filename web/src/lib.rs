use std::{cell::RefCell, rc::Rc};

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{JsFuture, spawn_local};
use web_sys::{window, Headers, MediaStreamConstraints, MediaStream, MediaRecorder, RequestInit, BlobEvent, Blob, FormData, MediaDevices, console};
use js_sys::Array;

pub async fn handle_req(events: Blob, id: String) {
    let form = FormData::new().unwrap();
    form.append_with_str("id", &id).unwrap();
    form.append_with_blob("video", &events).unwrap();

    let mut opts = RequestInit::new();
    opts.method("POST");
    opts.body(Some(&form));

    let headers = Headers::new().unwrap();
    headers.set("Content-Type", "multipart/form-data").unwrap();
    opts.headers(&headers);

    let window = window().unwrap();
    let request = window.fetch_with_str_and_init("http://127.0.0.1:4269/video_upload", &opts);
    JsFuture::from(request).await.unwrap();
}

#[wasm_bindgen]
pub struct XWing {
    media_devices: MediaDevices
}


#[wasm_bindgen]
impl XWing {
    #[wasm_bindgen(constructor)]
    pub fn new() -> XWing {
        let window = window().unwrap();
        let navigator = window.navigator();
        let media_devices = navigator.media_devices().unwrap(); 
        XWing { media_devices }
    }

    pub async fn start(&self) {
        let mut constraints = MediaStreamConstraints::new();
        constraints.audio(&JsValue::TRUE);
        constraints.video(&JsValue::TRUE);
        let promise = self.media_devices.get_user_media_with_constraints(&constraints).unwrap();
        let device: JsValue =  JsFuture::from(promise).await.unwrap();
        let stream = MediaStream::from(device);     
        let recorder = Rc::new(RefCell::new(MediaRecorder::new_with_media_stream(&stream).unwrap()));
        recorder.borrow_mut().start_with_time_slice(10).unwrap();
        let recorder_clone = Rc::clone(&recorder);
        let queue = Array::new();
        let mut i = 0;  

        let on_data = move |event: BlobEvent| {
            if event.data().unwrap().size() == 0.0 {
                return;
            } else if i == 100 {
                recorder_clone.borrow_mut().stop().unwrap();
                return
            } else if i % 10 == 0 {
                let buffer_blob = Blob::new_with_blob_sequence(&queue).unwrap();
                spawn_local(handle_req(buffer_blob, String::from(i.to_string())));
                queue.splice(0, queue.length(), &JsValue::UNDEFINED);
            }
            queue.push(&event.data().unwrap());
            i += 1;
        };

        let closure = Closure::<dyn FnMut(BlobEvent)>::new(on_data);
        recorder.borrow_mut().set_ondataavailable(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }
}

