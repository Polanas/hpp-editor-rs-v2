use std::{
    cell::Cell,
    collections::HashMap,
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use anyhow::{Result, bail};
use downcast_rs::{Downcast, impl_downcast};
use eframe::{glow, icon_data::from_png_bytes};
use pixas::bitmap::Bitmap;
use uuid::Uuid;
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

use crate::{
    animations::Animation,
    hats_data::{
        ExtraHatData, FlyingPetData, HatBaseData, HatData, HatElementData, HatElementDataRef,
        HatType, MAX_PETS, WalkingPetData, WearableData, WingsData,
    },
    image::Image,
    texture::Texture,
};

thread_local! {
    static HAT_ID_COUNTER: Cell<u32> = const { Cell::new(0) };
}

pub fn hat_element_id() -> HatElementId {
    let id = HAT_ID_COUNTER.get();
    HAT_ID_COUNTER.set(id + 1);
    HatElementId(id)
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Default)]
pub struct HatElementId(u32);

pub struct HatViewMut<'a> {
    base: &'a mut HatBaseData,
    texture: &'a mut Texture,
    bitmap: &'a mut Bitmap,
    animations: Option<&'a mut Vec<Animation>>,
}

pub trait HatElement: Downcast + std::fmt::Debug {
    fn base(&self) -> &HatBaseData;
    fn hat_element_data_ref(&self) -> HatElementDataRef;
    fn base_mut(&mut self) -> &mut HatBaseData;
    fn texture(&self) -> &Texture;
    fn texture_mut(&mut self) -> &mut Texture;
    fn bitmap(&self) -> &Bitmap;
    fn bitmap_mut(&mut self) -> &mut Bitmap;
    fn animations(&self) -> Option<&Vec<Animation>>;
    fn animations_mut(&mut self) -> Option<&mut Vec<Animation>>;
    fn id(&self) -> HatElementId;
    fn view_mut(&mut self) -> HatViewMut<'_>;
    fn frames_amount(&self) -> u32;
    fn is_unique(&self) -> bool;
}

pub trait IsPet {
    fn is_pet(&self) -> bool;
}

impl<T: HatElement + ?Sized> IsPet for T {
    fn is_pet(&self) -> bool {
        matches!(
            self.base().hat_type,
            HatType::WalkingPet | HatType::FlyingPet
        )
    }
}

impl_downcast!(HatElement);

macro_rules! impl_hat_element {
    ($t:ident, $($anims_name:ident).+, unique:$e:expr) => {
        paste::paste!{
            impl HatElement for [<$t Hat>] {
                fn view_mut(&mut self) -> HatViewMut<'_> {
                    HatViewMut {
                        base: &mut self.data.base,
                        texture: &mut self.texture,
                        bitmap: &mut self.bitmap,
                        animations: Some(&mut self.$($anims_name).+),
                    }
                }
                fn hat_element_data_ref(&self) -> HatElementDataRef {
                    HatElementDataRef::$t(&self.data)
                }
                fn is_unique(&self) -> bool {
                    $e
                }
                fn bitmap(&self) -> &Bitmap {
                    &self.bitmap
                }
                fn bitmap_mut(&mut self) -> &mut Bitmap {
                    &mut self.bitmap
                }
                fn base(&self) -> &HatBaseData {
                    &self.data.base
                }
                fn base_mut(&mut self) -> &mut HatBaseData {
                    &mut self.data.base
                }
                fn texture_mut(&mut self) -> &mut Texture {
                    &mut self.texture
                }
                fn texture(&self) -> &Texture {
                    &self.texture
                }
                fn animations(&self) -> Option<&Vec<Animation>> {
                    Some(&self.$($anims_name).+)
                }
                fn animations_mut(&mut self) -> Option<&mut Vec<Animation>> {
                    Some(&mut self.$($anims_name).+)
                }
                fn frames_amount(&self) -> u32 {
                    let frames_x = self.texture().width() / self.base().frame_size.x;
                    let frames_y = self.texture().height() / self.base().frame_size.y;
                    (frames_x * frames_y) as u32
                }
                fn id(&self) -> HatElementId {
                    self.id
                }
            }
        }
    };
    ($t:ident, unique:$e:expr) => {
        paste::paste! {
            impl HatElement for [<$t Hat>] {
                fn view_mut(&mut self) -> HatViewMut<'_> {
                    HatViewMut {
                        base: &mut self.data.base,
                        texture: &mut self.texture,
                        bitmap: &mut self.bitmap,
                        animations: None,
                    }
                }
                fn hat_element_data_ref(&self) -> HatElementDataRef {
                    HatElementDataRef::$t(&self.data)
                }
                fn bitmap(&self) -> &Bitmap {
                    &self.bitmap
                }
                fn is_unique(&self) -> bool {
                    $e
                }
                fn bitmap_mut(&mut self) -> &mut Bitmap {
                    &mut self.bitmap
                }
                fn base(&self) -> &HatBaseData {
                    &self.data.base
                }
                fn base_mut(&mut self) -> &mut HatBaseData {
                    &mut self.data.base
                }
                fn texture_mut(&mut self) -> &mut Texture {
                    &mut self.texture
                }
                fn texture(&self) -> &Texture {
                    &self.texture
                }
                fn animations(&self) -> Option<&Vec<Animation>> {
                    None
                }
                fn animations_mut(&mut self) -> Option<&mut Vec<Animation>> {
                    None
                }
                fn frames_amount(&self) -> u32 {
                    let frames_x = self.texture().width() / self.base().frame_size.x;
                    let frames_y = self.texture().height() / self.base().frame_size.y;
                    (frames_x * frames_y) as u32
                }
                fn id(&self) -> HatElementId {
                    self.id
                }
            }
        }
    };
}

