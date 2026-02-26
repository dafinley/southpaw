use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn check_posture_json_from_policy_text(
    policy_text: String,
    format_hint: Option<String>,
) -> Result<String, JsValue> {
    let hint_ref = format_hint.as_deref();
    southpaw_core::check_posture_json_from_policy_str(&policy_text, hint_ref).map_err(|err| {
        let hint_msg = match hint_ref {
            Some(h) => format!("format_hint: {h}"),
            None => "format_hint: not provided (auto-detecting from content)".to_string(),
        };
        JsValue::from_str(&format!(
            "southpaw check failed: {err}\n\n\
                 Troubleshooting:\n\
                 - {hint_msg}\n\
                 - Ensure policy text is valid YAML or JSON\n\
                 - Try passing format_hint explicitly as 'yaml' or 'json'"
        ))
    })
}
