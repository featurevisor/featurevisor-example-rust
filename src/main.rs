use std::error::Error;
use std::io;
use std::time::Duration;

use featurevisor::{
    context, create_featurevisor, DatafileInput, FeaturevisorOptions, LogLevel, VariableValue,
};

const DATAFILE_URL: &str =
    "https://featurevisor-example-cloudflare.pages.dev/production/featurevisor-sdk-v3.json";

fn main() -> Result<(), Box<dyn Error>> {
    let datafile = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?
        .get(DATAFILE_URL)
        .header("Accept", "application/json")
        .send()?
        .error_for_status()?
        .text()?;

    let f = create_featurevisor(FeaturevisorOptions {
        datafile: Some(DatafileInput::Json(datafile)),
        context: Some(context! {
            "userId" => "customer-123",
            "country" => "nl",
            "locale" => "nl-NL",
            "accountPlan" => "pro",
        }),
        log_level: Some(LogLevel::Error),
        ..Default::default()
    });

    let commerce_enabled = f.is_enabled("commerce_platform", None);
    let checkout_variation = f.get_variation("checkout_experience", None, None);
    let max_items = f.get_variable_integer("checkout_experience", "max_items", None, None);
    let payment_methods =
        f.get_variable_array("checkout_experience", "payment_methods", None, None);
    let endpoints = f
        .get_global_variable_object("serviceEndpoints", None, None)
        .ok_or_else(|| io::Error::other("serviceEndpoints is unavailable"))?;
    let support_contact = f.get_global_variable_string("supportContact", None, None);

    let base_url = string_field(&endpoints, "baseUrl")?;
    let timeout_ms = integer_field(&endpoints, "timeoutMs")?;
    let retries = integer_field(&endpoints, "retries")?;

    println!("Commerce platform enabled: {commerce_enabled}");
    println!(
        "Checkout variation: {}",
        checkout_variation.as_deref().unwrap_or("unavailable")
    );
    println!(
        "Maximum checkout items: {}",
        max_items
            .map(|value| value.to_string())
            .unwrap_or_else(|| "unavailable".to_string())
    );
    println!("Payment methods: {}", format_array(payment_methods));
    println!("Service endpoint: {base_url} (timeout: {timeout_ms} ms, retries: {retries})");
    println!(
        "Support contact: {}",
        support_contact.as_deref().unwrap_or("unavailable")
    );

    f.close();
    Ok(())
}

fn string_field<'a>(
    object: &'a std::collections::HashMap<String, VariableValue>,
    key: &str,
) -> Result<&'a str, Box<dyn Error>> {
    match object.get(key) {
        Some(VariableValue::String(value)) => Ok(value),
        _ => Err(io::Error::other(format!("{key} is unavailable")).into()),
    }
}

fn integer_field(
    object: &std::collections::HashMap<String, VariableValue>,
    key: &str,
) -> Result<i64, Box<dyn Error>> {
    match object.get(key) {
        Some(VariableValue::Integer(value)) => Ok(*value),
        _ => Err(io::Error::other(format!("{key} is unavailable")).into()),
    }
}

fn format_array(values: Option<Vec<VariableValue>>) -> String {
    let items = values
        .unwrap_or_default()
        .into_iter()
        .filter_map(|value| match value {
            VariableValue::String(value) => Some(value),
            _ => None,
        })
        .collect::<Vec<_>>();
    format!("[{}]", items.join(", "))
}
