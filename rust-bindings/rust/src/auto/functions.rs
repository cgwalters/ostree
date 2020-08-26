// This file was generated by gir (https://github.com/gtk-rs/gir)
// from gir-files (https://github.com/gtk-rs/gir-files)
// DO NOT EDIT

use gio;
use glib;
use glib::object::IsA;
use glib::translate::*;
use glib::GString;
use ostree_sys;
use std::mem;
use std::ptr;
#[cfg(any(feature = "v2020_1", feature = "dox"))]
use CommitSizesEntry;
use DiffFlags;
use DiffItem;
use ObjectType;


#[cfg(any(feature = "v2017_15", feature = "dox"))]
pub fn break_hardlink<P: IsA<gio::Cancellable>>(dfd: i32, path: &str, skip_xattrs: bool, cancellable: Option<&P>) -> Result<(), glib::Error> {
    unsafe {
        let mut error = ptr::null_mut();
        let _ = ostree_sys::ostree_break_hardlink(dfd, path.to_glib_none().0, skip_xattrs.to_glib(), cancellable.map(|p| p.as_ref()).to_glib_none().0, &mut error);
        if error.is_null() { Ok(()) } else { Err(from_glib_full(error)) }
    }
}

#[cfg(any(feature = "v2017_4", feature = "dox"))]
pub fn check_version(required_year: u32, required_release: u32) -> bool {
    unsafe {
        from_glib(ostree_sys::ostree_check_version(required_year, required_release))
    }
}

//pub fn checksum_bytes_peek(bytes: &glib::Variant) -> /*Unimplemented*/FixedArray TypeId { ns_id: 0, id: 3 }; 32 {
//    unsafe { TODO: call ostree_sys:ostree_checksum_bytes_peek() }
//}

//pub fn checksum_bytes_peek_validate(bytes: &glib::Variant) -> Result</*Unimplemented*/FixedArray TypeId { ns_id: 0, id: 3 }; 32, glib::Error> {
//    unsafe { TODO: call ostree_sys:ostree_checksum_bytes_peek_validate() }
//}

pub fn checksum_from_bytes_v(csum_v: &glib::Variant) -> Option<GString> {
    unsafe {
        from_glib_full(ostree_sys::ostree_checksum_from_bytes_v(csum_v.to_glib_none().0))
    }
}

pub fn checksum_to_bytes_v(checksum: &str) -> Option<glib::Variant> {
    unsafe {
        from_glib_full(ostree_sys::ostree_checksum_to_bytes_v(checksum.to_glib_none().0))
    }
}

#[cfg(any(feature = "v2018_2", feature = "dox"))]
pub fn commit_get_content_checksum(commit_variant: &glib::Variant) -> Option<GString> {
    unsafe {
        from_glib_full(ostree_sys::ostree_commit_get_content_checksum(commit_variant.to_glib_none().0))
    }
}

#[cfg(any(feature = "v2020_1", feature = "dox"))]
pub fn commit_get_object_sizes(commit_variant: &glib::Variant) -> Result<Vec<CommitSizesEntry>, glib::Error> {
    unsafe {
        let mut out_sizes_entries = ptr::null_mut();
        let mut error = ptr::null_mut();
        let _ = ostree_sys::ostree_commit_get_object_sizes(commit_variant.to_glib_none().0, &mut out_sizes_entries, &mut error);
        if error.is_null() { Ok(FromGlibPtrContainer::from_glib_container(out_sizes_entries)) } else { Err(from_glib_full(error)) }
    }
}

pub fn commit_get_parent(commit_variant: &glib::Variant) -> Option<GString> {
    unsafe {
        from_glib_full(ostree_sys::ostree_commit_get_parent(commit_variant.to_glib_none().0))
    }
}

#[cfg(any(feature = "v2016_3", feature = "dox"))]
pub fn commit_get_timestamp(commit_variant: &glib::Variant) -> u64 {
    unsafe {
        ostree_sys::ostree_commit_get_timestamp(commit_variant.to_glib_none().0)
    }
}

pub fn content_file_parse<P: IsA<gio::File>, Q: IsA<gio::Cancellable>>(compressed: bool, content_path: &P, trusted: bool, cancellable: Option<&Q>) -> Result<(gio::InputStream, gio::FileInfo, glib::Variant), glib::Error> {
    unsafe {
        let mut out_input = ptr::null_mut();
        let mut out_file_info = ptr::null_mut();
        let mut out_xattrs = ptr::null_mut();
        let mut error = ptr::null_mut();
        let _ = ostree_sys::ostree_content_file_parse(compressed.to_glib(), content_path.as_ref().to_glib_none().0, trusted.to_glib(), &mut out_input, &mut out_file_info, &mut out_xattrs, cancellable.map(|p| p.as_ref()).to_glib_none().0, &mut error);
        if error.is_null() { Ok((from_glib_full(out_input), from_glib_full(out_file_info), from_glib_full(out_xattrs))) } else { Err(from_glib_full(error)) }
    }
}

