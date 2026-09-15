#![cfg(target_arch = "wasm32")]

use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{Event, FileReader, HtmlInputElement};

thread_local! {
    static QUEUE: RefCell<Vec<(String, Vec<u8>)>> = const { RefCell::new(Vec::new()) };
}

pub fn request_voicebank() {
    let document = web_sys::window().and_then(|w| w.document()).expect("browser document");
    let input: HtmlInputElement = document.create_element("input").unwrap().dyn_into().unwrap();
    input.set_type("file");
    input.set_multiple(true);
    input.set_attribute("webkitdirectory", "").unwrap();
    input.set_attribute("directory", "").unwrap();

    let change = Closure::wrap(Box::new(move |event: Event| {
        let input: HtmlInputElement = event.target().unwrap().dyn_into().unwrap();
        let files = input.files().unwrap();
        let total = files.length();
        let remaining = Rc::new(RefCell::new(total));
        for index in 0..total {
            let file = files.get(index).unwrap();
            let name = js_sys::Reflect::get(file.as_ref(), &"webkitRelativePath".into())
                .ok().and_then(|v| v.as_string()).filter(|v| !v.is_empty())
                .unwrap_or_else(|| file.name());
            let reader = FileReader::new().unwrap();
            let reader_clone = reader.clone();
            let name_clone = name.clone();
            let remaining_clone = remaining.clone();
            let onload = Closure::once(Box::new(move |_event: Event| {
                if let Ok(result) = reader_clone.result() {
                    let bytes = js_sys::Uint8Array::new(&result).to_vec();
                    QUEUE.with(|queue| queue.borrow_mut().push((name_clone, bytes)));
                }
                *remaining_clone.borrow_mut() -= 1;
            }) as Box<dyn FnOnce(_)>);
            reader.set_onload(Some(onload.as_ref().unchecked_ref()));
            onload.forget();
            reader.read_as_array_buffer(&file).unwrap();
        }
    }) as Box<dyn FnMut(_)>);
    input.set_onchange(Some(change.as_ref().unchecked_ref()));
    change.forget();
    input.click();
}

pub fn take_files() -> Vec<(String, Vec<u8>)> {
    QUEUE.with(|queue| std::mem::take(&mut *queue.borrow_mut()))
}
