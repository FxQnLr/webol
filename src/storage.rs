use std::{
    ffi::OsStr,
    fs::{create_dir_all, File, read_dir},
    io::{Read, Write},
    path::Path,
};

use ipnetwork::IpNetwork;
use mac_address::MacAddress;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{debug, trace, warn};
use utoipa::ToSchema;

use crate::error::Error;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Device {
    pub id: String,
    pub mac: MacAddress,
    pub broadcast_addr: String,
    pub ip: Option<IpNetwork>,
    pub times: Option<Vec<u64>>,
}

impl Device {
    const STORAGE_PATH: &'static str = "devices";

    pub fn setup() -> Result<String, Error> {
        trace!("check for storage STORAGE_PATH=\"{}\"", Self::STORAGE_PATH);
        let sp = Path::new(Self::STORAGE_PATH);
        if !sp.exists() {
            warn!("device storage path doesn't exist, creating it");
            create_dir_all(Self::STORAGE_PATH)?;
        };

        debug!("device storage STORAGE_PATH=\"{}\"", Self::STORAGE_PATH);

        Ok(Self::STORAGE_PATH.to_string())
    }

    pub fn read(id: &str) -> Result<Self, Error> {
        trace!(?id, "attempt to read file");
        let mut file = File::open(format!("{}/{id}.json", Self::STORAGE_PATH))?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)?;
        trace!(?id, ?buf, "read successfully from file");

        let dev = serde_json::from_str(&buf)?;
        Ok(dev)
    }

    pub fn read_all() -> Result<Vec<Self>, Error> {
        trace!("attempt to read all files");
        let st_path = read_dir(Self::STORAGE_PATH)?;

        let mut devices = vec![];
        for file_path in st_path {
            let file_path = file_path?;
            if file_path.path().extension() != Some(OsStr::new("json")) { continue; };
            let mut file = File::open(file_path.path())?;
            let mut buf = String::new();
            file.read_to_string(&mut buf)?;
            trace!(?file, ?buf, "read successfully from file");
            devices.push(serde_json::from_str(&buf)?);
        }

        Ok(devices)
    }

    pub fn write(&self) -> Result<(), Error> {
        trace!(?self.id, ?self, "attempt to write to file");
        let mut file = File::create(format!("{}/{}.json", Self::STORAGE_PATH, self.id))?;
        file.write_all(json!(self).to_string().as_bytes())?;
        trace!(?self.id, "wrote successfully to file");

        Ok(())
    }
}

// Dead Code allowed because of use in OpenApi Macro (not really dead code)
#[allow(dead_code)]
#[derive(ToSchema)]
#[schema(as = Device)]
pub struct DeviceSchema {
    pub id: String,
    pub mac: String,
    pub broadcast_addr: String,
    pub ip: String,
    pub times: Option<Vec<i64>>,
}
