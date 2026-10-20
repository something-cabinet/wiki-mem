use std::io::Read;
use std::path::Path;

const ACTION_UPDATE: &str = "update";
const STDIN_READ_ERROR: &str = "Failed to read stdin";
const STDIN_JSON_ERROR: &str = "Invalid JSON on stdin";
const STDIN_OBJECT_ERROR: &str = "Invalid JSON on stdin: expected a JSON object";
const CONTENT_FIELD: &str = "content";
const CONTENT_SOURCE_CONFLICT: &str =
    "Provide the page body either as `content` on stdin or with --content-file <path>, not both";
const CONTENT_FILE_READ_ERROR: &str = "Failed to read --content-file";

pub fn read_stdin() -> Result<String, anyhow::Error> {
    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .map_err(|error| anyhow::anyhow!("{STDIN_READ_ERROR}: {error}"))?;
    Ok(input)
}

pub fn read_update_payload(
    content_file: Option<&Path>,
) -> Result<serde_json::Value, anyhow::Error> {
    let input = read_stdin()?;
    let mut args = match content_file {
        Some(_) if input.trim().is_empty() => serde_json::json!({}),
        _ => parse_update_payload(&input)?,
    };
    apply_content_file(&mut args, content_file)?;
    Ok(args)
}

pub fn attach_update_identity(args: &mut serde_json::Value, id: &str) {
    let Some(object) = args.as_object_mut() else {
        return;
    };
    object.insert("action".into(), serde_json::json!(ACTION_UPDATE));
    object.insert("id".into(), serde_json::json!(id));
}

fn apply_content_file(
    args: &mut serde_json::Value,
    content_file: Option<&Path>,
) -> Result<(), anyhow::Error> {
    let Some(path) = content_file else {
        return Ok(());
    };
    let Some(object) = args.as_object_mut() else {
        return Err(anyhow::anyhow!(STDIN_OBJECT_ERROR));
    };
    if object.contains_key(CONTENT_FIELD) {
        return Err(anyhow::anyhow!(CONTENT_SOURCE_CONFLICT));
    }
    let body = read_content_file(path)?;
    object.insert(CONTENT_FIELD.to_string(), serde_json::Value::String(body));
    Ok(())
}

fn read_content_file(path: &Path) -> Result<String, anyhow::Error> {
    std::fs::read_to_string(path)
        .map_err(|error| anyhow::anyhow!("{CONTENT_FILE_READ_ERROR} {}: {error}", path.display()))
}

fn parse_update_payload(input: &str) -> Result<serde_json::Value, anyhow::Error> {
    serde_json::from_str(input).map_err(|error| anyhow::anyhow!("{STDIN_JSON_ERROR}: {error}"))
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    const FILE_BODY: &str = "schema_version: 1\nstate: |-\n  Body from file.\n";
    const STDIN_BODY: &str = "Body from stdin.";

    fn body_file() -> tempfile::NamedTempFile {
        let mut file = tempfile::NamedTempFile::new().expect("create body file");
        file.write_all(FILE_BODY.as_bytes()).expect("write body");
        file
    }

    #[test]
    fn content_file_supplies_body() {
        let file = body_file();
        let mut args = serde_json::json!({ "status": "draft" });
        apply_content_file(&mut args, Some(file.path())).expect("content file must apply");
        assert_eq!(args[CONTENT_FIELD].as_str(), Some(FILE_BODY));
        assert_eq!(args["status"].as_str(), Some("draft"));
    }

    #[test]
    fn content_file_and_stdin_content_are_rejected() {
        let file = body_file();
        let mut args = serde_json::json!({ "content": STDIN_BODY });
        let error = apply_content_file(&mut args, Some(file.path()))
            .expect_err("two content sources must be rejected");
        assert!(error.to_string().contains("not both"), "{error}");
        assert_eq!(args[CONTENT_FIELD].as_str(), Some(STDIN_BODY));
    }

    #[test]
    fn unreadable_content_file_is_actionable() {
        let missing = std::path::PathBuf::from("does-not-exist-body.md");
        let mut args = serde_json::json!({});
        let error = apply_content_file(&mut args, Some(&missing))
            .expect_err("missing content file must be rejected");
        assert!(error.to_string().contains("--content-file"), "{error}");
        assert!(
            error.to_string().contains("does-not-exist-body.md"),
            "{error}"
        );
    }
}