impl_hat_element!(Wearable, data.animations, unique: true);
impl_hat_element!(Wings, data.animations, unique: true);
impl_hat_element!(FlyingPet, data.animations, unique: false);
impl_hat_element!(WalkingPet, data.animations, unique: false);
impl_hat_element!(Extra, unique: false);

macro_rules! hat_element_def {
    ($type_name:ident, $data_type:tt) => {
        #[derive(Debug)]
        pub struct $type_name {
            data: $data_type,
            texture: Texture,
            bitmap: Bitmap,
            id: HatElementId,
        }
    };
}

hat_element_def!(WearableHat, WearableData);
hat_element_def!(WingsHat, WingsData);
hat_element_def!(FlyingPetHat, FlyingPetData);
hat_element_def!(WalkingPetHat, WalkingPetData);
hat_element_def!(ExtraHat, ExtraHatData);

pub trait LoadHatElement: Sized + HatElement {
    type Data: serde::Serialize + serde::de::DeserializeOwned + Clone;
    fn load(data: Self::Data, image: Image, gl: &glow::Context) -> Result<Self>;
}

macro_rules! impl_load_hat_element {
    (@manual $hat:ident, $data:ident) => {
        impl LoadHatElement for $hat {
            type Data = $data;

            fn load(mut data: Self::Data, image: Image, gl: &glow::Context) -> Result<Self> {
                let (bitmap, aseprite_data) = image.to_bitmap_with_data();
                if let Some(aseprite_data) = aseprite_data {
                    data.base.frame_size = aseprite_data.frame_size;
                }
                let texture = Texture::from_bitmap(gl, &bitmap)?;
                Ok(Self {
                    data,
                    texture,
                    bitmap,
                    id: hat_element_id(),
                })
            }
        }
    };
    (@anims $hat:ident) => {
        paste::paste! {
            impl LoadHatElement for [<$hat Hat>] {
                type Data = [<$hat Data>];

                fn load(mut data: Self::Data, image: Image, gl: &glow::Context) -> Result<Self> {
                    let (bitmap, aseprite_data) = image.to_bitmap_with_data();
                    if let Some(aseprite_data) = aseprite_data {
                        data.base.frame_size = aseprite_data.frame_size;
                        data.animations = aseprite_data.animations;
                    }
                    let texture = Texture::from_bitmap(gl, &bitmap)?;
                    Ok(Self {
                        data,
                        texture,
                        bitmap,
                        id: hat_element_id(),
                    })
                }
            }

        }
    };
    ($hat:ident) => {
        paste::paste! {
            impl_load_hat_element!(@manual [<$hat Hat>], [<$hat Data>])
        }
    };
}

impl_load_hat_element!(@anims Wearable);
impl_load_hat_element!(@anims Wings);
impl_load_hat_element!(@anims FlyingPet);
impl_load_hat_element!(@anims WalkingPet);
impl_load_hat_element!(@manual ExtraHat, ExtraHatData);

#[derive(Debug, Default)]
pub struct Hat {
    elements: HashMap<HatElementId, Box<dyn HatElement>>,
    path: Option<PathBuf>,
    name: Option<String>,
}

macro_rules! hat_by_type_def {
    ($method_name:ident, $hat_type:ty, $hat_type_enum:expr) => {
        pub fn $method_name(&self) -> Option<&$hat_type> {
            self.elements()
                .find(|e| e.base().hat_type == $hat_type_enum)
                .and_then(|e| e.downcast_ref::<$hat_type>())
        }
        paste::paste! {
            pub fn [<$method_name _mut>](&mut self) -> Option<&mut $hat_type> {
                self.elements_mut()
                    .find(|e| e.base().hat_type == $hat_type_enum)
                    .and_then(|e| e.downcast_mut::<$hat_type>())
            }
        }
    };
}

impl Hat {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_path(path: &Path) -> Self {
        Self {
            elements: Default::default(),
            path: Some(path.to_path_buf()),
            name: None,
        }
    }

