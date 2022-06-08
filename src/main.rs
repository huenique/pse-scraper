mod pse;

#[tokio::main]
async fn main() {
    use std::time::Instant;
    let now = Instant::now();

    let _res = pse::search::get_listed_companies().await;

    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
}
