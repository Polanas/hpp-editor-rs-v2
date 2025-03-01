use anyhow::{Result, anyhow, bail};
use bevy_math::IVec2;
use log::warn;
use pixas::bitmap::Bitmap;
use std::path::{Path, PathBuf};

use asefile::AsepriteFile;

use crate::animations::{AnimType, Animation, Frame};

#[derive(Debug)]
pub enum Image {
    Bitmap(Bitmap),
    Aseprite(Box<AsepriteFile>, PathBuf),
}

pub fn bitmap_from_ase(ase_file: &AsepriteFile) -> Bitmap {
    let frames: Vec<_> = (0..ase_file.num_frames())
        .map(|n| ase_file.frame(n))
        .map(|f| {
            let bitmap = Bitmap::from_bytes(
                ase_file.width() as _,
                ase_file.height() as _,
                f.image().as_ref(),
            );
            bitmap
        })
        .collect();
    let size_scale_x = (frames.len() as f64).sqrt().ceil() as i32;
    //we can shave off the last row, but only if it's empty
    let size_scale_y = {
        let max_frames = size_scale_x * size_scale_x;
        let current_frames = frames.len() as i32;
        let row_of_frames = size_scale_x;
        let empty_frames = max_frames - current_frames;
        if empty_frames >= row_of_frames {
            size_scale_x - 1
        } else {
            size_scale_x
        }
    };
    let mut bitmap = Bitmap::empty(
        (ase_file.width() as i32) * size_scale_x,
        (ase_file.height() as i32) * size_scale_y,
    );
    for x in 0..size_scale_x {
        for y in 0..size_scale_y {
            let Some(frame) = &frames.get((y * size_scale_x + x) as usize) else {
                break;
            };
            bitmap.draw(
                frame,
                x * (ase_file.width() as i32),
                y * (ase_file.height() as i32),
            );
        }
    }
    bitmap
}

pub struct AsepriteData {
    pub frame_size: IVec2,
    pub animations: Vec<Animation>,
}

impl Image {
    pub fn new(path: &Path) -> Result<Self> {
        match path
            .extension()
            .ok_or_else(|| anyhow!("could not get file extension"))
            .map(|s| s.to_string_lossy().to_string())?
            .as_str()
        {
            "png" => Ok(Image::Bitmap(Bitmap::from_path(path)?)),
            "aseprite" => Ok(Image::Aseprite(
                AsepriteFile::read_file(path)?.into(),
                path.to_path_buf(),
            )),
            _ => bail!("expected png or aseprite extension"),
        }
    }

    pub fn to_bitmap_with_data(self) -> (Bitmap, Option<AsepriteData>) {
        match self {
            Image::Bitmap(bitmap) => (bitmap, None),
            Image::Aseprite(ref aseprite_file, _) => {
                let aseprite_data = self.aseprite_data();
                (bitmap_from_ase(&aseprite_file), aseprite_data)
            }
        }
    }

    pub fn aseprite_data(&self) -> Option<AsepriteData> {
        let Image::Aseprite(ase_file, _) = self else {
            return None;
        };
        let num_tags = ase_file.num_tags();
        Some(AsepriteData {
            frame_size: IVec2::new(ase_file.width() as _, ase_file.height() as _),
            animations: (0..num_tags)
                .map(|i| ase_file.tag(i))
                .filter_map(|t| match t.name().to_string().to_lowercase().as_str() {
                    "ondefault" => Some((t, AnimType::OnDefault)),
                    "onpressquack" => Some((t, AnimType::OnPressQuack)),
                    "onreleasequack" => Some((t, AnimType::OnReleaseQuack)),
                    "onpetstop" => Some((t, AnimType::OnPetStop)),
                    "onpetapproach" => Some((t, AnimType::OnPetApproach)),
                    "onduckdeath" => Some((t, AnimType::OnDuckDeath)),
                    "onduckjump" => Some((t, AnimType::OnDuckJump)),
                    "onduckland" => Some((t, AnimType::OnDuckLand)),
                    "onduckglide" => Some((t, AnimType::OnDuckGlide)),
                    "onduckwalk" => Some((t, AnimType::OnDuckWalk)),
                    "onducksneak" => Some((t, AnimType::OnDuckSneak)),
                    "onducknetted" => Some((t, AnimType::OnDuckNetted)),
                    "onduckspawned" => Some((t, AnimType::OnDuckSpawned)),
                    "onhatpickedup" => Some((t, AnimType::OnHatPickedUp)),
                    other => {
                        warn!("encountered unknown animation: {}", other);
                        None
                    }
                })
                .map(|(t, anim_type)| {
                    Animation::new(
                        anim_type,
                        -1.,
                        false,
                        (t.from_frame()..t.to_frame())
                            .map(|f| {
                                Frame::with_delay(f, ase_file.frame(f).duration() as f32 / 1000.0)
                            })
                            .collect(),
                    )
                })
                .collect::<Vec<_>>(),
        })
    }

    pub fn width(&self) -> i32 {
        match self {
            Image::Bitmap(bitmap) => bitmap.width(),
            Image::Aseprite(aseprite_file, _) => aseprite_file.width() as _,
        }
    }

    pub fn height(&self) -> i32 {
        match self {
            Image::Bitmap(bitmap) => bitmap.height(),
            Image::Aseprite(aseprite_file, _) => aseprite_file.height() as _,
        }
    }

    pub fn path(&self) -> Option<&Path> {
        match self {
            Image::Bitmap(bitmap) => bitmap.path(),
            Image::Aseprite(_, path_buf) => Some(path_buf),
        }
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        match self {
            Image::Bitmap(bitmap) => bitmap.save(path),
            Image::Aseprite(aseprite_file, _) => bitmap_from_ase(aseprite_file).save(path),
        }
    }
}
