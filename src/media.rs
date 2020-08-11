use rocket::State;
use std::{
    error,
    fs,
    ops::Deref,
    path::PathBuf,
    sync::RwLock
};
use super::user;

#[get("/page/MediaIndex")]
pub fn rocket_route_media_index(_user: user::User, mi: State<MediaIndex>) -> String {
    debug!("media index");
    mi.read().unwrap().to_string()
}

fn dir_list(dir: PathBuf) -> Result<String, Box<dyn error::Error>> {
    let res = fs::read_dir(&dir)?;
    let out = res.map(|entry| {
            match  entry {
                Ok(e) => {
                    let p = e.path();
                    if p.is_dir() {
                        format!("\n{{\"dir\": {} }}", dir_list(p).unwrap_or("".to_string()))
                    } else {
                        format!("\n{{\"file\": {:?}}}", p.file_name().unwrap_or_default())
                    }
                },
                _ => "".to_string()
            }
         }).collect::<Vec<_>>().join(",");
     Ok(format!("{{\"path\":{:?},\"dir\": [\n{}\n]}}", dir.file_name().unwrap_or_default(), out))
}

pub fn media_str() -> String {
    dir_list(PathBuf::from("site/media")).unwrap_or("failed".to_string() )
}

pub struct MediaIndex (RwLock<String>);
impl MediaIndex {
    pub fn new() -> MediaIndex {
        MediaIndex( RwLock::new( media_str() ))
    }
}
impl Deref for MediaIndex {
    type Target = RwLock<String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
