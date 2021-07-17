mod utils;

use js_sys::Date;
use serde::{Deserialize, Serialize};

use tar::{EntryType, Header};
use utils::set_panic_hook;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

mod streams_sys {
    pub use wasm_streams::readable::sys::*;
    pub use wasm_streams::writable::sys::*;
}

#[wasm_bindgen(typescript_custom_section)]
const ITAR_ENTRY_OPTS: &'static str = r#"
interface ITarEntryOpts {
    /*
        The file mode this entry will be created with.
        Note that this is the base10 variant of the value, not base8.
        This defaults to 644 for files and 755 for directories.
    */
    mode?: number,
    /*
        The mtime attribute to attach to this entry.
        It should be in whole seconds since epoch, or a standard Unix timestamp.

        Defaults to now.
    */
    mtime?: number,
    /*
        If set, the uid to attribute with this file.
        TAR unpackers will typically create the file with inherited uid/gid attributes.
    */
    uid?: number,
    /*
        If set, the gid to attribute with this file.
        TAR unpackers will typically create the file with inherited uid/gid attributes.
    */
    gid?: number
}
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "ITarEntryOpts")]
    pub type ITarEntryOpts;
}

#[derive(Serialize, Deserialize)]
pub struct TarAddOpts {
    pub mode: Option<u32>,
    pub mtime: Option<u32>,
    pub uid: Option<u32>,
    pub gid: Option<u32>,
}

impl TarAddOpts {
    pub fn extend_header(&self, entry_type: EntryType, header: &mut Header) {
        header.set_mtime(self.mtime.unwrap_or_else(|| (Date::now() / 1000.0) as u32) as u64);
        if let Some(uid) = self.uid {
            header.set_uid(uid as u64);
        }
        if let Some(gid) = self.gid {
            header.set_gid(gid as u64);
        }
        header.set_entry_type(entry_type);
        match entry_type {
            EntryType::Regular => header.set_mode(self.mode.unwrap_or(0o644)),
            EntryType::Directory => header.set_mode(self.mode.unwrap_or(0o755)),
            _ => {
                if let Some(mode) = self.mode {
                    header.set_mode(mode);
                }
            }
        }
        header.set_cksum();
    }
}

#[wasm_bindgen]
pub struct StreamingTarPacker {
    stream: streams_sys::WritableStream,
}

#[wasm_bindgen]
impl StreamingTarPacker {
    #[wasm_bindgen(constructor)]
    pub fn new(stream: streams_sys::WritableStream) -> Self {
        set_panic_hook();
        Self { stream }
    }

    /// Adds a directory to the tarball.
    #[wasm_bindgen(js_name = "addDir")]
    pub fn add_dir(&mut self, path: String, opts: &ITarEntryOpts) -> js_sys::Promise {
        let opts: TarAddOpts = opts.into_serde().unwrap();

        let stream = self.stream.clone();
        wasm_bindgen_futures::future_to_promise(async move {
            let mut header = tar::Header::new_ustar();
            header.set_path(path).unwrap();
            opts.extend_header(EntryType::dir(), &mut header);

            let writer = stream.get_writer().unwrap();
            JsFuture::from(writer.write(js_sys::Uint8Array::from(&header.as_bytes()[..]).into()))
                .await
                .unwrap();
            writer.release_lock();
            Ok(JsValue::UNDEFINED)
        })
    }

    /// Adds a file to the tarball from a stream.
    #[wasm_bindgen(js_name = "addFileStream")]
    pub fn add_file_stream(
        &mut self,
        path: String,
        opts: &ITarEntryOpts,
        size: u32,
        reader: streams_sys::ReadableStream,
    ) -> js_sys::Promise {
        let opts: TarAddOpts = opts.into_serde().unwrap();
        let stream = self.stream.clone();
        let size = size;

        wasm_bindgen_futures::future_to_promise(async move {
            let mut header = tar::Header::new_ustar();
            header.set_path(path).unwrap();
            header.set_size(size as u64);
            opts.extend_header(EntryType::file(), &mut header);

            let writer = stream.get_writer().unwrap();
            JsFuture::from(writer.write(js_sys::Uint8Array::from(&header.as_bytes()[..]).into()))
                .await
                .unwrap();
            writer.release_lock();

            JsFuture::from(reader.pipe_to(
                &stream,
                streams_sys::PipeOptions::new(true, true, true, None),
            ))
            .await
            .unwrap();

            let padding = 512 - (size % 512);
            let writer = stream.get_writer().unwrap();
            if padding > 0 {
                JsFuture::from(
                    writer.write(
                        js_sys::Uint8Array::from(vec![0; padding as usize].as_slice()).into(),
                    ),
                )
                .await
                .unwrap();
            }
            writer.release_lock();

            Ok(JsValue::UNDEFINED)
        })
    }

    /// Finishes the pack and closes the writer. After this is called no other method may be called on the packer.
    pub fn finish(self) -> js_sys::Promise {
        let stream = self.stream;
        wasm_bindgen_futures::future_to_promise(async move {
            let writer = stream.get_writer().unwrap();
            JsFuture::from(writer.write(js_sys::Uint8Array::from(vec![0; 1024].as_slice()).into()))
                .await
                .unwrap();
            JsFuture::from(writer.close()).await.unwrap();
            writer.release_lock();
            Ok(JsValue::from(stream))
        })
    }
}
