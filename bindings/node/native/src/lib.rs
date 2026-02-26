use napi::bindgen_prelude::*;
use napi_derive::napi;

#[napi]
pub fn check_posture_json(policy_file: Option<String>) -> Result<String> {
    let path = policy_file.unwrap_or_else(|| "southpaw.yaml".to_string());
    southpaw_core::check_posture_json_from_path(&path).map_err(|err| {
        Error::from_reason(format!(
            "southpaw check failed: {err}\n\n\
             Troubleshooting:\n\
             - Ensure policy file exists at: {path}\n\
             - Check that it is valid YAML or JSON\n\
             - Run `southpaw check --policy {path}` for detailed diagnostics"
        ))
    })
}
