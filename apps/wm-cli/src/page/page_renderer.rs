use wm_core::page::RecordMigrationPlan;

const CONTENT_LIMIT: usize = 500;
const STATUS_DELETED: &str = "deleted";
const STATUS_UPDATED: &str = "updated";
const STATUS_LINKED: &str = "linked";
const STATUS_UNLINKED: &str = "unlinked";
const GET_FIELD_ID: &str = "id";
const GET_FIELD_TITLE: &str = "title";
const GET_FIELD_CONTENT: &str = "content";
const GET_FIELD_TAGS: &str = "tags";
const GET_FIELD_TYPE: &str = "type";
const GET_DETAIL_FIELDS: [&str; 4] = [
    GET_FIELD_TITLE,
    GET_FIELD_CONTENT,
    GET_FIELD_TAGS,
    GET_FIELD_TYPE,
];

pub fn print_page_get(resp: &serde_json::Value, id: &str, json: bool) -> Result<(), anyhow::Error> {
    if json {
        return print_page_get_json(resp, id);
    }
    let content = resp[GET_FIELD_CONTENT].as_str().unwrap_or("");
    print_page_get_text(id, resp[GET_FIELD_TITLE].as_str(), content);
    Ok(())
}

fn print_page_get_json(resp: &serde_json::Value, id: &str) -> Result<(), anyhow::Error> {
    let mut payload = serde_json::Map::new();
    payload.insert(
        GET_FIELD_ID.to_string(),
        serde_json::Value::String(id.to_string()),
    );
    for field in GET_DETAIL_FIELDS {
        if let Some(value) = resp.get(field) {
            payload.insert(field.to_string(), value.clone());
        }
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::Value::Object(payload))?
    );
    Ok(())
}

fn print_page_get_text(id: &str, title: Option<&str>, content: &str) {
    println!("--- {} ---", id);
    if let Some(title) = title {
        println!("{}", title);
    }
    println!("{}", truncate_content(content));
}

fn truncate_content(content: &str) -> &str {
    if content.len() > CONTENT_LIMIT {
        return &content[..CONTENT_LIMIT];
    }
    content
}

pub fn print_page_list(resp: &serde_json::Value, json: bool) -> Result<(), anyhow::Error> {
    let pages = resp["pages"].as_array().cloned().unwrap_or_default();
    if json {
        return print_page_list_json(&pages);
    }
    print_page_list_text(&pages);
    Ok(())
}

fn print_page_list_json(pages: &[serde_json::Value]) -> Result<(), anyhow::Error> {
    let payload = serde_json::json!({ "pages": pages, "total": pages.len() });
    println!("{}", serde_json::to_string_pretty(&payload)?);
    Ok(())
}

fn print_page_list_text(pages: &[serde_json::Value]) {
    for page in pages {
        println!(
            "  {}  [{}]",
            page["id"],
            page["type"].as_str().unwrap_or("")
        );
    }
    println!("{} pages", pages.len());
}

pub fn print_page_created(
    resp: &serde_json::Value,
    path: &str,
    fallback_type: &str,
    json: bool,
) -> Result<(), anyhow::Error> {
    let id = resp["id"].as_str().unwrap_or("").to_string();
    let resp_type = resp["type"].as_str().unwrap_or(fallback_type).to_string();
    if json {
        return print_page_created_json(&id, path, &resp_type);
    }
    println!("Created page: {} ({})", id, resp_type);
    Ok(())
}

fn print_page_created_json(id: &str, path: &str, page_type: &str) -> Result<(), anyhow::Error> {
    let payload = serde_json::json!({ "id": id, "path": path, "type": page_type });
    println!("{}", serde_json::to_string_pretty(&payload)?);
    Ok(())
}

pub fn print_page_deleted(
    resp: &serde_json::Value,
    id: &str,
    json: bool,
) -> Result<(), anyhow::Error> {
    let status = resp["status"].as_str().unwrap_or(STATUS_DELETED);
    if json {
        let payload = serde_json::json!({ "id": id, "status": status });
        println!("{}", serde_json::to_string_pretty(&payload)?);
        return Ok(());
    }
    println!("Deleted page: {}", id);
    Ok(())
}

pub fn print_page_updated(id: &str, json: bool) {
    if json {
        let payload = serde_json::json!({ "id": id, "status": STATUS_UPDATED });
        let rendered = serde_json::to_string_pretty(&payload).unwrap_or_default();
        println!("{}", rendered);
        return;
    }
    println!("Updated: {}", id);
}

pub fn print_page_linked(
    id: &str,
    target: &str,
    edge: &str,
    json: bool,
) -> Result<(), anyhow::Error> {
    if json {
        let payload = serde_json::json!({
            "id": id,
            "target": target,
            "type": edge,
            "status": STATUS_LINKED,
        });
        println!("{}", serde_json::to_string_pretty(&payload)?);
        return Ok(());
    }
    println!("Linked {} --[{}]--> {}", id, edge, target);
    Ok(())
}

pub fn print_page_unlinked(id: &str, target: &str, json: bool) -> Result<(), anyhow::Error> {
    if json {
        let payload = serde_json::json!({ "id": id, "target": target, "status": STATUS_UNLINKED });
        println!("{}", serde_json::to_string_pretty(&payload)?);
        return Ok(());
    }
    println!("Unlinked {} from {}", id, target);
    Ok(())
}

pub fn print_record_plan(plan: &RecordMigrationPlan, json: bool) -> Result<(), anyhow::Error> {
    if json {
        println!("{}", serde_json::to_string_pretty(plan)?);
        return Ok(());
    }
    print!("{}", wm_core::page::render_migration_report(plan));
    Ok(())
}
