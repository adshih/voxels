use server::Server;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut server = Server::bind("127.0.0.1:8080").await?;
    server.run().await
}
