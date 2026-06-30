mod app;
mod components;
mod config;
mod nix;

fn main() -> std::io::Result<()> {
    app::run()
}
