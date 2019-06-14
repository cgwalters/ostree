use crate::{RepoCheckoutMode, RepoCheckoutOverwriteMode, RepoDevInoCache, SePolicy};
use glib::translate::*;
use libc::c_char;
use ostree_sys::*;
use std::path::PathBuf;

mod repo_checkout_filter;

pub use self::repo_checkout_filter::RepoCheckoutFilter;

pub struct RepoCheckoutAtOptions {
    pub mode: RepoCheckoutMode,
    pub overwrite_mode: RepoCheckoutOverwriteMode,
    pub enable_uncompressed_cache: bool,
    pub enable_fsync: bool,
    pub process_whiteouts: bool,
    pub no_copy_fallback: bool,
    pub force_copy: bool,
    pub bareuseronly_dirs: bool,
    pub force_copy_zerosized: bool,
    pub subpath: Option<PathBuf>,
    pub devino_to_csum_cache: Option<RepoDevInoCache>,
    /// A callback function to decide which files and directories will be checked out from the
    /// repo. See the documentation on [RepoCheckoutFilter](struct.RepoCheckoutFilter.html) for more
    /// information on the signature.
    ///
    /// # Panics
    /// This callback may not panic. If it does, `abort()` will be called to avoid unwinding across
    /// an FFI boundary and into the libostree C code (which is Undefined Behavior). If you prefer to
    /// swallow the panic rather than aborting, you can use `std::panic::catch_unwind` inside your
    /// callback to catch and silence any panics that occur.
    pub filter: Option<RepoCheckoutFilter>,
    pub sepolicy: Option<SePolicy>,
    pub sepolicy_prefix: Option<String>,
}

impl Default for RepoCheckoutAtOptions {
    fn default() -> Self {
        RepoCheckoutAtOptions {
            mode: RepoCheckoutMode::None,
            overwrite_mode: RepoCheckoutOverwriteMode::None,
            enable_uncompressed_cache: false,
            enable_fsync: false,
            process_whiteouts: false,
            no_copy_fallback: false,
            force_copy: false,
            bareuseronly_dirs: false,
            force_copy_zerosized: false,
            subpath: None,
            devino_to_csum_cache: None,
            filter: None,
            sepolicy: None,
            sepolicy_prefix: None,
        }
    }
}

type StringStash<'a, T> = Stash<'a, *const c_char, Option<T>>;
type WrapperStash<'a, GlibT, WrappedT> = Stash<'a, *mut GlibT, Option<WrappedT>>;

impl<'a> ToGlibPtr<'a, *const OstreeRepoCheckoutAtOptions> for RepoCheckoutAtOptions {
    #[allow(clippy::type_complexity)]
    type Storage = (
        Box<OstreeRepoCheckoutAtOptions>,
        StringStash<'a, PathBuf>,
        StringStash<'a, String>,
        WrapperStash<'a, OstreeRepoDevInoCache, RepoDevInoCache>,
        WrapperStash<'a, OstreeSePolicy, SePolicy>,
    );

