#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let runtime = monitorawake::start(
        monitorawake::data_directory()?,
        Box::new(|timer| {
            println!("Timer finished: {}", timer.name);
            Ok(())
        }),
    )
    .await?;
    println!("MonitorAwake · local server (no browser opened)");
    println!("Settings: {}", runtime.config_url);
    println!("The link contains a private key valid for this session. Press Ctrl+C to quit.");
    tokio::signal::ctrl_c().await?;
    runtime.stop();
    Ok(())
}
