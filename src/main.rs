//! SigmaOS component binary: wiki-hub

use sigmaos_wiki-hub::Component;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Component::new().operate()?;
    println!("sigmaos-wiki-hub: ok");
    Ok(())
}
