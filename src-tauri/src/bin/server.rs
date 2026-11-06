#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let runtime = monitorawake::start(
        monitorawake::data_directory()?,
        Box::new(|timer| {
            println!("Timer terminato: {}", timer.name);
            Ok(())
        }),
    )
    .await?;
    println!("MonitorAwake · server locale (nessun browser aperto)");
    println!("Configurazione: {}", runtime.config_url);
    println!("Il link contiene una chiave privata valida per questa sessione. Ctrl+C per uscire.");
    tokio::signal::ctrl_c().await?;
    runtime.stop();
    Ok(())
}