pub fn content_file_parse_at<P: IsA<gio::Cancellable>>(compressed: bool, parent_dfd: i32, path: &str, trusted: bool, cancellable: Option<&P>) -> Result<(gio::InputStream, gio::FileInfo, glib::Variant), glib::Error> {
    unsafe {
        let mut out_input = ptr::null_mut();
        let mut out_file_info = ptr::null_mut();
        let mut out_xattrs = ptr::null_mut();
        let mut error = ptr::null_mut();
        let _ = ostree_sys::ostree_content_file_parse_at(compressed.to_glib(), parent_dfd, path.to_glib_none().0, trusted.to_glib(), &mut out_input, &mut out_file_info, &mut out_xattrs, cancellable.map(|p| p.as_ref()).to_glib_none().0, &mut error);
        if error.is_null() { Ok((from_glib_full(out_input), from_glib_full(out_file_info), from_glib_full(out_xattrs))) } else { Err(from_glib_full(error)) }
    }
}

pub fn content_stream_parse<P: IsA<gio::InputStream>, Q: IsA<gio::Cancellable>>(compressed: bool, input: &P, input_length: u64, trusted: bool, cancellable: Option<&Q>) -> Result<(gio::InputStream, gio::FileInfo, glib::Variant), glib::Error> {
    unsafe {
        let mut out_input = ptr::null_mut();
        let mut out_file_info = ptr::null_mut();
        let mut out_xattrs = ptr::null_mut();
        let mut error = ptr::null_mut();
        let _ = ostree_sys::ostree_content_stream_parse(compressed.to_glib(), input.as_ref().to_glib_none().0, input_length, trusted.to_glib(), &mut out_input, &mut out_file_info, &mut out_xattrs, cancellable.map(|p| p.as_ref()).to_glib_none().0, &mut error);
        if error.is_null() { Ok((from_glib_full(out_input), from_glib_full(out_file_info), from_glib_full(out_xattrs))) } else { Err(from_glib_full(error)) }
    }
}

pub fn create_directory_metadata(dir_info: &gio::FileInfo, xattrs: Option<&glib::Variant>) -> Option<glib::Variant> {
    unsafe {
        from_glib_full(ostree_sys::ostree_create_directory_metadata(dir_info.to_glib_none().0, xattrs.to_glib_none().0))
    }
}

pub fn diff_dirs<P: IsA<gio::File>, Q: IsA<gio::File>, R: IsA<gio::Cancellable>>(flags: DiffFlags, a: &P, b: &Q, modified: &[&DiffItem], removed: &[gio::File], added: &[gio::File], cancellable: Option<&R>) -> Result<(), glib::Error> {
    unsafe {
        let mut error = ptr::null_mut();
        let _ = ostree_sys::ostree_diff_dirs(flags.to_glib(), a.as_ref().to_glib_none().0, b.as_ref().to_glib_none().0, modified.to_glib_none().0, removed.to_glib_none().0, added.to_glib_none().0, cancellable.map(|p| p.as_ref()).to_glib_none().0, &mut error);
        if error.is_null() { Ok(()) } else { Err(from_glib_full(error)) }
    }
}

//#[cfg(any(feature = "v2017_4", feature = "dox"))]
//pub fn diff_dirs_with_options<P: IsA<gio::File>, Q: IsA<gio::File>, R: IsA<gio::Cancellable>>(flags: DiffFlags, a: &P, b: &Q, modified: &[&DiffItem], removed: &[gio::File], added: &[gio::File], options: /*Ignored*/Option<&mut DiffDirsOptions>, cancellable: Option<&R>) -> Result<(), glib::Error> {
//    unsafe { TODO: call ostree_sys:ostree_diff_dirs_with_options() }
//}

pub fn diff_print<P: IsA<gio::File>, Q: IsA<gio::File>>(a: &P, b: &Q, modified: &[&DiffItem], removed: &[gio::File], added: &[gio::File]) {
    unsafe {
        ostree_sys::ostree_diff_print(a.as_ref().to_glib_none().0, b.as_ref().to_glib_none().0, modified.to_glib_none().0, removed.to_glib_none().0, added.to_glib_none().0);
    }
}

#[cfg(any(feature = "v2017_10", feature = "dox"))]
pub fn gpg_error_quark() -> glib::Quark {
    unsafe {
        from_glib(ostree_sys::ostree_gpg_error_quark())
    }
}

pub fn metadata_variant_type(objtype: ObjectType) -> Option<glib::VariantType> {
    unsafe {
        from_glib_none(ostree_sys::ostree_metadata_variant_type(objtype.to_glib()))
    }
}

