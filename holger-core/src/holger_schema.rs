use holger_core::config::HolgerConfig;
use schemars::schema_for;
use serde_json;

fn main() {
    let schema = schema_for!(HolgerConfig);
    let json = serde_json::to_string_pretty(&schema).unwrap();
    println!("{}", json);
}
