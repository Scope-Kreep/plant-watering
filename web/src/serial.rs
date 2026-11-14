use std::time::Duration;

use eyre::{Context, Result};
use serde::Deserialize;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_serial::SerialPortBuilderExt;

use crate::features::{moisture::Moisture, state::State, stepper::StepperMeasurement};

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum SensorMessage {
    #[serde(rename = "moisture")]
    Moisture { raw: u16, spread: u16, percent: f32 },

    #[serde(rename = "state")]
    State {
        name: String,
        reason: Option<String>,
    },

    #[serde(rename = "stepper")]
    StepperMeasurement {
        current_scale: u16,
        pwm_scale: u8,
        microstep_counter: u16,
        step_time: u32,
        sg_samples: Vec<u16>,
    },
}

async fn store(db: &mut toasty::Db, msg: SensorMessage) -> toasty::Result<()> {
    match msg {
        SensorMessage::Moisture {
            raw,
            spread,
            percent,
        } => {
            toasty::create!(Moisture {
                raw,
                spread,
                percent
            })
            .exec(db)
            .await?;
        }
        SensorMessage::State { name, reason } => {
            toasty::create!(State { name, reason }).exec(db).await?;
        }
        SensorMessage::StepperMeasurement {
            current_scale,
            pwm_scale,
            microstep_counter,
            step_time,
            sg_samples,
        } => {
            toasty::create!(StepperMeasurement {
                current_scale,
                pwm_scale,
                microstep_counter,
                step_time,
                sg_samples
            })
            .exec(db)
            .await?;
        }
    }

    Ok(())
}

async fn read_port(path: &str, baud: u32, db: &mut toasty::Db) -> Result<()> {
    let port = tokio_serial::new(path, baud)
        .open_native_async()
        .wrap_err_with(|| format!("failed to open serial port {path}"))?;
    let mut lines = BufReader::new(port).lines();

    while let Some(line) = lines.next_line().await.wrap_err("serial read failed")? {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        match serde_json::from_str::<SensorMessage>(line) {
            Ok(msg) => {
                if let Err(e) = store(db, msg).await {
                    eprintln!("db write failed: {e}");
                }
            }
            Err(e) => eprintln!("bad line {line:?}: {e}"),
        }
    }

    Ok(())
}

pub async fn run(port: String, baud: u32, mut db: toasty::Db) {
    loop {
        if let Err(e) = read_port(&port, baud, &mut db).await {
            eprintln!("serial error: {e}");
        }
        tokio::time::sleep(Duration::from_secs(2)).await
    }
}
