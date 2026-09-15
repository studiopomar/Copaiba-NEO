#![cfg(target_arch = "wasm32")]

use wasm_bindgen::JsCast;
use web_sys::{Blob, HtmlAnchorElement, Url};

pub fn download_text(filename: &str, contents: &str) -> Result<(), String> {
    let array = js_sys::Array::new();
    array.push(&wasm_bindgen::JsValue::from_str(contents));
    let blob = Blob::new_with_str_sequence(&array).map_err(|e| format!("Blob: {e:?}"))?;
    let url = Url::create_object_url_with_blob(&blob).map_err(|e| format!("URL: {e:?}"))?;
    let document = web_sys::window().and_then(|w| w.document()).ok_or("documento indisponível")?;
    let anchor: HtmlAnchorElement = document.create_element("a").map_err(|e| format!("anchor: {e:?}"))?.dyn_into().map_err(|_| "anchor inválido")?;
    anchor.set_href(&url);
    anchor.set_download(filename);
    anchor.click();
    let _ = Url::revoke_object_url(&url);
    Ok(())
}
