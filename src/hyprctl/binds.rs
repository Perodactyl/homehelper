use super::send_command;
use anyhow::Result;
use serde::Deserialize;

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Deserialize)]
struct BindInternal {
    locked: bool,
    mouse: bool,
    release: bool,
    repeat: bool,
    #[serde(rename = "longPress")]
    long_press: bool,
    non_consuming: bool,
    has_description: bool,
    modmask: u32,
    submap: String,
    key: String,
    keycode: u32,
    catch_all: bool,
    description: String,
    dispatcher: String,
    arg: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindTrigger {
    Press,
    Release,
    LongPress,
    CatchAll,
    Mouse,
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct Bind {
    pub locked: bool,
    pub repeat: bool,
    pub non_consuming: bool,
    pub trigger: BindTrigger,
    pub modmask: u32,
    pub submap: Option<String>,
    pub key: String,
    pub keycode: u32,
    pub description: Option<String>,
    pub action: (String, String),
}
impl From<BindInternal> for Bind {
    fn from(value: BindInternal) -> Self {
        Bind {
            locked: value.locked,
            repeat: value.repeat,
            non_consuming: value.non_consuming,
            modmask: value.modmask,
            trigger: if value.mouse {
                BindTrigger::Mouse
            } else if value.catch_all {
                BindTrigger::CatchAll
            } else if value.long_press {
                BindTrigger::LongPress
            } else if value.release {
                BindTrigger::Release
            } else {
                BindTrigger::Press
            },
            submap: if value.submap.is_empty() {
                None
            } else {
                Some(value.submap)
            },
            key: value.key,
            keycode: value.keycode,
            description: if value.has_description {
                Some(value.description)
            } else {
                None
            },
            action: (value.dispatcher, value.arg),
        }
    }
}

pub fn binds() -> Result<Vec<Bind>> {
    Ok(
        serde_json::from_str::<Vec<BindInternal>>(&send_command(b"j/binds")?)?
            .into_iter()
            .map(<BindInternal as Into<Bind>>::into)
            .collect(),
    )
}
