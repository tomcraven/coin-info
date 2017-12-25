extern crate reqwest;
extern crate serde_json;
extern crate structopt;
#[macro_use]
extern crate structopt_derive;
extern crate termion;
extern crate tui;

#[macro_use]
extern crate serde_derive;

use tui::Terminal;
use tui::backend::MouseBackend;
use tui::widgets::{border, Block, Row, Table, Widget};
use tui::layout::{Direction, Group, Rect, Size};

use structopt::StructOpt;

use termion::event;
use termion::input::TermRead;

use std::vec::Vec;
use std::sync::mpsc;
use std::thread;
use std::time;
use std::io;

#[derive(Deserialize)]
struct Ticker {
    symbol: String,
    price_usd: String,
    percent_change_1h: String,
    percent_change_24h: String,
    percent_change_7d: String,

    #[serde(rename = "24h_volume_usd")] volume_usd: String,
}

#[derive(StructOpt, Debug)]
#[structopt(name = "coin-info", about = "A crypto coin price ticker")]
struct Opt {
    #[structopt(help = "List of coins to track")] symbols: Vec<String>,
}

fn get_tickers_for_symbols(symbols: &Vec<String>) -> Vec<Ticker> {
    let mut resp = reqwest::get("https://api.coinmarketcap.com/v1/ticker/").unwrap();
    let body = resp.text().unwrap();
    let all_symbols: Vec<Ticker> = serde_json::from_str(&body).unwrap();
    all_symbols
        .into_iter()
        .filter(|ref ticker| match symbols.len() {
            0 => true,
            _ => symbols.contains(&ticker.symbol.to_lowercase()),
        })
        .collect::<Vec<_>>()
}

enum Event {
    Input(event::Key),
    Tick,
}

struct App {
    size: Rect,
    tickers: Vec<Ticker>,
}

impl App {
    fn new() -> App {
        App {
            tickers: vec![],
            size: Rect::default(),
        }
    }
}

fn draw(t: &mut Terminal<MouseBackend>, app: &App) {
    Group::default()
        .direction(Direction::Horizontal)
        .sizes(&[Size::Percent(100)])
        .render(t, &app.size, |t, chunks| {
            Table::new(
                ["index", "symbol", "usd", "1h", "24h", "7d", "24h volume"].into_iter(),
                app.tickers.iter().enumerate().map(|(i, ticker)| {
                    Row::Data(
                        vec![
                            (i + 1).to_string(),
                            ticker.symbol.clone(),
                            ticker.price_usd.clone(),
                            ticker.percent_change_1h.clone(),
                            ticker.percent_change_24h.clone(),
                            ticker.percent_change_7d.clone(),
                            ticker.volume_usd.clone(),
                        ].into_iter(),
                    )
                }),
            ).block(Block::default().borders(border::ALL))
                .widths(&[6, 10, 10, 10, 10, 10, 15])
                .render(t, &chunks[0]);
        });
    t.draw().unwrap();
}

fn main() {
    let opt = Opt::from_args();

    // Terminal initialization
    let backend = MouseBackend::new().unwrap();
    let mut terminal = Terminal::new(backend).unwrap();

    // Channels
    let (tx, rx) = mpsc::channel();
    let input_tx = tx.clone();
    let clock_tx = tx.clone();

    // Input
    thread::spawn(move || {
        let stdin = io::stdin();
        for c in stdin.keys() {
            let evt = c.unwrap();
            input_tx.send(Event::Input(evt)).unwrap();
            if evt == event::Key::Char('q') {
                break;
            }
        }
    });

    // Tick
    thread::spawn(move || loop {
        clock_tx.send(Event::Tick).unwrap();
        thread::sleep(time::Duration::from_millis(500));
    });

    // App
    let mut app = App::new();

    // First draw call
    terminal.clear().unwrap();
    terminal.hide_cursor().unwrap();
    app.size = terminal.size().unwrap();
    draw(&mut terminal, &app);

    // Main loop
    loop {
        let size = terminal.size().unwrap();
        if size != app.size {
            terminal.resize(size).unwrap();
            app.size = size;
        }

        let evt = rx.recv().unwrap();
        match evt {
            Event::Input(input) => match input {
                event::Key::Char('q') => {
                    break;
                }
                _ => {}
            },
            Event::Tick => {
                app.tickers = get_tickers_for_symbols(&opt.symbols);
            }
        }
        draw(&mut terminal, &app);
    }

    terminal.show_cursor().unwrap();
    terminal.clear().unwrap();
}