pub fn object_from_string(str: &str) -> (GString, ObjectType) {
    unsafe {
        let mut out_checksum = ptr::null_mut();
        let mut out_objtype = mem::MaybeUninit::uninit();
        ostree_sys::ostree_object_from_string(str.to_glib_none().0, &mut out_checksum, out_objtype.as_mut_ptr());
        let out_objtype = out_objtype.assume_init();
        (from_glib_full(out_checksum), from_glib(out_objtype))
    }
}

pub fn object_name_deserialize(variant: &glib::Variant) -> (GString, ObjectType) {
    unsafe {
        let mut out_checksum = ptr::null();
        let mut out_objtype = mem::MaybeUninit::uninit();
        ostree_sys::ostree_object_name_deserialize(variant.to_glib_none().0, &mut out_checksum, out_objtype.as_mut_ptr());
        let out_objtype = out_objtype.assume_init();
        (from_glib_none(out_checksum), from_glib(out_objtype))
    }
}

pub fn object_name_serialize(checksum: &str, objtype: ObjectType) -> Option<glib::Variant> {
    unsafe {
        from_glib_none(ostree_sys::ostree_object_name_serialize(checksum.to_glib_none().0, objtype.to_glib()))
    }
}

pub fn object_to_string(checksum: &str, objtype: ObjectType) -> Option<GString> {
    unsafe {
        from_glib_full(ostree_sys::ostree_object_to_string(checksum.to_glib_none().0, objtype.to_glib()))
    }
}

pub fn object_type_from_string(str: &str) -> ObjectType {
    unsafe {
        from_glib(ostree_sys::ostree_object_type_from_string(str.to_glib_none().0))
    }
}

pub fn object_type_to_string(objtype: ObjectType) -> Option<GString> {
    unsafe {
        from_glib_none(ostree_sys::ostree_object_type_to_string(objtype.to_glib()))
    }
}

pub fn parse_refspec(refspec: &str) -> Result<(Option<GString>, GString), glib::Error> {
    unsafe {
        let mut out_remote = ptr::null_mut();
        let mut out_ref = ptr::null_mut();
        let mut error = ptr::null_mut();
        let _ = ostree_sys::ostree_parse_refspec(refspec.to_glib_none().0, &mut out_remote, &mut out_ref, &mut error);
        if error.is_null() { Ok((from_glib_full(out_remote), from_glib_full(out_ref))) } else { Err(from_glib_full(error)) }
    }
}

#[cfg(any(feature = "v2016_6", feature = "dox"))]
pub fn raw_file_to_archive_z2_stream<P: IsA<gio::InputStream>, Q: IsA<gio::Cancellable>>(input: &P, file_info: &gio::FileInfo, xattrs: Option<&glib::Variant>, cancellable: Option<&Q>) -> Result<gio::InputStream, glib::Error> {
    unsafe {
        let mut out_input = ptr::null_mut();
        let mut error = ptr::null_mut();
        let _ = ostree_sys::ostree_raw_file_to_archive_z2_stream(input.as_ref().to_glib_none().0, file_info.to_glib_none().0, xattrs.to_glib_none().0, &mut out_input, cancellable.map(|p| p.as_ref()).to_glib_none().0, &mut error);
        if error.is_null() { Ok(from_glib_full(out_input)) } else { Err(from_glib_full(error)) }
    }
}

#[cfg(any(feature = "v2017_3", feature = "dox"))]
pub fn raw_file_to_archive_z2_stream_with_options<P: IsA<gio::InputStream>, Q: IsA<gio::Cancellable>>(input: &P, file_info: &gio::FileInfo, xattrs: Option<&glib::Variant>, options: Option<&glib::Variant>, cancellable: Option<&Q>) -> Result<gio::InputStream, glib::Error> {
    unsafe {
        let mut out_input = ptr::null_mut();
        let mut error = ptr::null_mut();
        let _ = ostree_sys::ostree_raw_file_to_archive_z2_stream_with_options(input.as_ref().to_glib_none().0, file_info.to_glib_none().0, xattrs.to_glib_none().0, options.to_glib_none().0, &mut out_input, cancellable.map(|p| p.as_ref()).to_glib_none().0, &mut error);
        if error.is_null() { Ok(from_glib_full(out_input)) } else { Err(from_glib_full(error)) }
    }
}

