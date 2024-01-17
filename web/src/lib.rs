use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{JsFuture, spawn_local};
use web_sys::{window, Headers, MediaStreamConstraints, MediaStream, MediaRecorder, RequestInit, BlobEvent, Blob, FormData};
use js_sys::Array;

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
pub async fn media_init() {
    // let mut queue = MediaQueue::new();
    // let queue_clone = queue.clone();
    let recorder = get_recorder().await;
    let recorder_clone = recorder.clone();
    recorder.start_with_time_slice(10).unwrap();

    let queue = Array::new();
    let mut i = 0;  
    let on_data = move |event: BlobEvent| {
        if event.data().unwrap().size() == 0.0 {
            return;
        } else if i == 100 {
            recorder_clone.stop().unwrap();
            return
        } else if i % 10 == 0 {
            let buffer_blob = Blob::new_with_buffer_source_sequence(&queue).unwrap();
            spawn_local(handle_req(buffer_blob, String::from(i.to_string())));
            queue.splice(0, queue.length(), &JsValue::UNDEFINED);
        }
        queue.push(&event.data().unwrap());
        i += 1;
    };

    let closure = Closure::<dyn FnMut(BlobEvent)>::new(on_data);
    recorder.set_ondataavailable(Some(closure.as_ref().unchecked_ref()));
    closure.forget();

}
