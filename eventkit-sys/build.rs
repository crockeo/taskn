use std::env;
use std::path::PathBuf;

use bindgen::Builder;

fn main() {
    let builder = Builder::default()
        .clang_args(&["-x", "objective-c"])
        .block_extern_crate(true)
        .objc_extern_crate(true)
        .generate_block(true)
        .rustfmt_bindings(true)
        .blacklist_item("timezone")
        .blacklist_type("objc_object")
        // FndrOpaqueInfo is repr(packed) and embedded in other packed types
        // which causes issues for rustc
        // so we disallow it and its friends
        .blacklist_type("FndrOpaqueInfo")
        .blacklist_type("HFSCatalogFile")
        .blacklist_type("HFSPlusCatalogFile")
        .blacklist_type("HFSCatalogFolder")
        .blacklist_type("HFSPlusCatalogFolder")
        .header_contents("EventKit.h", "#include <EventKit/EventKit.h>");

    let bindings = builder.generate().unwrap();

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    bindings.write_to_file(out_dir.join("eventkit.rs")).unwrap();
}
