use std::path::{Path, PathBuf};

use bevy_math::IVec2;
use num_derive::FromPrimitive;
use serde::{Deserialize, Serialize};

use crate::{animations::Animation, ui_text::Translatable};

pub const HPP_EXTENSION: &str = "hatspp";
pub const DOT_HPP_EXTENSION: &str = ".hatspp";
pub const DEFAULT_PET_SPEED: i32 = 10;
pub const DEFAULT_PET_DISTANCE: i32 = 10;
pub const MAX_PETS: usize = 5;
// pub const DEFAULT_WINGS_IDLE_FRAME: i32 = 0;
// pub const DEFAULT_AUTO_SPEED: i32 = 4;
pub const MAX_EXTRA_HAT_SIZE: IVec2 = IVec2::new(97, 56);
pub const MIN_FRAME_SIZE: i32 = 32;
pub const MAX_FRAME_SIZE: i32 = 64;

#[derive(
    Copy, Clone, Debug, Default, PartialEq, Eq, Hash, FromPrimitive, Serialize, Deserialize, strum::EnumIter
)]
//TODO: add preview back
pub enum HatType {
    #[default]
    Wearable,
    Wings,
    Extra,
    FlyingPet,
    WalkingPet,
    Room,
}

impl Translatable for HatType {
    fn translate_key(&self) -> &str {
        match self {
            HatType::Wearable => "Wearable",
            HatType::Wings => "Wings",
            HatType::Extra => "Extra",
            HatType::FlyingPet => "FlyingPet",
            HatType::WalkingPet => "WalkingPet",
            HatType::Room => "Room",
        }
    }
}

#[derive(Copy, Clone, Default, PartialEq, Eq, Debug)]
pub enum LinkFrameState {
    #[default]
    Default,
    Saved,
    Inverted,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct HatBaseData {
    pub hat_type: HatType,
    pub frame_size: IVec2,
    pub local_image_path: Option<PathBuf>,
    pub local_script_path: Option<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PetBaseData {
    pub distance: i32,
    pub flipped: bool,
}
impl Default for PetBaseData {
    fn default() -> Self {
        Self {
            distance: DEFAULT_PET_DISTANCE,
            flipped: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WearableData {
    pub base: HatBaseData,
    pub strapped_on: bool,
    pub animations: Vec<Animation>,
}

impl Default for WearableData {
    fn default() -> Self {
        Self {
            base: HatBaseData {
                hat_type: HatType::Wearable,
                frame_size: IVec2::splat(MIN_FRAME_SIZE),
                local_image_path: None,
                local_script_path: None,
            },
            strapped_on: Default::default(),
            animations: Default::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WingsData {
    pub general_offset: IVec2,
    pub crouch_offset: IVec2,
    pub ragdoll_offset: IVec2,
    pub slide_offset: IVec2,
    pub net_offset: IVec2,
    pub glide_frame: i32,
    pub idle_frame: i32,
    pub delay: f32,
    pub changes_animations: bool,
    pub base: HatBaseData,
    pub animations: Vec<Animation>,
}

impl Default for WingsData {
    fn default() -> Self {
        Self {
            general_offset: Default::default(),
            crouch_offset: Default::default(),
            ragdoll_offset: Default::default(),
            slide_offset: Default::default(),
            net_offset: Default::default(),
            glide_frame: Default::default(),
            idle_frame: Default::default(),
            delay: Default::default(),
            changes_animations: Default::default(),
            base: HatBaseData {
                hat_type: HatType::Wings,
                frame_size: IVec2::splat(MIN_FRAME_SIZE),
                local_image_path: None,
                local_script_path: None,
            },
            animations: Default::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FlyingPetData {
    pub base: HatBaseData,
    pub pet_base: PetBaseData,
    pub animations: Vec<Animation>,
    pub speed: i32,
}

impl Default for FlyingPetData {
    fn default() -> Self {
        Self {
            base: HatBaseData {
                hat_type: HatType::FlyingPet,
                frame_size: IVec2::splat(MIN_FRAME_SIZE),
                local_image_path: None,
                local_script_path: None,
            },
            pet_base: Default::default(),
            speed: Default::default(),
            animations: Default::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WalkingPetData {
    pub base: HatBaseData,
    pub pet_base: PetBaseData,
    pub animations: Vec<Animation>,
}

impl Default for WalkingPetData {
    fn default() -> Self {
        Self {
            base: HatBaseData {
                hat_type: HatType::WalkingPet,
                frame_size: IVec2::splat(MIN_FRAME_SIZE),
                local_image_path: None,
                local_script_path: None,
            },
            pet_base: Default::default(),
            animations: Default::default(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ExtraHatData {
    pub base: HatBaseData,
}

impl Default for ExtraHatData {
    fn default() -> Self {
        Self {
            base: HatBaseData {
                hat_type: HatType::WalkingPet,
                frame_size: IVec2::splat(MIN_FRAME_SIZE),
                local_image_path: None,
                local_script_path: None,
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum HatElementData {
    Wearable(WearableData),
    Wings(WingsData),
    Extra(ExtraHatData),
    FlyingPet(FlyingPetData),
    WalkingPet(WalkingPetData),
}

impl HatElementData {
    pub fn base(&self) -> &HatBaseData {
        match self {
            HatElementData::Wearable(wearable_data) => &wearable_data.base,
            HatElementData::Wings(wings_data) => &wings_data.base,
            HatElementData::Extra(extra_hat_data) => &extra_hat_data.base,
            HatElementData::FlyingPet(flying_pet_data) => &flying_pet_data.base,
            HatElementData::WalkingPet(walking_pet_data) => &walking_pet_data.base,
        }
    }

    pub fn base_mut(&mut self) -> &mut HatBaseData {
        match self {
            HatElementData::Wearable(wearable_data) => &mut wearable_data.base,
            HatElementData::Wings(wings_data) => &mut wings_data.base,
            HatElementData::Extra(extra_hat_data) => &mut extra_hat_data.base,
            HatElementData::FlyingPet(flying_pet_data) => &mut flying_pet_data.base,
            HatElementData::WalkingPet(walking_pet_data) => &mut walking_pet_data.base,
        }
    }
}

#[derive(Debug, Clone)]
pub enum HatElementDataRef<'a> {
    Wearable(&'a WearableData),
    Wings(&'a WingsData),
    Extra(&'a ExtraHatData),
    FlyingPet(&'a FlyingPetData),
    WalkingPet(&'a WalkingPetData),
}

impl HatElementDataRef<'_> {
    pub fn to_hat_element_data(&self) -> HatElementData {
        match self {
            HatElementDataRef::Wearable(wearable_data) => {
                HatElementData::Wearable((*wearable_data).clone())
            }
            HatElementDataRef::Wings(wings_data) => HatElementData::Wings((*wings_data).clone()),
            HatElementDataRef::Extra(extra_hat_data) => {
                HatElementData::Extra((*extra_hat_data).clone())
            }
            HatElementDataRef::FlyingPet(flying_pet_data) => {
                HatElementData::FlyingPet((*flying_pet_data).clone())
            }
            HatElementDataRef::WalkingPet(walking_pet_data) => {
                HatElementData::WalkingPet((*walking_pet_data).clone())
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HatData {
    pub elements: Vec<HatElementData>,
    pub name: String,
}

impl HatData {
    pub fn new(name: String) -> Self {
        Self {
            elements: Default::default(),
            name,
        }
    }
}
