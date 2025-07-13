mod format;
#[cfg(test)]
mod test;

use crate::format::{format, FormatResult};
use foro_plugin_utils::compat_util::get_target;
use foro_plugin_utils::data_json_utils::JsonGetter;
use foro_plugin_utils::foro_plugin_setup;
use serde_json::{json, Value};

pub fn main_with_json(input: Value) -> Value {
    let start = std::time::Instant::now();

    let target = match get_target(&input) {
        Ok(t) => t,
        Err(e) => {
            return json!({
                "plugin-panic": format!("Failed to get target: {}", e),
            });
        }
    };

    let target_content = match String::get_value(&input, ["target-content"]) {
        Ok(content) => content,
        Err(e) => {
            return json!({
                "plugin-panic": format!("Failed to get target-content: {}", e),
            });
        }
    };

    let result = match format(target, target_content) {
        Ok(FormatResult::Success { formatted_content }) => {
            json!({
                "format-status": "success",
                "formatted-content": formatted_content,
            })
        }
        Ok(FormatResult::Ignored) => {
            json!({
                "format-status": "ignored",
            })
        }
        Ok(FormatResult::Error { error }) => {
            json!({
                "format-status": "error",
                "format-error": error,
            })
        }
        Err(e) => {
            json!({
                "plugin-panic": e.to_string(),
            })
        }
    };

    println!("time: {:?}", start.elapsed());

    result
}

foro_plugin_setup!(main_with_json);