    // We need to make sure that all memory pointed to by the returned pointer is kept alive by
    // either the `self` reference or the returned Stash.
    fn to_glib_none(&'a self) -> Stash<*const OstreeRepoCheckoutAtOptions, Self> {
        // Creating this struct from zeroed memory is fine since it's `repr(C)` and only contains
        // primitive types. In fact, the libostree docs say to zero the struct. This means we handle
        // the unused bytes correctly.
        // The struct needs to be boxed so the pointer we return remains valid even as the Stash is
        // moved around.
        let mut options = Box::new(unsafe { std::mem::zeroed::<OstreeRepoCheckoutAtOptions>() });
        options.mode = self.mode.to_glib();
        options.overwrite_mode = self.overwrite_mode.to_glib();
        options.enable_uncompressed_cache = self.enable_uncompressed_cache.to_glib();
        options.enable_fsync = self.enable_fsync.to_glib();
        options.process_whiteouts = self.process_whiteouts.to_glib();
        options.no_copy_fallback = self.no_copy_fallback.to_glib();
        options.force_copy = self.force_copy.to_glib();
        options.bareuseronly_dirs = self.bareuseronly_dirs.to_glib();
        options.force_copy_zerosized = self.force_copy_zerosized.to_glib();

        // We keep these complex values alive by returning them in our Stash. Technically, some of
        // these are being kept alive by `self` already, but it's better to be consistent here.
        let subpath = self.subpath.to_glib_none();
        options.subpath = subpath.0;
        let sepolicy_prefix = self.sepolicy_prefix.to_glib_none();
        options.sepolicy_prefix = sepolicy_prefix.0;
        let devino_to_csum_cache = self.devino_to_csum_cache.to_glib_none();
        options.devino_to_csum_cache = devino_to_csum_cache.0;
        let sepolicy = self.sepolicy.to_glib_none();
        options.sepolicy = sepolicy.0;

        if let Some(filter) = &self.filter {
            options.filter_user_data = filter.to_glib_none().0;
            options.filter = Some(repo_checkout_filter::filter_trampoline_unwindsafe);
        }

        Stash(
            options.as_ref(),
            (
                options,
                subpath,
                sepolicy_prefix,
                devino_to_csum_cache,
                sepolicy,
            ),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RepoCheckoutFilterResult;
    use gio::{File, NONE_CANCELLABLE};
    use glib_sys::{GFALSE, GTRUE};
    use std::ffi::{CStr, CString};
    use std::ptr;

    #[test]
    fn should_convert_default_options() {
        let options = RepoCheckoutAtOptions::default();
        let stash = options.to_glib_none();
        let ptr = stash.0;
        unsafe {
            assert_eq!((*ptr).mode, OSTREE_REPO_CHECKOUT_MODE_NONE);
            assert_eq!((*ptr).overwrite_mode, OSTREE_REPO_CHECKOUT_OVERWRITE_NONE);
            assert_eq!((*ptr).enable_uncompressed_cache, GFALSE);
            assert_eq!((*ptr).enable_fsync, GFALSE);
            assert_eq!((*ptr).process_whiteouts, GFALSE);
            assert_eq!((*ptr).no_copy_fallback, GFALSE);
            assert_eq!((*ptr).force_copy, GFALSE);
            assert_eq!((*ptr).bareuseronly_dirs, GFALSE);
            assert_eq!((*ptr).force_copy_zerosized, GFALSE);
            assert_eq!((*ptr).unused_bools, [GFALSE; 4]);
            assert_eq!((*ptr).subpath, ptr::null());
            assert_eq!((*ptr).devino_to_csum_cache, ptr::null_mut());
            assert_eq!((*ptr).unused_ints, [0; 6]);
            assert_eq!((*ptr).unused_ptrs, [ptr::null_mut(); 3]);
            assert_eq!((*ptr).filter, None);
            assert_eq!((*ptr).filter_user_data, ptr::null_mut());
            assert_eq!((*ptr).sepolicy, ptr::null_mut());
            assert_eq!((*ptr).sepolicy_prefix, ptr::null());
        }
    }

    #[test]
    fn should_convert_non_default_options() {
        let options = RepoCheckoutAtOptions {
            mode: RepoCheckoutMode::User,
            overwrite_mode: RepoCheckoutOverwriteMode::UnionIdentical,
            enable_uncompressed_cache: true,
            enable_fsync: true,
            process_whiteouts: true,
            no_copy_fallback: true,
            force_copy: true,
            bareuseronly_dirs: true,
            force_copy_zerosized: true,
            subpath: Some("sub/path".into()),
            devino_to_csum_cache: Some(RepoDevInoCache::new()),
            filter: RepoCheckoutFilter::new(|_repo, _path, _stat| RepoCheckoutFilterResult::Skip),
            sepolicy: Some(SePolicy::new(&File::new_for_path("a/b"), NONE_CANCELLABLE).unwrap()),
            sepolicy_prefix: Some("prefix".into()),
        };
        let stash = options.to_glib_none();
        let ptr = stash.0;
        unsafe {
            assert_eq!((*ptr).mode, OSTREE_REPO_CHECKOUT_MODE_USER);
            assert_eq!(
                (*ptr).overwrite_mode,
                OSTREE_REPO_CHECKOUT_OVERWRITE_UNION_IDENTICAL
            );
            assert_eq!((*ptr).enable_uncompressed_cache, GTRUE);
            assert_eq!((*ptr).enable_fsync, GTRUE);
            assert_eq!((*ptr).process_whiteouts, GTRUE);
            assert_eq!((*ptr).no_copy_fallback, GTRUE);
            assert_eq!((*ptr).force_copy, GTRUE);
            assert_eq!((*ptr).bareuseronly_dirs, GTRUE);
            assert_eq!((*ptr).force_copy_zerosized, GTRUE);
            assert_eq!((*ptr).unused_bools, [GFALSE; 4]);
            assert_eq!(
                CStr::from_ptr((*ptr).subpath),
                CString::new("sub/path").unwrap().as_c_str()
            );
            assert_eq!(
                (*ptr).devino_to_csum_cache,
                options.devino_to_csum_cache.to_glib_none().0
            );
            assert_eq!((*ptr).unused_ints, [0; 6]);
            assert_eq!((*ptr).unused_ptrs, [ptr::null_mut(); 3]);
            assert!((*ptr).filter == Some(repo_checkout_filter::filter_trampoline_unwindsafe));
            assert_eq!(
                (*ptr).filter_user_data,
                options.filter.as_ref().unwrap().to_glib_none().0,
            );
            assert_eq!((*ptr).sepolicy, options.sepolicy.to_glib_none().0);
            assert_eq!(
                CStr::from_ptr((*ptr).sepolicy_prefix),
                CString::new("prefix").unwrap().as_c_str()
            );
        }
    }
}