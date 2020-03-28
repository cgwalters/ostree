//! Tests that mostly use the API and access the booted sysroot read-only.

use anyhow::Result;
use gio::prelude::*;
use ostree::prelude::*;

use crate::test::*;

pub(crate) fn tests() -> impl IntoIterator<Item = Test> {
    let mut tests = crate::deftests!(test_sysroot_ro);
    if !std::path::Path::new("/run/ostree-booted").exists() {
        for t in &mut tests {
            t.is_ignored = true
        }
    };
    tests
}

fn test_sysroot_ro() -> Result<()> {
    let cancellable = Some(gio::Cancellable::new());
    let sysroot = ostree::Sysroot::new_default();
    sysroot.load(cancellable.as_ref())?;
    assert!(sysroot.is_booted());

    let booted = sysroot.get_booted_deployment().expect("booted deployment");
    assert!(!booted.is_staged());
    let repo = sysroot.repo().expect("repo");

    let csum = booted.get_csum().expect("booted csum");
    let csum = csum.as_str();

    let (root, rev) = repo.read_commit(csum, cancellable.as_ref())?;
    assert_eq!(rev, csum);
    let root = root.downcast::<ostree::RepoFile>().expect("downcast");
    root.ensure_resolved()?;

    Ok(())
}
