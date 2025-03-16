use std::{
    cell::Cell,
    collections::HashMap,
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use anyhow::{Context as _, Result, bail};
use downcast_rs::{Downcast, impl_downcast};
use eframe::{glow, icon_data::from_png_bytes};
use pixas::bitmap::Bitmap;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use uuid::Uuid;
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

use crate::{
    animations::Animation,
    hats_data::{
        ExtraHatData, FlyingPetData, HatBaseData, HatData, HatElementData, HatElementDataRef,
        HatType, MAX_PETS, WalkingPetData, WearableData, WingsData,
    },
    image::Image,
    path_utils::{LocalPath, LocalPathError},
    texture::Texture,
};

thread_local! {
    static HAT_ID_COUNTER: Cell<u32> = const { Cell::new(1) };
}

pub fn hat_element_id() -> HatElementId {
    let id = HAT_ID_COUNTER.get();
    HAT_ID_COUNTER.set(id + 1);
    HatElementId(id)
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Default)]
pub struct HatElementId(pub u32);

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
    type Data: serde::Serialize + serde::de::DeserializeOwned + Clone + Default;
    fn load(data: Self::Data, image: Image, gl: &glow::Context) -> Result<Self>;
    fn load_from_path(path: &Path, gl: &glow::Context) -> Result<Self> {
        let image = Image::new(path).context(format!("could not load image at {:?}", &path))?;
        Self::load(Self::Data::default(), image, gl)
    }
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

#[derive(Debug, Clone, Copy, Hash, Default)]
pub struct HatId(pub u32);

thread_local! {
    static FRAME_ID_COUNTER: Cell<u32> = const { Cell::new(0) };
}

pub fn hat_id() -> HatId {
    let id = FRAME_ID_COUNTER.get();
    FRAME_ID_COUNTER.set(id + 1);
    HatId(id)
}

