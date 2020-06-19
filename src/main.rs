use std::path::PathBuf;

use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use clap::Clap;
use rusqlite::{Connection, OptionalExtension};
use std::sync::Mutex;
use tide::{Request, Response};

#[derive(Clap, Debug)]
struct Opt {
    #[clap(short, long)]
    db: PathBuf,

    #[clap(long, default_value = "/")]
    prefix: String,

    #[clap(long, default_value = "0.0.0.0:9556")]
    bind: String,
}

struct State {
    conn: Mutex<Connection>,
    prefix: String,
}

#[async_std::main]
async fn main() -> Result<()> {
    env_logger::init();

    let opt = Opt::parse();
    let conn = Mutex::new(
        rusqlite::Connection::open(&opt.db).with_context(|| anyhow!("opening {:?}", opt.db))?,
    );

    let mut app = tide::with_state(State {
        conn,
        prefix: opt.prefix.to_string(),
    });

    app.at("/healthcheck")
        .get(|req: Request<State>| async move {
            let conn = req.state().conn.lock().unwrap();
            match conn
                .query_row(
                    "select key, value from data limit 1",
                    rusqlite::NO_PARAMS,
                    |_| Ok(()),
                )
                .optional()
            {
                Ok(_) => Ok(Response::from("{ok: true}")),
                Err(e) => {
                    log::warn!("healthcheck error: {:?}", e);
                    Ok(Response::new(500))
                }
            }
        });

    app.at(&format!("{}*", opt.prefix))
        .get(|req: Request<State>| async move {
            let state = req.state();
            let path = req.url().path();
            let path = &path[state.prefix.len()..];
            let conn = state.conn.lock().unwrap();
            let value: Option<String> = conn
                .query_row("select value from data where key=?", &[path], |r| r.get(0))
                .optional()?;
            if let Some(value) = value {
                Ok(Response::from(value))
            } else {
                Ok(Response::new(404))
            }
        });

    app.listen(&opt.bind).await?;

    Ok(())
}
