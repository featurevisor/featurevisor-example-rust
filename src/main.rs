use std::error::Error;

use featurevisor::{context, create_featurevisor, DatafileInput, FeaturevisorOptions};

const DATAFILE_URL: &str =
    "https://featurevisor-example-cloudflare.pages.dev/production/featurevisor-mobile.json";

fn main() -> Result<(), Box<dyn Error>> {
    let datafile = reqwest::blocking::get(DATAFILE_URL)?
        .error_for_status()?
        .text()?;

    let f = create_featurevisor(FeaturevisorOptions {
        datafile: Some(DatafileInput::Json(datafile)),
        ..Default::default()
    });

    f.set_context(
        context! {
            "userId" => "mobile-user",
            "country" => "nl",
        },
        false,
    );

    let enabled = f.is_enabled("mobile_experience", None);
    let variation = f.get_variation("mobile_experience", None, None);
    let welcome_message = f.get_variable_string("mobile_experience", "welcome_message", None, None);

    println!("Datafile revision: {}", f.get_revision());
    println!("Feature 'mobile_experience' enabled: {enabled}");
    println!("Variation: {}", variation.as_deref().unwrap_or("none"));
    println!(
        "Variable 'welcome_message': {}",
        welcome_message.as_deref().unwrap_or("none")
    );

    f.close();
    Ok(())
}
