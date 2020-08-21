use std::{
    borrow::Cow,
    error,
    fs,
    ops::Deref,
    path::{Path, PathBuf},
    sync::RwLock
};
use super::wikifile;

const DELIMETER : &str = "<!--REVISION HEADER DEMARCATION>";

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct PageRevisionStruct {
	pub page : String,
	pub revision : String,
	pub previous_revision : String,
	pub create_date : String,
	pub revision_date : String,
	pub revised_by : String,
	pub comment : String,
    #[serde(default)]
    #[serde(skip_serializing)]
    pub lock : String,
    #[serde(default)]
    #[serde(skip_serializing)]
    pub data : String
}

impl PageRevisionStruct {
    // clean the page field to make sure it is valid
    fn clean(&mut self, case_sense: bool) {
        let res = match case_sense {
            true => Cow::from(&self.page),
            false => Cow::from(self.page.to_lowercase())
        };
        let res = res.replace(".", "");
        let res = res.replace("/", "");
        self.page = res.replace("\\", "");
    }
}

/// Tries to load a tinywiki file, splitting it into two string, the content and the version info
pub fn load_parts<P: AsRef<Path>>(path: P) -> Result<(String, PageRevisionStruct), Box<dyn error::Error>> {
    let fs = fs::read_to_string(path)?;
    let res = split_version(&fs)?;
    let mut header: PageRevisionStruct = serde_json::from_str(res.0)?;
    header.clean(true);
    Ok((res.1.to_string(), header))
}

/// Takes a revision structure and data and writes them to the wiki
pub fn write_parts(vers: &PageRevisionStruct, data: &str) -> Result<(), Box<dyn error::Error>> {
    // generate location for file
    let pbase = wikifile::get_path("wiki").join(&vers.page);

    // generate the data to write
    let vinfo = serde_json::to_string_pretty(vers)?;
    let all = join_version(&vinfo, data);

    // write to current and to the revision number
    fs::write(pbase.join(&vers.revision), &all)?;
    fs::write(pbase.join("current"), &all)?;
    Ok(())
 }

/// Takes a file in wiki format and divides it into the version information and data
fn split_version<'a>(in_str : &'a str) -> Result<(&'a str, &'a str), &'a str> {
    let v: Vec<&str> = in_str.split(DELIMETER).collect();
    match v.len() {
        2 => Ok( (v[0], v[1]) ),
        _ => Err("Bad versioned string")
    }
}



fn join_version(ver_str : &str, data_str : &str) -> String {
    let mut res = String::with_capacity(ver_str.len()+data_str.len()+11);
    res.push_str(ver_str);
    res.push_str("\n");
    res.push_str(DELIMETER);
    res.push_str("\n");
    res.push_str(data_str);
    res
}



#[derive(Deserialize, Serialize, Debug)]
/// container used to store data and version for it, such as config and user data
pub struct WikiContainer<S> {
    pub data : S,
    pub header : PageRevisionStruct
}

/// Structure used to store containers in Rocket to allow shared mutability
pub struct WikiStruct<S> (pub RwLock<WikiContainer<S>>);
impl<S> Deref for WikiStruct<S> {
    type Target = RwLock<WikiContainer<S>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

static mut FILE_ROOT: Option<String> = None;

pub fn set_path(root_path: String) {
    unsafe {
        FILE_ROOT = Some(root_path);
    }
}
pub fn get_path(path_ext: &str) -> PathBuf {
    unsafe {
        PathBuf::from(FILE_ROOT.as_ref().unwrap()).join(path_ext)
    }
}