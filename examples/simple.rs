const SCHEMA: &'static str = r#"
{
    "$id": "https://example.com/job-posting.schema.json",
    "$schema": "https://json-schema.org/draft/2020-12/schema",
    "description": "A representation of a job posting",
    "type": "object",
    "required": ["title", "company", "location", "description"],
    "properties": {
        "title": { "type": "string" },
        "company": { "type": "string" },
        "location": { "type": "string" },
        "description": { "type": "string" },
        "employmentType": { "type": "string" },
        "salary": { "type": "number", "minimum": 0 },
        "applicationDeadline": { "type": "string", "format": "date" }
    }
}"#;

fn main() {
    let json_schema: serde_json::Value =
        serde_json::from_str(SCHEMA).expect("must be a valid JSON");
    let documents = json_schema_faker::generate(&json_schema, 3).unwrap();
}
