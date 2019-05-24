// This file was generated by gir (https://github.com/gtk-rs/gir)
// from gir-files (https://github.com/gtk-rs/gir-files)
// DO NOT EDIT

use Error;
use Repo;
use gio;
use glib;
use glib::GString;
use glib::object::IsA;
use glib::translate::*;
use ostree_sys;
use std::fmt;
use std::mem;
use std::ptr;

glib_wrapper! {
    pub struct RepoFile(Object<ostree_sys::OstreeRepoFile, ostree_sys::OstreeRepoFileClass, RepoFileClass>) @implements gio::File;

    match fn {
        get_type => || ostree_sys::ostree_repo_file_get_type(),
    }
}

pub const NONE_REPO_FILE: Option<&RepoFile> = None;

pub trait RepoFileExt: 'static {
    fn ensure_resolved(&self) -> Result<(), Error>;

    fn get_checksum(&self) -> Option<GString>;

    fn get_repo(&self) -> Option<Repo>;

    fn get_root(&self) -> Option<RepoFile>;

    fn get_xattrs<P: IsA<gio::Cancellable>>(&self, cancellable: Option<&P>) -> Result<glib::Variant, Error>;

    fn tree_find_child(&self, name: &str) -> (i32, bool, glib::Variant);

    fn tree_get_contents(&self) -> Option<glib::Variant>;

    fn tree_get_contents_checksum(&self) -> Option<GString>;

    fn tree_get_metadata(&self) -> Option<glib::Variant>;

    fn tree_get_metadata_checksum(&self) -> Option<GString>;

    fn tree_query_child<P: IsA<gio::Cancellable>>(&self, n: i32, attributes: &str, flags: gio::FileQueryInfoFlags, cancellable: Option<&P>) -> Result<gio::FileInfo, Error>;

    fn tree_set_metadata(&self, checksum: &str, metadata: &glib::Variant);
}

impl<O: IsA<RepoFile>> RepoFileExt for O {
    fn ensure_resolved(&self) -> Result<(), Error> {
        unsafe {
            let mut error = ptr::null_mut();
            let _ = ostree_sys::ostree_repo_file_ensure_resolved(self.as_ref().to_glib_none().0, &mut error);
            if error.is_null() { Ok(()) } else { Err(from_glib_full(error)) }
        }
    }

    fn get_checksum(&self) -> Option<GString> {
        unsafe {
            from_glib_none(ostree_sys::ostree_repo_file_get_checksum(self.as_ref().to_glib_none().0))
        }
    }

    fn get_repo(&self) -> Option<Repo> {
        unsafe {
            from_glib_none(ostree_sys::ostree_repo_file_get_repo(self.as_ref().to_glib_none().0))
        }
    }

    fn get_root(&self) -> Option<RepoFile> {
        unsafe {
            from_glib_none(ostree_sys::ostree_repo_file_get_root(self.as_ref().to_glib_none().0))
        }
    }

    fn get_xattrs<P: IsA<gio::Cancellable>>(&self, cancellable: Option<&P>) -> Result<glib::Variant, Error> {
        unsafe {
            let mut out_xattrs = ptr::null_mut();
            let mut error = ptr::null_mut();
            let _ = ostree_sys::ostree_repo_file_get_xattrs(self.as_ref().to_glib_none().0, &mut out_xattrs, cancellable.map(|p| p.as_ref()).to_glib_none().0, &mut error);
            if error.is_null() { Ok(from_glib_full(out_xattrs)) } else { Err(from_glib_full(error)) }
        }
    }

    fn tree_find_child(&self, name: &str) -> (i32, bool, glib::Variant) {
        unsafe {
            let mut is_dir = mem::uninitialized();
            let mut out_container = ptr::null_mut();
            let ret = ostree_sys::ostree_repo_file_tree_find_child(self.as_ref().to_glib_none().0, name.to_glib_none().0, &mut is_dir, &mut out_container);
            (ret, from_glib(is_dir), from_glib_full(out_container))
        }
    }

    fn tree_get_contents(&self) -> Option<glib::Variant> {
        unsafe {
            from_glib_full(ostree_sys::ostree_repo_file_tree_get_contents(self.as_ref().to_glib_none().0))
        }
    }

    fn tree_get_contents_checksum(&self) -> Option<GString> {
        unsafe {
            from_glib_none(ostree_sys::ostree_repo_file_tree_get_contents_checksum(self.as_ref().to_glib_none().0))
        }
    }

    fn tree_get_metadata(&self) -> Option<glib::Variant> {
        unsafe {
            from_glib_full(ostree_sys::ostree_repo_file_tree_get_metadata(self.as_ref().to_glib_none().0))
        }
    }

    fn tree_get_metadata_checksum(&self) -> Option<GString> {
        unsafe {
            from_glib_none(ostree_sys::ostree_repo_file_tree_get_metadata_checksum(self.as_ref().to_glib_none().0))
        }
    }

    fn tree_query_child<P: IsA<gio::Cancellable>>(&self, n: i32, attributes: &str, flags: gio::FileQueryInfoFlags, cancellable: Option<&P>) -> Result<gio::FileInfo, Error> {
        unsafe {
            let mut out_info = ptr::null_mut();
            let mut error = ptr::null_mut();
            let _ = ostree_sys::ostree_repo_file_tree_query_child(self.as_ref().to_glib_none().0, n, attributes.to_glib_none().0, flags.to_glib(), &mut out_info, cancellable.map(|p| p.as_ref()).to_glib_none().0, &mut error);
            if error.is_null() { Ok(from_glib_full(out_info)) } else { Err(from_glib_full(error)) }
        }
    }

    fn tree_set_metadata(&self, checksum: &str, metadata: &glib::Variant) {
        unsafe {
            ostree_sys::ostree_repo_file_tree_set_metadata(self.as_ref().to_glib_none().0, checksum.to_glib_none().0, metadata.to_glib_none().0);
        }
    }
}

impl fmt::Display for RepoFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "RepoFile")
    }
}