use dobot_fx24::{error, Dobot};
use serialport::SerialPort;

use crate::{Shapes, _wait_n_ms};

use super::structs::{DobotPath, Position, RelayPath};
/// In this repository, a relay connected to arduino is used.
/// therefore, a default connection to arduino at /dev/ttyACM0, baud rate 9600 is used.
/// (Path is set in config.json (is autocreated at startup of program), or in structs.rs, Config::default) 
/// If you're interested in this implementatiom, update values to your needs.
/// The relay arduino circuit just reads 1 to open and 0 to close the relay, nothing fancy.
impl RelayPath {
    pub fn new(&self) -> Result<Box<dyn SerialPort>, serialport::Error> {
        serialport::new(&self.relaypath, 9600)
            .timeout(std::time::Duration::from_millis(10))
            .open()
    }
}

pub fn toggle_relay<'a>(relay: &'a mut Box<dyn SerialPort>, signal: bool) -> Option<String> {
    let signal_bytes = match signal {
        true => "1\n".as_bytes(),
        false => "0\n".as_bytes(),
    };
    match relay.write(signal_bytes) {
        Ok(success) => {
            log::info!("Written {} bytes", success);
            None
        }
        Err(errmsg) => Some(errmsg.to_string()),
    }
}

/// Same as relay, path defined in config.json or structs.rs, Config::default.
impl DobotPath {
    pub async fn go_home(&self) -> Option<String> {
        match Dobot::open(&self.dobotpath).await {
            Ok(mut dobot) => match dobot.set_home().await {
                Ok(_handle) => {
                    let _ = _handle.wait().await;
                    None
                }
                Err(errmsg) => Some(format!("{}", errmsg)),
            },
            Err(errmsg) => Some(format!("{}", errmsg)),
        }
    }

    async fn move_dobot_to(&self, position: Position) -> Option<String> {
        match Dobot::open(&self.dobotpath).await {
            Ok(mut dobot) => {
                set_dobot_params(&mut dobot, position.v.in_float, position.a.in_float)
                    .await
                    .unwrap();
                match dobot
                    .move_to(
                        position.x.in_float,
                        position.y.in_float,
                        position.z.in_float,
                        position.r.in_float,
                    )
                    .await
                    .unwrap()
                    .wait()
                    .await
                {
                    Ok(_) => None,
                    Err(errmsg) => Some(format!("{}", errmsg)),
                }
            }
            Err(errmsg) => Some(format!("{}", errmsg)),
        }
    }

    async fn move_dobot_sequence(&self, sequence: Vec<Position>) -> Option<String> {
        match Dobot::open(&self.dobotpath).await {
            Ok(mut dobot) => {
                for position in sequence {
                    set_dobot_params(&mut dobot, position.v.in_float, position.a.in_float)
                        .await
                        .unwrap();
                    match dobot
                        .move_to(
                            position.x.in_float,
                            position.y.in_float,
                            position.z.in_float,
                            position.r.in_float,
                        )
                        .await
                        .unwrap()
                        .wait()
                        .await
                    {
                        Ok(_) => {}
                        Err(errmsg) => return Some(format!("{}", errmsg)),
                    }
                }
                None
            }
            Err(errmsg) => Some(format!("{}", errmsg)),
        }
    }

    pub async fn _get_cur_pos(&self) -> (Option<dobot_fx24::Pose>, Option<String>) {
        match Dobot::open(&self.dobotpath).await {
            Ok(mut dobot) => match dobot.get_pose().await {
                Ok(pose) => (Some(pose), None),
                Err(errmsg) => (None, Some(format!("{}", errmsg))),
            },
            Err(errmsg) => (None, Some(format!("{}", errmsg))),
        }
    }

    pub async fn test_connection(&self) -> Option<String> {
        match Dobot::open(&self.dobotpath).await {
            Ok(_) => None,
            Err(errmsg) => Some(format!("{}", errmsg)),
        }
    }
}

async fn set_dobot_params<'a>(
    internal_dobot: &'a mut Dobot,
    v: f32,
    a: f32,
) -> Result<(), error::Error> {
    internal_dobot
        .set_ptp_coordinate_params(v, a)
        .await
        .unwrap()
        .wait()
        .await
}

pub async fn perform_sequences<'a>(
    dobot_path: DobotPath,
    sequences: Vec<Position>,
) -> Option<String> {
    if dobot_path.dobotpath.is_empty() {
        return Some("Dobot path not set.".to_string());
    }
    dobot_path.move_dobot_sequence(sequences).await
}

pub async fn perform_moveto<'a>(dobot_path: DobotPath, sequences: Position) -> Option<String> {
    if dobot_path.dobotpath.is_empty() {
        return Some("Dobot path not set.".to_string());
    }
    dobot_path.move_dobot_to(sequences).await
}

pub async fn go_home<'a>(dobot_path: DobotPath) -> Option<String> {
    if dobot_path.dobotpath.is_empty() {
        return Some("Dobot path not set.".to_string());
    }
    dobot_path.go_home().await
}

pub async fn test_connection<'a>(dobot_path: DobotPath) -> Option<String> {
    if dobot_path.dobotpath.is_empty() {
        return Some("Dobot path not set.".to_string());
    }
    dobot_path.test_connection().await
}

pub async fn draw_shape(
    dobot_path: DobotPath,
    relay_path: RelayPath,
    shape_to_draw: Shapes,
    arm_speed: f32,
    arm_accel: f32,
) -> Option<String> {
    log::info!(
        "Drawing shape {:#?}, speed {}, acceleration {}",
        shape_to_draw,
        arm_speed,
        arm_accel
    );
    let shape_motion_data = shape_to_draw.motion_data(arm_speed, arm_accel);
    match relay_path.new() {
        Ok(mut relay) => {
            _wait_n_ms(2000).await;
            if let Some(errmsg) = dobot_path
                .move_dobot_to(shape_motion_data.last().unwrap().clone())
                .await
            {
                return Some(format!("{}", errmsg));
            };
            _wait_n_ms(500).await;
            toggle_relay(&mut relay, true);
            if let Some(errmsg) = dobot_path.move_dobot_sequence(shape_motion_data).await {
                return Some(format!("{}", errmsg));
            };
            _wait_n_ms(500).await;
            toggle_relay(&mut relay, false);
            _wait_n_ms(500).await;
            if let Some(errmsg) = dobot_path
                .move_dobot_to(Position::position(200.0, 0.0, 0.0, 0.0, 200.0, 200.0))
                .await
            {
                return Some(format!("{}", errmsg));
            };
        }
        Err(errmsg) => return Some(format!("{}", errmsg)),
    }
    None
}

pub async fn simulate_drawing<'a>() -> Option<String> {
    _wait_n_ms(1500).await;
    None
}