pub fn raw_file_to_content_stream<P: IsA<gio::InputStream>, Q: IsA<gio::Cancellable>>(input: &P, file_info: &gio::FileInfo, xattrs: Option<&glib::Variant>, cancellable: Option<&Q>) -> Result<(gio::InputStream, u64), glib::Error> {
    unsafe {
        let mut out_input = ptr::null_mut();
        let mut out_length = mem::MaybeUninit::uninit();
        let mut error = ptr::null_mut();
        let _ = ostree_sys::ostree_raw_file_to_content_stream(input.as_ref().to_glib_none().0, file_info.to_glib_none().0, xattrs.to_glib_none().0, &mut out_input, out_length.as_mut_ptr(), cancellable.map(|p| p.as_ref()).to_glib_none().0, &mut error);
        let out_length = out_length.assume_init();
        if error.is_null() { Ok((from_glib_full(out_input), out_length)) } else { Err(from_glib_full(error)) }
    }
}

pub fn validate_checksum_string(sha256: &str) -> Result<(), glib::Error> {
    unsafe {
        let mut error = ptr::null_mut();
        let _ = ostree_sys::ostree_validate_checksum_string(sha256.to_glib_none().0, &mut error);
        if error.is_null() { Ok(()) } else { Err(from_glib_full(error)) }
    }
}

#[cfg(any(feature = "v2018_6", feature = "dox"))]
pub fn validate_collection_id(collection_id: Option<&str>) -> Result<(), glib::Error> {
    unsafe {
        let mut error = ptr::null_mut();
        let _ = ostree_sys::ostree_validate_collection_id(collection_id.to_glib_none().0, &mut error);
        if error.is_null() { Ok(()) } else { Err(from_glib_full(error)) }
    }
}

#[cfg(any(feature = "v2017_8", feature = "dox"))]
pub fn validate_remote_name(remote_name: &str) -> Result<(), glib::Error> {
    unsafe {
        let mut error = ptr::null_mut();
        let _ = ostree_sys::ostree_validate_remote_name(remote_name.to_glib_none().0, &mut error);
        if error.is_null() { Ok(()) } else { Err(from_glib_full(error)) }
    }
}

pub fn validate_rev(rev: &str) -> Result<(), glib::Error> {
    unsafe {
        let mut error = ptr::null_mut();
        let _ = ostree_sys::ostree_validate_rev(rev.to_glib_none().0, &mut error);
        if error.is_null() { Ok(()) } else { Err(from_glib_full(error)) }
    }
}

pub fn validate_structureof_checksum_string(checksum: &str) -> Result<(), glib::Error> {
    unsafe {
        let mut error = ptr::null_mut();
        let _ = ostree_sys::ostree_validate_structureof_checksum_string(checksum.to_glib_none().0, &mut error);
        if error.is_null() { Ok(()) } else { Err(from_glib_full(error)) }
    }
}

pub fn validate_structureof_commit(commit: &glib::Variant) -> Result<(), glib::Error> {
    unsafe {
        let mut error = ptr::null_mut();
        let _ = ostree_sys::ostree_validate_structureof_commit(commit.to_glib_none().0, &mut error);
        if error.is_null() { Ok(()) } else { Err(from_glib_full(error)) }
    }
}

pub fn validate_structureof_csum_v(checksum: &glib::Variant) -> Result<(), glib::Error> {
    unsafe {
        let mut error = ptr::null_mut();
        let _ = ostree_sys::ostree_validate_structureof_csum_v(checksum.to_glib_none().0, &mut error);
        if error.is_null() { Ok(()) } else { Err(from_glib_full(error)) }
    }
}

pub fn validate_structureof_dirmeta(dirmeta: &glib::Variant) -> Result<(), glib::Error> {
    unsafe {
        let mut error = ptr::null_mut();
        let _ = ostree_sys::ostree_validate_structureof_dirmeta(dirmeta.to_glib_none().0, &mut error);
        if error.is_null() { Ok(()) } else { Err(from_glib_full(error)) }
    }
}

pub fn validate_structureof_dirtree(dirtree: &glib::Variant) -> Result<(), glib::Error> {
    unsafe {
        let mut error = ptr::null_mut();
        let _ = ostree_sys::ostree_validate_structureof_dirtree(dirtree.to_glib_none().0, &mut error);
        if error.is_null() { Ok(()) } else { Err(from_glib_full(error)) }
    }
}

pub fn validate_structureof_file_mode(mode: u32) -> Result<(), glib::Error> {
    unsafe {
        let mut error = ptr::null_mut();
        let _ = ostree_sys::ostree_validate_structureof_file_mode(mode, &mut error);
        if error.is_null() { Ok(()) } else { Err(from_glib_full(error)) }
    }
}

pub fn validate_structureof_objtype(objtype: u8) -> Result<(), glib::Error> {
    unsafe {
        let mut error = ptr::null_mut();
        let _ = ostree_sys::ostree_validate_structureof_objtype(objtype, &mut error);
        if error.is_null() { Ok(()) } else { Err(from_glib_full(error)) }
    }
}
