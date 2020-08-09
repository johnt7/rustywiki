use std::{
    borrow::Cow,
    error,
    fs,
    path::Path
};

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

// TODO - centralize wiki local file path handling??
// TODO - where do I get load
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
    let pbase = Path::new("site/wiki/").join(&vers.page);

    let vinfo = serde_json::to_string_pretty(vers)?;
    let all = join_version(&vinfo, data);

//    let pver = pbase.join(&vers.revision);
    fs::write(pbase.join(&vers.revision), &all)?;
//    let cver = pbase.join("current");
    fs::write(pbase.join("current"), &all)?;
    Ok(())
    // write to /wiki/input.page/current
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