#[derive(Debug)]
pub struct Hat {
    elements: HashMap<HatElementId, Box<dyn HatElement>>,
    path: PathBuf,
    name: String,
    name_set_by_user: bool,
    id: HatId,
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
#[derive(Clone, Copy, Debug)]
pub enum HatSaveType {
    Folder,
    File,
}

impl Hat {
    pub fn new(path: &Path, name: &str) -> Self {
        Self {
            elements: Default::default(),
            path: path.to_path_buf(),
            name: name.to_string(),
            name_set_by_user: false,
            id: hat_id(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    pub fn element(&self, id: HatElementId) -> Option<&dyn HatElement> {
        self.elements.get(&id).map(|e| &**e)
    }

    pub fn element_exists(&self, id: HatElementId) -> bool {
        self.elements.contains_key(&id)
    }

    pub fn element_mut(&mut self, id: HatElementId) -> Option<&mut dyn HatElement> {
        self.elements.get_mut(&id).map(|e| &mut **e)
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

    pub fn can_add_elements(&self) -> bool {
        self.can_add_pets() && !HatType::iter().all(|hat_type| self.has_element(hat_type))
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

    pub fn remove_element(&mut self, element_id: HatElementId) {
        self.elements.remove(&element_id);
    }

    pub fn has_element(&self, hat_type: HatType) -> bool {
        self.elements().any(|e| e.base().hat_type == hat_type)
    }

    pub fn has_element_with_id(&self, hat_id: HatElementId) -> bool {
        self.elements().any(|e| e.id() == hat_id)
    }

    pub fn load(path: impl AsRef<Path>, gl: &glow::Context) -> Result<Self> {
        let path = path.as_ref();
        let data_path = path.join("data.json");
        let images_path = path.join("images");
        for path in &[path, &images_path] {
            if !path.exists() {
                bail!("expected {:?} to exist", path);
            }
        }

        let data: HatData = if data_path.exists() {
            File::open(&data_path)
                .context(format!("could not open {:?}", &data_path))
                .and_then(|mut file| {
                    let mut data = String::new();
                    file.read_to_string(&mut data)
                        .context(format!("could not read {:?}", &data_path))
                        .map(|_| data)
                })
                .and_then(|data_string| {
                    serde_json::from_str(&data_string)
                        .context(format!("could not parse {:?}", &data_path))
                })?
        } else {
            let mut file =
                File::create(&data_path).context(format!("could not create {:?}", data_path))?;
            let hat_data = HatData::new("Default".to_string());
            write!(
                file,
                "{}",
                serde_json::to_string_pretty(&hat_data).expect("should always succeed")
            )
            .context(format!("could not write into {:?}", &data_path))?;
            hat_data
        };

        let mut hat = Hat::new(path, &data.name);
        for element in data.elements {
            let local_image_path = element.base().local_image_path.as_ref().unwrap();
            let image_path = path.join(local_image_path);
            let bitmap = Bitmap::from_path(&image_path)
                .context(format!("could not read image at {:?}", &image_path))?;

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

    // pub fn load_from_file(path: impl AsRef<Path>, gl: &eframe::glow::Context) -> Result<Self> {
    //     let path = path.as_ref();
    //     if !path.exists() {
    //         bail!("expected path to exist: {:?}", path);
    //     }
    //
    //     let file = File::open(path)?;
    //     let mut zip_archive = ZipArchive::new(file)?;
    //     let hat_data: HatData = {
    //         let mut data_json = zip_archive.by_name("data.json")?;
    //         let mut data_json_string = String::new();
    //         data_json.read_to_string(&mut data_json_string)?;
    //         serde_json::from_str(&data_json_string)?
    //     };
    //     let mut hat = Hat::with_path(path, &hat_data.name);
    //
    //     for element in hat_data.elements {
    //         let image_path = element.base().local_image_path.as_ref().unwrap();
    //         let index = zip_archive.index_for_path(image_path).unwrap();
    //         let mut entry = zip_archive.by_index(index)?;
    //         let mut data: Vec<u8> = vec![];
    //         entry.read_to_end(&mut data)?;
    //         let bitmap = Bitmap::from_png_bytes(&data[..], None)?;
    //
    //         match element {
    //             HatElementData::Wearable(wearable_data) => {
    //                 hat.add_element(WearableHat::load(wearable_data, Image::Bitmap(bitmap), gl)?)
    //             }
    //             HatElementData::Wings(wings_data) => {
    //                 hat.add_element(WingsHat::load(wings_data, Image::Bitmap(bitmap), gl)?)
    //             }
    //             HatElementData::Extra(extra_hat_data) => {
    //                 hat.add_element(ExtraHat::load(extra_hat_data, Image::Bitmap(bitmap), gl)?)
    //             }
    //             HatElementData::FlyingPet(flying_pet_data) => hat.add_element(FlyingPetHat::load(
    //                 flying_pet_data,
    //                 Image::Bitmap(bitmap),
    //                 gl,
    //             )?),
    //             HatElementData::WalkingPet(walking_pet_data) => hat.add_element(
    //                 WalkingPetHat::load(walking_pet_data, Image::Bitmap(bitmap), gl)?,
    //             ),
    //         };
    //     }
    //
    //     Ok(hat)
    // }

    // pub fn save_as(&mut self) -> Result<()> {
    //     let path = rfd::FileDialog::new()
    //         .pick_folder()
    //         .context("could not pick folder")?;
    //     self.save(&path)?;
    //     *self.path_mut() = path;
    //     Ok(())
    // }
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        if let Err(err) = self.check_files_integrity() {
            bail!("failed files integrity check: {}", err.to_string());
        }
        let path = path.as_ref().join("data.json");
        let uuid_path: PathBuf = {
            let mut path = path.to_path_buf().into_os_string();
            path.push("_");
            path.push(Uuid::new_v4().to_string());
            path.into()
        };

        let mut file =
            File::create(&uuid_path).context(format!("could not create {:?}", uuid_path))?;

        let data_string = serde_json::to_string_pretty(&self.gen_hat_data(HatSaveType::Folder))
            .context("could not generate data.json")?;

        write!(file, "{}", data_string).context(format!(
            "could not write hat data to file at {:?}",
            &uuid_path
        ))?;

        if std::fs::exists(&path).unwrap_or(false) {
            if let Err(err) = std::fs::remove_file(&path) {
                std::fs::remove_file(&uuid_path)
                    .context(format!("could not remove {:?}", &uuid_path))?;
                return Err(err.into());
            }
        }

        std::fs::rename(&uuid_path, path)
            .context(format!("could not rename file: {:?}", uuid_path))?;
        Ok(())
    }

    pub fn gen_hat_data(&self, save_type: HatSaveType) -> HatData {
        let mut hat_data = HatData::new(self.name().to_string());
        for element in self.elements() {
            let local_image_path = match save_type {
                HatSaveType::Folder => {
                    //TODO: account for the situation where image is NOT in images - copy pngs
                    let image_path = element.bitmap().path().unwrap();
                    match image_path.local_path(self.path()) {
                        Ok(path) => path,
                        Err(LocalPathError::PathNotInDir) => todo!(),
                    }
                }
                HatSaveType::File => Path::new("images").join(format!("{}.png", element.id().0)),
            };
            let mut element_data = element.hat_element_data_ref().to_hat_element_data();
            let base = element_data.base_mut();
            base.local_image_path = Some(local_image_path);
            assert!(base.local_image_path.is_some());
            hat_data.elements.push(element_data);
        }
        hat_data
    }

    pub fn check_files_integrity(&self) -> Result<()> {
        if !self.path().exists() {
            bail!("{:?} does not exist", self.path());
        }
        for element in self.elements() {
            if let Some(path) = &element.base().local_image_path {
                let path = self.path().join(path);
                if !path.exists() {
                    bail!("{:?} does not exist", path);
                }
            }
            if let Some(path) = &element.base().local_script_path
                && !self.path().join(path).exists()
            {
                let path = self.path().join(path);
                if !path.exists() {
                    bail!("{:?} does not exist", path);
                }
            }
        }

        Ok(())
    }

    pub fn export_to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        if let Err(err) = self.check_files_integrity() {
            bail!("failed files integrity check: {}", err.to_string());
        }
        let path = path.as_ref();

        let uuid_path: PathBuf = {
            let mut path = path.to_path_buf().into_os_string();
            path.push("_");
            path.push(Uuid::new_v4().to_string());
            path.into()
        };

        let file =
            File::create(&uuid_path).context(format!("could not create {:?}", &uuid_path))?;
        let hat_data = self.gen_hat_data(HatSaveType::File);
        let mut zip_writer = ZipWriter::new(file);
        let options = SimpleFileOptions::default();

        zip_writer
            .add_directory("images", options)
            .context("could not add images directory")?;

        for (element_data, element) in hat_data.elements.iter().zip(self.elements()) {
            let mut bitmap_png_data = vec![];
            element
                .bitmap()
                .to_png_bytes(&mut bitmap_png_data)
                .context(format!(
                    "could not convert image at {:?} to png data",
                    element.bitmap().path().unwrap_or(Path::new("[no path]"))
                ))?;
            zip_writer
                .start_file_from_path(
                    element_data.base().local_image_path.as_ref().unwrap(),
                    options,
                )
                .context("could not start adding image file")?;
            zip_writer
                .write_all(&bitmap_png_data)
                .context("could not add image file")?;
        }

        zip_writer
            .start_file("data.json", options)
            .context("could not start adding data.json file")?;
        zip_writer
            .write_all(
                serde_json::to_string_pretty(&hat_data)
                    .context("could not generate data.json")?
                    .as_bytes(),
            )
            .context("could not write data.json")?;
        zip_writer
            .finish()
            .context("could not finish writing files")?;

        if std::fs::exists(path).unwrap_or(false) {
            if let Err(err) = std::fs::remove_file(path) {
                std::fs::remove_file(&uuid_path)
                    .context(format!("could not remove file at {:?}", &uuid_path))?;
                return Err(err.into());
            }
        }
        std::fs::rename(&uuid_path, path)
            .context(format!("could not rename file at {:?}", &uuid_path))?;
        Ok(())
    }

    hat_by_type_def!(wereable, WearableHat, HatType::Wearable);
    hat_by_type_def!(wings, WingsHat, HatType::Wings);
    hat_by_type_def!(extra, ExtraHat, HatType::Extra);
    hat_by_type_def!(flying_pet, FlyingPetHat, HatType::FlyingPet);
    hat_by_type_def!(walking_pet, WalkingPetHat, HatType::WalkingPet);

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn path_mut(&mut self) -> &mut PathBuf {
        &mut self.path
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn name_mut(&mut self) -> &mut String {
        &mut self.name
    }

    pub fn name_set_by_user(&self) -> bool {
        self.name_set_by_user
    }

    pub fn name_set_by_user_mut(&mut self) -> &mut bool {
        &mut self.name_set_by_user
    }

    pub fn id(&self) -> HatId {
        self.id
    }
}
