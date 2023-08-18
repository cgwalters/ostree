//! Tests for ostree-prepare-root.c that run in a container image

use std::process::Command;
use std::path::Path;

use anyhow::Result;
use xshell::cmd;

/// Re-execute the current process if the provided environment variable is not set.
pub(crate) fn reexec_with_guardenv(k: &str, prefix_args: &[&str]) -> Result<()> {
    use std::os::unix::process::CommandExt;
    if std::env::var_os(k).is_some() {
        return Ok(());
    }
    let self_exe = std::fs::read_link("/proc/self/exe")?;
    let mut prefix_args = prefix_args.iter();
    let mut cmd = if let Some(p) = prefix_args.next() {
        let mut c = Command::new(p);
        c.args(prefix_args);
        c.arg(self_exe);
        c
    } else {
        Command::new(self_exe)
    };
    cmd.env(k, "1");
    cmd.args(std::env::args_os().skip(1));
    Err(cmd.exec().into())
}

/// Command to enter a mount namespace
const UNSHARE_ARGS: &[&str] = &["unshare", "-m"];

/// Ensure we're in a user and mount namespace, running as root in that namespace
fn reexec_with_unshare() -> Result<()> {
    reexec_with_guardenv("_REEXEC_UNSHARED", UNSHARE_ARGS)
}

fn init_test_repo(sh: &xshell::Shell, repopath: &str, composefs: bool) -> Result<()> {
    cmd!(sh, "ostree --repo={repopath} init --mode=bare").run()?;
    if composefs {
        cmd!(sh, "ostree --repo={repopath} config set ex-integrity.composefs true").run()?;
    }
    cmd!(sh, "ostree --repo={repopath} config set sysroot.bootloader none").run()?;
    sh.create_dir("tmproot")?;
    let g = sh.push_dir("tmproot");
    sh.create_dir("usr/bin")?;
    sh.write_file("usr/bin/bash", "this is bash")?;
    let moddir = "usr/lib/modules/5.10.27";
    sh.create_dir(moddir)?;
    sh.write_file(format!("{moddir}/vmlinuz"), "this is a kernel")?;
    sh.write_file(format!("{moddir}/initramfs.img"), "this is an initramfs")?;
    sh.write_file("usr/lib/os-release", "ID=testos\n")?;
    drop(g);
    cmd!(sh, "ostree --repo={repopath} commit -b testos --tree=dir=tmproot").run()?;
    Ok(())
}

fn get_ostree_from_bootloader_file(buf: &str) -> Result<String> {
    for line in buf.lines() {
        if let Some((k, v)) = line.split_once(' ') {
            if k != "options" {
                continue
            }
            let kargs = ostree_ext::ostree::KernelArgs::from_string(v);
            return Ok(kargs.get_last_value("ostree").ok_or_else(|| anyhow::anyhow!("Missing ostree="))?.to_string());
        }
    }
    anyhow::bail!("Failed to find options")
}

fn createroot(sh: &xshell::Shell, tmpdir: &str, composefs: bool) -> Result<()> {
    sh.create_dir("empty")?;
    // Since we're running with high privilege, if we detect we're on an existing
    // ostree system, then let's make the real /sysroot inaccessible
    if Path::new("/sysroot").exists() {
        cmd!(sh, "mount --bind empty /sysroot").run()?;
    }
    // Make an overlayfs for / because prepare-root writes to it
    sh.create_dir("tmp-root-ovl/upper")?;
    sh.create_dir("tmp-root-ovl/work")?;
    sh.create_dir("tmp-root")?;
    cmd!(sh, "mount -t overlay -o lowerdir=/,upperdir={tmpdir}/tmp-root-ovl/upper,workdir={tmpdir}/tmp-root-ovl/work overlay tmp-root").run()?;
    cmd!(sh, "mount --bind tmp-root /").run()?;
    sh.change_dir(tmpdir);
    cmd!(sh, "ls").run()?;
    sh.create_dir("sysroot")?;
    let sysroot_ostree = "sysroot/ostree";
    cmd!(sh, "ostree admin init-fs --modern sysroot").run()?;
    cmd!(sh, "ostree admin os-init --sysroot sysroot testos").run()?;
    let repopath = &format!("{sysroot_ostree}/repo");
    init_test_repo(sh, repopath, composefs)?;
    let deployroot = &format!("{sysroot_ostree}/deploy/exampleos");
    sh.create_dir(Path::new(deployroot).parent().unwrap())?;
    cmd!(sh, "ostree admin deploy --sysroot=sysroot --os testos testos {tmpdir}/{deployroot}").run()?;

    let ostree_path = {
        let abs_boot_entries = &format!("{tmpdir}/sysroot/boot/loader/entries");
        let entry = std::fs::read_dir(abs_boot_entries)?.next().ok_or_else(|| anyhow::anyhow!("Failed to find bootloader entry"))??;
        let contents = std::fs::read_to_string(entry.path())?;
        get_ostree_from_bootloader_file(&contents)?
    };

    // Fake out /proc/cmdline
    sh.write_file("cmdline", format!("ostree={ostree_path}"))?;
    cmd!(sh, "mount --bind cmdline /proc/cmdline").run()?;

    // We want an empty /run
    sh.create_dir("run")?;
    cmd!(sh, "mount --bind run /run").run()?; // run run run!

    // Make a stub /tmp
    sh.create_dir("tmp")?;

    Ok(())
}

pub(crate) fn itest_basic() -> Result<()> {
    reexec_with_unshare()?;
    println!("Testing basic");
    let sh = &xshell::Shell::new()?;
    let tmpdir = tempfile::tempdir()?;
    let tmpdir = tmpdir.path();
    let tmpdir = tmpdir.to_str().unwrap();
    sh.change_dir(tmpdir);
    createroot(sh, tmpdir, false)?;

    // Now, run the remounts
    cmd!(sh, "env ROOT_TMPDIR={tmpdir}/tmp /usr/lib/ostree/ostree-prepare-root {tmpdir}/sysroot").run()?;

    // And verify the result
    cmd!(sh, "findmnt sysroot").run()?;
    Ok(())
}

pub(crate) fn itest_composefs() -> Result<()> {
    reexec_with_unshare()?;
    println!("Testing composefs");
    let sh = &xshell::Shell::new()?;
    let tmpdir = tempfile::tempdir()?;
    let tmpdir = tmpdir.path();
    let tmpdir = tmpdir.to_str().unwrap();
    sh.change_dir(tmpdir);
    createroot(sh, tmpdir, true)?;

    std::fs::write(format!("{tmpdir}/ostree-prepare-root.conf"), "\
[composefs] \
enabled=true
")?;

    // Now, run the remounts
    cmd!(sh, "env ROOT_TMPDIR={tmpdir}/tmp /usr/lib/ostree/ostree-prepare-root {tmpdir}/sysroot").run()?;

    // And verify the result
    cmd!(sh, "findmnt sysroot").run()?;
    Ok(())
}
