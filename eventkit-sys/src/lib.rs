// bindgen generated rust code from objective C causes a lot of warnings
// so we just turn off the relevant warnings for the syscrate with the
// understanding that it will be wrapped later on :)
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

include!(concat!(env!("OUT_DIR"), "/eventkit.rs"));
