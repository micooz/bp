use serde::Deserialize;
use tide::Request;

#[derive(Debug, Deserialize)]
struct Animal {
    name: String,
    legs: u8,
}

pub async fn run() -> tide::Result<()> {
    let mut app = tide::new();
    app.at("/orders/shoes").get(order_shoes);
    app.listen("127.0.0.1:8080").await?;
    Ok(())
}

async fn order_shoes(req: Request<()>) -> tide::Result {
    let Animal { name, legs } = req.query()?;
    Ok(format!("Hello, {}! I've put in an order for {} shoes", name, legs).into())
}
