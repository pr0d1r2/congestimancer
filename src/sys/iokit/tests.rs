use std::io;

use objc2_core_foundation::{
    CFBoolean, CFDictionary, CFNumber, CFString, CFType,
};

use super::{Service, bsd_service, plane_name, props_of, services, value_of};
use crate::sys::props::{PropValue, Props};

#[test]
fn scalars_convert_to_their_kind() {
    let int = CFNumber::new_i64(-7);
    let float = CFNumber::new_f64(0.25);
    let text = CFString::from_str("AC Power");
    assert_eq!(value_of(&int), Some(PropValue::Int(-7)));
    assert_eq!(value_of(&float), Some(PropValue::Float(0.25)));
    assert_eq!(value_of(CFBoolean::new(true)), Some(PropValue::Bool(true)));
    assert_eq!(value_of(&text), Some(PropValue::Text("AC Power".into())));
}

#[test]
fn a_nested_dictionary_is_not_a_scalar() {
    let dict = CFDictionary::<CFType, CFType>::empty();
    assert_eq!(value_of(&dict), None);
}

#[test]
fn a_dictionary_becomes_a_flat_table() {
    let (k1, k2, k3) = (
        CFString::from_str("Bytes (Read)"),
        CFString::from_str("Nested"),
        CFNumber::new_i64(1),
    );
    let (v1, v2) = (
        CFNumber::new_i64(42),
        CFDictionary::<CFType, CFType>::empty(),
    );
    let keys: [&CFType; 3] = [&k1, &k2, &k3];
    let values: [&CFType; 3] = [&v1, &v2, &v1];
    let dict = CFDictionary::from_slices(&keys, &values);
    let want = Props::from([("Bytes (Read)".to_owned(), PropValue::Int(42))]);
    assert_eq!(props_of(dict.as_opaque()), want);
}

#[test]
fn a_plane_name_is_nul_terminated_and_cut_to_fit() {
    let name = plane_name(c"IOService");
    assert_eq!(name.get(..10).map(<[_]>::len), Some(10));
    assert_eq!(name.get(8), Some(&(b'e' as libc::c_char)));
    assert_eq!(name.get(9), Some(&0));
    let long = std::ffi::CString::new(vec![b'x'; 200]).unwrap_or_default();
    assert_eq!(plane_name(&long).last(), Some(&0));
}

#[test]
fn the_null_object_is_no_service() {
    assert!(Service::new(0).is_none());
}

#[test]
fn an_unknown_class_matches_nothing() {
    let found = services(c"CgmrNoSuchClass").map(|s| s.len());
    assert_eq!(found.ok(), Some(0));
}

#[test]
fn an_unknown_bsd_name_is_unsupported() {
    let err = bsd_service(c"cgmrnodisk9").err();
    assert_eq!(err.map(|e| e.kind()), Some(io::ErrorKind::Unsupported));
}

#[test]
fn host_block_storage_walks_the_plane() {
    let drivers = services(c"IOBlockStorageDriver").unwrap_or_default();
    let first = drivers.first();
    assert!(first.is_some_and(|d| d.conforms_to(c"IOBlockStorageDriver")));
    assert!(first.is_some_and(|d| d.props("Statistics").is_some()));
    assert!(first.and_then(Service::parent).is_some());
    let media = first.and_then(Service::child);
    assert!(media.is_some_and(|m| m.property("BSD Name").is_some()));
}
