use std::collections::HashMap;

use env_as_html_data::{transform_html, HtmlChanged};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = transformHtml)]
pub fn transform_html_for_javascript(html: &str, environment: JsValue) -> Result<String, JsValue> {
    let environment: HashMap<String, String> = serde_wasm_bindgen::from_value(environment)
        .map_err(|error| JsValue::from_str(&format!("INVALID_ENVIRONMENT: {error}")))?;

    match transform_html(&environment, html)
        .map_err(|error| JsValue::from_str(&format!("HTML_TRANSFORM_ERROR: {error}")))?
    {
        HtmlChanged::Yes(html) => Ok(html),
        HtmlChanged::No => Ok(html.to_string()),
    }
}