    pub fn with_path_and_name(path: &Path, name: &str) -> Self {
        Self {
            elements: Default::default(),
            path: Some(path.to_path_buf()),
            name: Some(name.to_string()),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    pub fn elements(&self) -> impl Iterator<Item = &dyn HatElement> {
        self.elements.values().map(|e| &**e)
    }

    pub fn elements_mut(&mut self) -> impl Iterator<Item = &mut dyn HatElement> {
        self.elements.values_mut().map(|e| &mut **e)
    }

    pub fn pets_amount(&self) -> usize {
        self.elements().filter(|e| e.is_pet()).count()
    }

    pub fn can_add_pets(&self) -> bool {
        self.pets_amount() < MAX_PETS
    }

    pub fn add_element(&mut self, element: impl HatElement) {
        if element.is_pet() && !self.can_add_pets() {
            return;
        }
        if element.is_unique() && self.has_element(element.base().hat_type) {
            return;
        }
        self.elements.insert(element.id(), Box::new(element));
    }

    pub fn has_element(&self, hat_type: HatType) -> bool {
        self.elements().any(|e| e.base().hat_type == hat_type)
    }

    pub fn load(path: impl AsRef<Path>, gl: &eframe::glow::Context) -> Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            bail!("expected path to exist: {:?}", path);
        }

        let file = File::open(path)?;
        let mut zip_archive = ZipArchive::new(file)?;
        let hat_data: HatData = {
            let mut data_json = zip_archive.by_name("data.json")?;
            let mut data_json_string = String::new();
            data_json.read_to_string(&mut data_json_string)?;
            serde_json::from_str(&data_json_string)?
        };
        let mut hat = Hat::with_path_and_name(path, &hat_data.name);

        for element in hat_data.elements {
            let image_path = element.base().local_image_path.as_ref().unwrap();
            let index = zip_archive.index_for_path(image_path).unwrap();
            let mut entry = zip_archive.by_index(index)?;
            let mut data: Vec<u8> = vec![];
            entry.read_to_end(&mut data)?;
            let bitmap = Bitmap::from_png_bytes(&data[..], None)?;

            match element {
                HatElementData::Wearable(wearable_data) => {
                    hat.add_element(WearableHat::load(wearable_data, Image::Bitmap(bitmap), gl)?)
                }
                HatElementData::Wings(wings_data) => {
                    hat.add_element(WingsHat::load(wings_data, Image::Bitmap(bitmap), gl)?)
                }
                HatElementData::Extra(extra_hat_data) => {
                    hat.add_element(ExtraHat::load(extra_hat_data, Image::Bitmap(bitmap), gl)?)
                }
                HatElementData::FlyingPet(flying_pet_data) => hat.add_element(FlyingPetHat::load(
                    flying_pet_data,
                    Image::Bitmap(bitmap),
                    gl,
                )?),
                HatElementData::WalkingPet(walking_pet_data) => hat.add_element(
                    WalkingPetHat::load(walking_pet_data, Image::Bitmap(bitmap), gl)?,
                ),
            };
        }

        Ok(hat)
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        //TODO: add contexts
        let path = path.as_ref();
        let uuid_path: PathBuf = {
            let mut path = path.to_path_buf().into_os_string();
            path.push("_");
            path.push(Uuid::new_v4().to_string());
            path.into()
        };

        let Some(name) = self.name() else {
            bail!("expected hat to have a name");
        };
        //FOR NOW ignoring the possibility of the file already present
        let file = File::create(&uuid_path)?;
        let mut hat_data = HatData::new(name.to_string());

        for element in self.elements() {
            let local_image_path = Path::new("images").join(format!("{}.png", element.id().0));
            let mut element_data = element.hat_element_data_ref().to_hat_element_data();
            let base = element_data.base_mut();
            base.local_image_path = Some(local_image_path);
            assert!(base.local_image_path.is_some());
            hat_data.elements.push(element_data);
        }
        let mut zip_writer = ZipWriter::new(file);
        let options = SimpleFileOptions::default();

        zip_writer.add_directory("images", options)?;

        for (element_data, element) in hat_data.elements.iter().zip(self.elements()) {
            let mut bitmap_png_data = vec![];
            element.bitmap().to_png_bytes(&mut bitmap_png_data)?;
            zip_writer.start_file_from_path(
                element_data.base().local_image_path.as_ref().unwrap(),
                options,
            )?;
            zip_writer.write_all(&bitmap_png_data)?;
        }

        zip_writer.start_file("data.json", options)?;
        zip_writer.write_all(serde_json::to_string_pretty(&hat_data)?.as_bytes())?;
        zip_writer.finish()?;

        if std::fs::exists(path).unwrap_or(false) {
            if let Err(err) = std::fs::remove_file(path) {
                std::fs::remove_file(&uuid_path)?;
                return Err(err.into());
            }
        }
        std::fs::rename(&uuid_path, path)?;
        Ok(())
    }

    hat_by_type_def!(wereable, WearableHat, HatType::Wearable);
    hat_by_type_def!(wings, WingsHat, HatType::Wings);
    hat_by_type_def!(extra, ExtraHat, HatType::Extra);
    hat_by_type_def!(flying_pet, FlyingPetHat, HatType::FlyingPet);
    hat_by_type_def!(walking_pet, WalkingPetHat, HatType::WalkingPet);

    pub fn name(&self) -> Option<&String> {
        self.name.as_ref()
    }

    pub fn path(&self) -> Option<&PathBuf> {
        self.path.as_ref()
    }

    pub fn path_mut(&mut self) -> &mut Option<PathBuf> {
        &mut self.path
    }
}
