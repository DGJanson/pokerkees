// LLM GENERATED CODE FOR
// AZURE DEVOPS INTEGRATION

use serde::Deserialize;
use std::process::Command;
use std::time::Instant;

use crate::state::AzureWorkItem;

/// simple struct to "catch"
/// the output of the cli call
#[derive(Debug, Deserialize)]
struct AzWorkItem {
    fields: serde_json::Value,
}

// Helper function to handle json response
fn parse_json_robust(bytes: &[u8]) -> Result<AzWorkItem, String> {
    // strip UTF-8 BOM if present
    let mut start = 0;
    if bytes.len() >= 3 && bytes[0] == 0xEF && bytes[1] == 0xBB && bytes[2] == 0xBF {
        start = 3;
    }
    let slice = &bytes[start..];

    // Try as UTF-8 bytes first
    match serde_json::from_slice::<AzWorkItem>(slice) {
        Ok(v) => Ok(v),
        Err(e_utf8) => {
            // Fallback: try decoding via Windows-1252 then parse from string.
            let (cow, _, _) = encoding_rs::WINDOWS_1252.decode(slice);
            match serde_json::from_str::<AzWorkItem>(&cow) {
                Ok(v) => Ok(v),
                Err(e_cp1252) => {
                    // Help yourself by logging a snippet of the payload
                    let preview_len = slice.len().min(400);
                    let preview = String::from_utf8_lossy(&slice[..preview_len]);
                    Err(format!(
                        "Failed parsing az json (utf8 err: {e_utf8}; cp1252 err: {e_cp1252}). Preview (first {preview_len} bytes): {preview}"
                    ))
                }
            }
        }
    }
}

/// Perform the az cli call to get info on work item
///
pub fn fetch_azure_item(org: &str, id: usize) -> Result<AzureWorkItem, String> {
    let mut cmd = if cfg!(target_os = "windows") {
        let mut c = Command::new("cmd");
        c.arg("/C").arg("az");
        c
    } else {
        Command::new("az")
    };

    // Force clean, UTF-8, no color/noise output
    cmd.env("PYTHONIOENCODING", "utf-8")
        .env("AZURE_CORE_NO_COLOR", "1");

    cmd.args([
        "boards",
        "work-item",
        "show",
        "--id",
        &id.to_string(),
        "--org",
        org,
        "--expand",
        "none",
        "--fields",
        "System.Id,System.Title,System.State,System.AssignedTo,System.Tags,System.Description,Microsoft.VSTS.Common.AcceptanceCriteria,System.CreatedDate",
        "-o",
        "json",
    ]);

    let output = cmd.output().map_err(|e| format!("Failed to run az: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("az error: {stderr}"));
    }

    let wi = parse_json_robust(&output.stdout)?;

    let fields = wi.fields.as_object().ok_or("unexpected fields shape")?;

    let title = fields
        .get("System.Title")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();

    let state = fields
        .get("System.State")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();

    let assigned_to = fields
        .get("System.AssignedTo")
        .and_then(|v| v.get("displayName"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let tags = fields
        .get("System.Tags")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .split(';')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();

    // TODO REMOVE HTML?
    let description_html = fields
        .get("System.Description")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();

    // TODO REMOVE HTML?
    let acceptance_html = fields
        .get("Microsoft.VSTS.Common.AcceptanceCriteria")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();

    let created_date = fields
        .get("System.CreatedDate")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();

    // Convert html to text
    let description = html2text::from_read(description_html.as_bytes(), 120)
        .unwrap_or("COULD NOT PARSE DESCRIPTION HTML".to_string());
    let acceptance_criteria = html2text::from_read(acceptance_html.as_bytes(), 120)
        .unwrap_or("COULD NOT PARSE ACCEPTANCE HTML".to_string());

    Ok(AzureWorkItem {
        id,
        title,
        state,
        assigned_to,
        tags,
        description,
        acceptance_criteria,
        created_date,
        last_fetched_at: Instant::now(),
    })
}
