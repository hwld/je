mod client;
mod dotenv;

use std::env;

use client::Client;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let args = env::args().collect::<Vec<String>>();
    let input = args.get(1).unwrap();

    let client = Client::new()?;
    let candidates = client.get_candidates(input).await?;

    println!("----------");
    for (i, candidate) in candidates.iter().enumerate() {
        println!("{} {}: {}", i + 1, candidate.name, candidate.desc);
    }
    println!("----------");

    Ok(())
}
