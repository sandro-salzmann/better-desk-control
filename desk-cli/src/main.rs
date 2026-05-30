use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use desk_core::{raw_to_cm, DeskController, DeskReporter, Direction};

/// Formats the "height" line printed by the `height` command.
fn fmt_height(raw: i32, cm: f64) -> String {
    format!("{cm:.1} cm  (raw {raw})")
}

struct CliReporter;

impl DeskReporter for CliReporter {
    // The CLI runs one command and exits, so live height notifications are
    // dropped; the `height` command reads the current value directly.
    fn height(&self, _raw: i32, _cm: f64) {}
}

// --- address ----------------------------------------------------------------

fn require_address(opt: Option<String>) -> Result<String> {
    opt.ok_or_else(|| {
        anyhow!("no desk address — pass --address <MAC> (run `desk-cli scan` to find it)")
    })
}

// --- CLI --------------------------------------------------------------------

/// Shown at the bottom of `--help`, after the auto-generated command list.
const AFTER_HELP: &str = "\
Address:
  Every command except `scan` requires a desk MAC via --address. Run `scan`
  to discover nearby desks and their addresses.

Examples:
  # find nearby desks and their addresses
  desk-cli scan

  # check the current height
  desk-cli -a DF:EA:BA:E8:8E:44 height

  # nudge up for half a second
  desk-cli -a DF:EA:BA:E8:8E:44 up 0.5

  # hold down for one second (the default duration)
  desk-cli -a DF:EA:BA:E8:8E:44 down

  # stop / release the motor
  desk-cli -a DF:EA:BA:E8:8E:44 stop";

#[derive(Parser)]
#[command(
    name = "desk-cli",
    version,
    after_help = AFTER_HELP
)]
struct Cli {
    /// Desk MAC address, e.g. DF:EA:BA:E8:8E:44. Required for every command except `scan`.
    #[arg(short, long, global = true)]
    address: Option<String>,

    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Scan for nearby desks and print their name + address.
    Scan,
    /// Print the current desk height.
    Height,
    /// Hold UP for N seconds.
    Up {
        #[arg(default_value_t = 1.0)]
        seconds: f64,
    },
    /// Hold DOWN for N seconds.
    Down {
        #[arg(default_value_t = 1.0)]
        seconds: f64,
    },
    /// Stop / release the motor.
    Stop,
}

async fn cmd_scan(ctrl: &DeskController) -> Result<()> {
    let desks = ctrl.scan_desks().await.map_err(|e| anyhow!("{e}"))?;
    if desks.is_empty() {
        println!("no desks found");
        return Ok(());
    }
    for d in &desks {
        println!("  {}  —  {}", d.name, d.address);
    }
    Ok(())
}

async fn run_on_desk(ctrl: &Arc<DeskController>, cmd: Cmd) -> Result<()> {
    match cmd {
        Cmd::Height => match ctrl.current_raw() {
            Some(raw) => println!("{}", fmt_height(raw, raw_to_cm(raw))),
            None => println!("no height reported yet"),
        },
        Cmd::Up { seconds } => {
            ctrl.hold_for(Direction::Up, Duration::from_secs_f64(seconds))
                .await
        }
        Cmd::Down { seconds } => {
            ctrl.hold_for(Direction::Down, Duration::from_secs_f64(seconds))
                .await
        }
        Cmd::Stop => ctrl.stop().await,
        Cmd::Scan => unreachable!("scan is handled before connecting"),
    }
    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let ctrl = Arc::new(DeskController::new(Arc::new(CliReporter)));

    if let Cmd::Scan = cli.cmd {
        return cmd_scan(&ctrl).await;
    }

    let address = require_address(cli.address)?;
    if !ctrl.connect(&address).await {
        anyhow::bail!("could not connect to {address}");
    }
    let result = run_on_desk(&ctrl, cli.cmd).await;
    ctrl.disconnect().await;
    result
}
