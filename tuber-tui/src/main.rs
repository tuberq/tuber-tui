mod app;
mod ui;

use app::App;
use clap::Parser;
use crossterm::event::{Event, EventStream, KeyCode, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use futures::StreamExt;
use std::io::stdout;
use std::time::Duration;
use tokio::sync::mpsc;
use tuber_lib::client::TuberClient;
use tuber_lib::model::Snapshot;

#[derive(Parser)]
#[command(name = "tuber-tui", version, about = "TUI dashboard for tuber job queue")]
struct Cli {
    /// Server address [host][:port] (default: localhost:11300)
    #[arg(value_name = "HOST", env = "TUBER_ADDR")]
    addr: Option<String>,

    /// Poll interval in seconds
    #[arg(short, long, default_value = "1.5")]
    interval: f64,
}

enum PollMsg {
    Snapshot(Box<Snapshot>),
    Error(String),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Terminal setup
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = ratatui::init();

    let (tx, mut rx) = mpsc::channel::<PollMsg>(4);
    let addr = tuber_lib::resolve_addr(cli.addr.as_deref());
    let interval = Duration::from_secs_f64(cli.interval);

    // Poller task
    tokio::spawn(async move {
        loop {
            match TuberClient::connect(&addr).await {
                Ok(mut client) => loop {
                    match client.fetch_snapshot().await {
                        Ok(snap) => {
                            if tx.send(PollMsg::Snapshot(Box::new(snap))).await.is_err() {
                                return;
                            }
                        }
                        Err(e) => {
                            let _ = tx.send(PollMsg::Error(e.to_string())).await;
                            break; // reconnect
                        }
                    }
                    tokio::time::sleep(interval).await;
                },
                Err(e) => {
                    let _ = tx.send(PollMsg::Error(format!("Connect failed: {e}"))).await;
                    tokio::time::sleep(Duration::from_secs(2)).await;
                }
            }
        }
    });

    let mut app = App::new();
    let mut event_stream = EventStream::new();

    // Initial render
    terminal.draw(|f| ui::render(f, &app))?;

    loop {
        tokio::select! {
            Some(msg) = rx.recv() => {
                match msg {
                    PollMsg::Snapshot(snap) => app.update(*snap),
                    PollMsg::Error(e) => app.set_error(e),
                }
                terminal.draw(|f| ui::render(f, &app))?;
            }
            Some(Ok(event)) = event_stream.next() => {
                if let Event::Key(key) = event {
                    if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                        break;
                    }
                }
                terminal.draw(|f| ui::render(f, &app))?;
            }
        }
    }

    // Terminal teardown
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
