use num_derive::FromPrimitive;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{cell::Cell, fmt::Display, hash::Hash};

use crate::{hats_data::HatType, ui_text::Translatable};

pub static PET_ANIMATIONS: Lazy<Vec<AnimType>> = Lazy::new(|| {
    use AnimType::*;
    vec![
        OnDefault,
        OnPressQuack,
        OnReleaseQuack,
        OnPetStop,
        OnPetApproach,
        OnDuckDeath,
        OnDuckJump,
        OnDuckLand,
        OnDuckGlide,
        OnDuckWalk,
        OnDuckSneak,
        OnDuckNetted,
        OnDuckSpawned,
        OnHatPickedUp,
    ]
});
pub static NON_PET_ANIMATIONS: Lazy<Vec<AnimType>> = Lazy::new(|| {
    use AnimType::*;
    vec![
        OnDefault,
        OnPressQuack,
        OnReleaseQuack,
        // OnPetStop,
        // OnPetApproach,
        OnDuckDeath,
        OnDuckJump,
        OnDuckLand,
        OnDuckGlide,
        OnDuckWalk,
        OnDuckSneak,
        OnDuckNetted,
        OnDuckSpawned,
        OnHatPickedUp,
    ]
});

pub fn avalible_animations<'a>(hat_type: HatType) -> Option<&'a [AnimType]> {
    match hat_type {
        HatType::FlyingPet | HatType::WalkingPet => Some(&PET_ANIMATIONS),
        _ => Some(&NON_PET_ANIMATIONS),
    }
}

#[derive(Copy, Clone, Debug, FromPrimitive, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AnimType {
    #[default]
    OnDefault,
    OnPressQuack,
    OnReleaseQuack,
    OnPetStop,
    OnPetApproach,
    OnDuckDeath,
    OnDuckJump,
    OnDuckLand,
    OnDuckGlide,
    OnDuckWalk,
    OnDuckSneak,
    OnDuckNetted,
    OnDuckSpawned,
    OnHatPickedUp,
}

impl Translatable for AnimType {
    fn translate_key(&self) -> &str {
        match self {
            AnimType::OnDefault => "0",
            AnimType::OnPressQuack => "1",
            AnimType::OnReleaseQuack => "2",
            AnimType::OnPetStop => "3",
            AnimType::OnPetApproach => "4",
            AnimType::OnDuckDeath => "5",
            AnimType::OnDuckJump => "6",
            AnimType::OnDuckLand => "7",
            AnimType::OnDuckGlide => "8",
            AnimType::OnDuckWalk => "9",
            AnimType::OnDuckSneak => "10",
            AnimType::OnDuckNetted => "11",
            AnimType::OnDuckSpawned => "12",
            AnimType::OnHatPickedUp => "13",
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, Default)]
pub struct FrameId(pub u32);

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Frame {
    pub value: u32,
    pub delay: Option<f32>,
    #[serde(skip)]
    id: FrameId,
}

impl Hash for Frame {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Clone for Frame {
    fn clone(&self) -> Self {
        Self {
            value: self.value,
            id: frame_id(),
            delay: self.delay,
        }
    }
}

impl Frame {
    pub fn new(value: u32) -> Self {
        Self {
            value,
            id: frame_id(),
            delay: None,
        }
    }
    pub fn with_delay(value: u32, delay: f32) -> Self {
        Self {
            value,
            id: frame_id(),
            delay: Some(delay),
        }
    }
    pub fn id(&self) -> FrameId {
        self.id
    }
}

impl From<Frame> for u32 {
    fn from(frame: Frame) -> Self {
        frame.value
    }
}
impl From<Frame> for i32 {
    fn from(frame: Frame) -> Self {
        frame.value as _
    }
}
impl From<u32> for Frame {
    fn from(value: u32) -> Self {
        Self {
            value,
            id: frame_id(),
            delay: None,
        }
    }
}
impl From<i32> for Frame {
    fn from(value: i32) -> Self {
        Self {
            value: value as _,
            id: frame_id(),
            delay: None,
        }
    }
}

thread_local! {
    static FRAME_ID_COUNTER: Cell<u32> = const { Cell::new(0) };
}

pub fn frame_id() -> FrameId {
    let id = FRAME_ID_COUNTER.get();
    FRAME_ID_COUNTER.set(id + 1);
    FrameId(id)
}

#[derive(Clone, Debug, Serialize, Default, Deserialize)]
pub struct Animation {
    //TODO: add support for diff. delay per frame
    pub anim_type: AnimType,
    pub delay: f32,
    pub looping: bool,
    pub frames: Vec<Frame>,
    #[serde(skip)]
    pub new_frame: i32,
    #[serde(skip)]
    pub new_range_start: i32,
    #[serde(skip)]
    pub new_range_end: i32,
}

impl Animation {
    pub fn new(anim_type: AnimType, delay: f32, looping: bool, frames: Vec<Frame>) -> Self {
        Self {
            anim_type,
            delay,
            looping,
            frames,
            new_frame: 1,
            new_range_end: 1,
            new_range_start: 1,
        }
    }
}
