// This file was generated by gir (https://github.com/gtk-rs/gir @ ffda6f9)
// from gir-files (https://github.com/gtk-rs/gir-files @ ???)
// DO NOT EDIT

extern crate libostree_sys;
extern crate shell_words;
extern crate tempdir;
use std::env;
use std::error::Error;
use std::path::Path;
use std::mem::{align_of, size_of};
use std::process::Command;
use std::str;
use libostree_sys::*;

static PACKAGES: &[&str] = &["ostree-1"];

#[derive(Clone, Debug)]
struct Compiler {
    pub args: Vec<String>,
}

impl Compiler {
    pub fn new() -> Result<Compiler, Box<Error>> {
        let mut args = get_var("CC", "cc")?;
        args.push("-Wno-deprecated-declarations".to_owned());
        // For %z support in printf when using MinGW.
        args.push("-D__USE_MINGW_ANSI_STDIO".to_owned());
        args.extend(get_var("CFLAGS", "")?);
        args.extend(get_var("CPPFLAGS", "")?);
        args.extend(pkg_config_cflags(PACKAGES)?);
        Ok(Compiler { args })
    }

    pub fn define<'a, V: Into<Option<&'a str>>>(&mut self, var: &str, val: V) {
        let arg = match val.into() {
            None => format!("-D{}", var),
            Some(val) => format!("-D{}={}", var, val),
        };
        self.args.push(arg);
    }

    pub fn compile(&self, src: &Path, out: &Path) -> Result<(), Box<Error>> {
        let mut cmd = self.to_command();
        cmd.arg(src);
        cmd.arg("-o");
        cmd.arg(out);
        let status = cmd.spawn()?.wait()?;
        if !status.success() {
            return Err(format!("compilation command {:?} failed, {}",
                               &cmd, status).into());
        }
        Ok(())
    }

    fn to_command(&self) -> Command {
        let mut cmd = Command::new(&self.args[0]);
        cmd.args(&self.args[1..]);
        cmd
    }
}

fn get_var(name: &str, default: &str) -> Result<Vec<String>, Box<Error>> {
    match env::var(name) {
        Ok(value) => Ok(shell_words::split(&value)?),
        Err(env::VarError::NotPresent) => Ok(shell_words::split(default)?),
        Err(err) => Err(format!("{} {}", name, err).into()),
    }
}

fn pkg_config_cflags(packages: &[&str]) -> Result<Vec<String>, Box<Error>> {
    if packages.is_empty() {
        return Ok(Vec::new());
    }
    let mut cmd = Command::new("pkg-config");
    cmd.arg("--cflags");
    cmd.args(packages);
    let out = cmd.output()?;
    if !out.status.success() {
        return Err(format!("command {:?} returned {}",
                           &cmd, out.status).into());
    }
    let stdout = str::from_utf8(&out.stdout)?;
    Ok(shell_words::split(stdout.trim())?)
}


#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct Layout {
    size: usize,
    alignment: usize,
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
struct Results {
    /// Number of successfully completed tests.
    passed: usize,
    /// Total number of failed tests (including those that failed to compile).
    failed: usize,
    /// Number of tests that failed to compile.
    failed_to_compile: usize,
}

impl Results {
    fn record_passed(&mut self) {
        self.passed += 1;
    }
    fn record_failed(&mut self) {
        self.failed += 1;
    }
    fn record_failed_to_compile(&mut self) {
        self.failed += 1;
        self.failed_to_compile += 1;
    }
    fn summary(&self) -> String {
        format!(
            "{} passed; {} failed (compilation errors: {})",
            self.passed,
            self.failed,
            self.failed_to_compile)
    }
    fn expect_total_success(&self) {
        if self.failed == 0 {
            println!("OK: {}", self.summary());
        } else {
            panic!("FAILED: {}", self.summary());
        };
    }
}

#[test]
fn cross_validate_constants_with_c() {
    let tmpdir = tempdir::TempDir::new("abi").expect("temporary directory");
    let cc = Compiler::new().expect("configured compiler");

    assert_eq!("1",
               get_c_value(tmpdir.path(), &cc, "1").expect("C constant"),
               "failed to obtain correct constant value for 1");

    let mut results : Results = Default::default();
    for (i, &(name, rust_value)) in RUST_CONSTANTS.iter().enumerate() {
        match get_c_value(tmpdir.path(), &cc, name) {
            Err(e) => {
                results.record_failed_to_compile();
                eprintln!("{}", e);
            },
            Ok(ref c_value) => {
                if rust_value == c_value {
                    results.record_passed();
                } else {
                    results.record_failed();
                    eprintln!("Constant value mismatch for {}\nRust: {:?}\nC:    {:?}",
                              name, rust_value, c_value);
                }
            }
        };
        if (i + 1) % 25 == 0 {
            println!("constants ... {}", results.summary());
        }
    }
    results.expect_total_success();
}

#[test]
fn cross_validate_layout_with_c() {
    let tmpdir = tempdir::TempDir::new("abi").expect("temporary directory");
    let cc = Compiler::new().expect("configured compiler");

    assert_eq!(Layout {size: 1, alignment: 1},
               get_c_layout(tmpdir.path(), &cc, "char").expect("C layout"),
               "failed to obtain correct layout for char type");

    let mut results : Results = Default::default();
    for (i, &(name, rust_layout)) in RUST_LAYOUTS.iter().enumerate() {
        match get_c_layout(tmpdir.path(), &cc, name) {
            Err(e) => {
                results.record_failed_to_compile();
                eprintln!("{}", e);
            },
            Ok(c_layout) => {
                if rust_layout == c_layout {
                    results.record_passed();
                } else {
                    results.record_failed();
                    eprintln!("Layout mismatch for {}\nRust: {:?}\nC:    {:?}",
                              name, rust_layout, &c_layout);
                }
            }
        };
        if (i + 1) % 25 == 0 {
            println!("layout    ... {}", results.summary());
        }
    }
    results.expect_total_success();
}

fn get_c_layout(dir: &Path, cc: &Compiler, name: &str) -> Result<Layout, Box<Error>> {
    let exe = dir.join("layout");
    let mut cc = cc.clone();
    cc.define("ABI_TYPE_NAME", name);
    cc.compile(Path::new("tests/layout.c"), &exe)?;

    let mut abi_cmd = Command::new(exe);
    let output = abi_cmd.output()?;
    if !output.status.success() {
        return Err(format!("command {:?} failed, {:?}",
                           &abi_cmd, &output).into());
    }

    let stdout = str::from_utf8(&output.stdout)?;
    let mut words = stdout.trim().split_whitespace();
    let size = words.next().unwrap().parse().unwrap();
    let alignment = words.next().unwrap().parse().unwrap();
    Ok(Layout {size, alignment})
}

fn get_c_value(dir: &Path, cc: &Compiler, name: &str) -> Result<String, Box<Error>> {
    let exe = dir.join("constant");
    let mut cc = cc.clone();
    cc.define("ABI_CONSTANT_NAME", name);
    cc.compile(Path::new("tests/constant.c"), &exe)?;

    let mut abi_cmd = Command::new(exe);
    let output = abi_cmd.output()?;
    if !output.status.success() {
        return Err(format!("command {:?} failed, {:?}",
                           &abi_cmd, &output).into());
    }

    Ok(str::from_utf8(&output.stdout)?.trim().to_owned())
}

const RUST_LAYOUTS: &[(&str, Layout)] = &[
    ("OstreeAsyncProgressClass", Layout {size: size_of::<OstreeAsyncProgressClass>(), alignment: align_of::<OstreeAsyncProgressClass>()}),
    ("OstreeChecksumFlags", Layout {size: size_of::<OstreeChecksumFlags>(), alignment: align_of::<OstreeChecksumFlags>()}),
    ("OstreeCollectionRef", Layout {size: size_of::<OstreeCollectionRef>(), alignment: align_of::<OstreeCollectionRef>()}),
    ("OstreeCollectionRefv", Layout {size: size_of::<OstreeCollectionRefv>(), alignment: align_of::<OstreeCollectionRefv>()}),
    ("OstreeDeploymentUnlockedState", Layout {size: size_of::<OstreeDeploymentUnlockedState>(), alignment: align_of::<OstreeDeploymentUnlockedState>()}),
    ("OstreeDiffDirsOptions", Layout {size: size_of::<OstreeDiffDirsOptions>(), alignment: align_of::<OstreeDiffDirsOptions>()}),
    ("OstreeDiffFlags", Layout {size: size_of::<OstreeDiffFlags>(), alignment: align_of::<OstreeDiffFlags>()}),
    ("OstreeDiffItem", Layout {size: size_of::<OstreeDiffItem>(), alignment: align_of::<OstreeDiffItem>()}),
    ("OstreeGpgError", Layout {size: size_of::<OstreeGpgError>(), alignment: align_of::<OstreeGpgError>()}),
    ("OstreeGpgSignatureAttr", Layout {size: size_of::<OstreeGpgSignatureAttr>(), alignment: align_of::<OstreeGpgSignatureAttr>()}),
    ("OstreeGpgSignatureFormatFlags", Layout {size: size_of::<OstreeGpgSignatureFormatFlags>(), alignment: align_of::<OstreeGpgSignatureFormatFlags>()}),
    ("OstreeMutableTreeClass", Layout {size: size_of::<OstreeMutableTreeClass>(), alignment: align_of::<OstreeMutableTreeClass>()}),
    ("OstreeMutableTreeIter", Layout {size: size_of::<OstreeMutableTreeIter>(), alignment: align_of::<OstreeMutableTreeIter>()}),
    ("OstreeObjectType", Layout {size: size_of::<OstreeObjectType>(), alignment: align_of::<OstreeObjectType>()}),
    ("OstreeRepoCheckoutAtOptions", Layout {size: size_of::<OstreeRepoCheckoutAtOptions>(), alignment: align_of::<OstreeRepoCheckoutAtOptions>()}),
    ("OstreeRepoCheckoutFilterResult", Layout {size: size_of::<OstreeRepoCheckoutFilterResult>(), alignment: align_of::<OstreeRepoCheckoutFilterResult>()}),
    ("OstreeRepoCheckoutMode", Layout {size: size_of::<OstreeRepoCheckoutMode>(), alignment: align_of::<OstreeRepoCheckoutMode>()}),
    ("OstreeRepoCheckoutOverwriteMode", Layout {size: size_of::<OstreeRepoCheckoutOverwriteMode>(), alignment: align_of::<OstreeRepoCheckoutOverwriteMode>()}),
    ("OstreeRepoCommitFilterResult", Layout {size: size_of::<OstreeRepoCommitFilterResult>(), alignment: align_of::<OstreeRepoCommitFilterResult>()}),
    ("OstreeRepoCommitIterResult", Layout {size: size_of::<OstreeRepoCommitIterResult>(), alignment: align_of::<OstreeRepoCommitIterResult>()}),
    ("OstreeRepoCommitModifierFlags", Layout {size: size_of::<OstreeRepoCommitModifierFlags>(), alignment: align_of::<OstreeRepoCommitModifierFlags>()}),
    ("OstreeRepoCommitState", Layout {size: size_of::<OstreeRepoCommitState>(), alignment: align_of::<OstreeRepoCommitState>()}),
    ("OstreeRepoCommitTraverseFlags", Layout {size: size_of::<OstreeRepoCommitTraverseFlags>(), alignment: align_of::<OstreeRepoCommitTraverseFlags>()}),
    ("OstreeRepoCommitTraverseIter", Layout {size: size_of::<OstreeRepoCommitTraverseIter>(), alignment: align_of::<OstreeRepoCommitTraverseIter>()}),
    ("OstreeRepoFileClass", Layout {size: size_of::<OstreeRepoFileClass>(), alignment: align_of::<OstreeRepoFileClass>()}),
    ("OstreeRepoFinderAvahiClass", Layout {size: size_of::<OstreeRepoFinderAvahiClass>(), alignment: align_of::<OstreeRepoFinderAvahiClass>()}),
    ("OstreeRepoFinderConfigClass", Layout {size: size_of::<OstreeRepoFinderConfigClass>(), alignment: align_of::<OstreeRepoFinderConfigClass>()}),
    ("OstreeRepoFinderInterface", Layout {size: size_of::<OstreeRepoFinderInterface>(), alignment: align_of::<OstreeRepoFinderInterface>()}),
    ("OstreeRepoFinderMountClass", Layout {size: size_of::<OstreeRepoFinderMountClass>(), alignment: align_of::<OstreeRepoFinderMountClass>()}),
    ("OstreeRepoFinderOverrideClass", Layout {size: size_of::<OstreeRepoFinderOverrideClass>(), alignment: align_of::<OstreeRepoFinderOverrideClass>()}),
    ("OstreeRepoFinderResult", Layout {size: size_of::<OstreeRepoFinderResult>(), alignment: align_of::<OstreeRepoFinderResult>()}),
    ("OstreeRepoFinderResultv", Layout {size: size_of::<OstreeRepoFinderResultv>(), alignment: align_of::<OstreeRepoFinderResultv>()}),
    ("OstreeRepoListObjectsFlags", Layout {size: size_of::<OstreeRepoListObjectsFlags>(), alignment: align_of::<OstreeRepoListObjectsFlags>()}),
    ("OstreeRepoListRefsExtFlags", Layout {size: size_of::<OstreeRepoListRefsExtFlags>(), alignment: align_of::<OstreeRepoListRefsExtFlags>()}),
    ("OstreeRepoMode", Layout {size: size_of::<OstreeRepoMode>(), alignment: align_of::<OstreeRepoMode>()}),
    ("OstreeRepoPruneFlags", Layout {size: size_of::<OstreeRepoPruneFlags>(), alignment: align_of::<OstreeRepoPruneFlags>()}),
    ("OstreeRepoPruneOptions", Layout {size: size_of::<OstreeRepoPruneOptions>(), alignment: align_of::<OstreeRepoPruneOptions>()}),
    ("OstreeRepoPullFlags", Layout {size: size_of::<OstreeRepoPullFlags>(), alignment: align_of::<OstreeRepoPullFlags>()}),
    ("OstreeRepoRemoteChange", Layout {size: size_of::<OstreeRepoRemoteChange>(), alignment: align_of::<OstreeRepoRemoteChange>()}),
    ("OstreeRepoResolveRevExtFlags", Layout {size: size_of::<OstreeRepoResolveRevExtFlags>(), alignment: align_of::<OstreeRepoResolveRevExtFlags>()}),
    ("OstreeRepoTransactionStats", Layout {size: size_of::<OstreeRepoTransactionStats>(), alignment: align_of::<OstreeRepoTransactionStats>()}),
    ("OstreeSePolicyRestoreconFlags", Layout {size: size_of::<OstreeSePolicyRestoreconFlags>(), alignment: align_of::<OstreeSePolicyRestoreconFlags>()}),
    ("OstreeStaticDeltaGenerateOpt", Layout {size: size_of::<OstreeStaticDeltaGenerateOpt>(), alignment: align_of::<OstreeStaticDeltaGenerateOpt>()}),
    ("OstreeSysrootSimpleWriteDeploymentFlags", Layout {size: size_of::<OstreeSysrootSimpleWriteDeploymentFlags>(), alignment: align_of::<OstreeSysrootSimpleWriteDeploymentFlags>()}),
    ("OstreeSysrootUpgraderFlags", Layout {size: size_of::<OstreeSysrootUpgraderFlags>(), alignment: align_of::<OstreeSysrootUpgraderFlags>()}),
    ("OstreeSysrootUpgraderPullFlags", Layout {size: size_of::<OstreeSysrootUpgraderPullFlags>(), alignment: align_of::<OstreeSysrootUpgraderPullFlags>()}),
    ("OstreeSysrootWriteDeploymentsOpts", Layout {size: size_of::<OstreeSysrootWriteDeploymentsOpts>(), alignment: align_of::<OstreeSysrootWriteDeploymentsOpts>()}),
];

const RUST_CONSTANTS: &[(&str, &str)] = &[
    ("OSTREE_CHECKSUM_FLAGS_IGNORE_XATTRS", "1"),
    ("OSTREE_CHECKSUM_FLAGS_NONE", "0"),
    ("OSTREE_COMMIT_GVARIANT_STRING", "(a{sv}aya(say)sstayay)"),
    ("OSTREE_COMMIT_META_KEY_COLLECTION_BINDING", "ostree.collection-binding"),
    ("OSTREE_COMMIT_META_KEY_ENDOFLIFE", "ostree.endoflife"),
    ("OSTREE_COMMIT_META_KEY_ENDOFLIFE_REBASE", "ostree.endoflife-rebase"),
    ("OSTREE_COMMIT_META_KEY_REF_BINDING", "ostree.ref-binding"),
    ("OSTREE_COMMIT_META_KEY_SOURCE_TITLE", "ostree.source-title"),
    ("OSTREE_COMMIT_META_KEY_VERSION", "version"),
    ("OSTREE_DEPLOYMENT_UNLOCKED_DEVELOPMENT", "1"),
    ("OSTREE_DEPLOYMENT_UNLOCKED_HOTFIX", "2"),
    ("OSTREE_DEPLOYMENT_UNLOCKED_NONE", "0"),
    ("OSTREE_DIFF_FLAGS_IGNORE_XATTRS", "1"),
    ("OSTREE_DIFF_FLAGS_NONE", "0"),
    ("OSTREE_DIRMETA_GVARIANT_STRING", "(uuua(ayay))"),
    ("OSTREE_FILEMETA_GVARIANT_STRING", "(uuua(ayay))"),
    ("OSTREE_GPG_ERROR_INVALID_SIGNATURE", "1"),
    ("OSTREE_GPG_ERROR_MISSING_KEY", "2"),
    ("OSTREE_GPG_ERROR_NO_SIGNATURE", "0"),
    ("OSTREE_GPG_SIGNATURE_ATTR_EXP_TIMESTAMP", "7"),
    ("OSTREE_GPG_SIGNATURE_ATTR_FINGERPRINT", "5"),
    ("OSTREE_GPG_SIGNATURE_ATTR_FINGERPRINT_PRIMARY", "12"),
    ("OSTREE_GPG_SIGNATURE_ATTR_HASH_ALGO_NAME", "9"),
    ("OSTREE_GPG_SIGNATURE_ATTR_KEY_EXPIRED", "2"),
    ("OSTREE_GPG_SIGNATURE_ATTR_KEY_MISSING", "4"),
    ("OSTREE_GPG_SIGNATURE_ATTR_KEY_REVOKED", "3"),
    ("OSTREE_GPG_SIGNATURE_ATTR_PUBKEY_ALGO_NAME", "8"),
    ("OSTREE_GPG_SIGNATURE_ATTR_SIG_EXPIRED", "1"),
    ("OSTREE_GPG_SIGNATURE_ATTR_TIMESTAMP", "6"),
    ("OSTREE_GPG_SIGNATURE_ATTR_USER_EMAIL", "11"),
    ("OSTREE_GPG_SIGNATURE_ATTR_USER_NAME", "10"),
    ("OSTREE_GPG_SIGNATURE_ATTR_VALID", "0"),
    ("OSTREE_GPG_SIGNATURE_FORMAT_DEFAULT", "0"),
    ("OSTREE_MAX_METADATA_SIZE", "10485760"),
    ("OSTREE_MAX_METADATA_WARN_SIZE", "7340032"),
    ("OSTREE_OBJECT_TYPE_COMMIT", "4"),
    ("OSTREE_OBJECT_TYPE_COMMIT_META", "6"),
    ("OSTREE_OBJECT_TYPE_DIR_META", "3"),
    ("OSTREE_OBJECT_TYPE_DIR_TREE", "2"),
    ("OSTREE_OBJECT_TYPE_FILE", "1"),
    ("OSTREE_OBJECT_TYPE_PAYLOAD_LINK", "7"),
    ("OSTREE_OBJECT_TYPE_TOMBSTONE_COMMIT", "5"),
    ("OSTREE_ORIGIN_TRANSIENT_GROUP", "libostree-transient"),
    ("OSTREE_RELEASE_VERSION", "8"),
    ("OSTREE_REPO_CHECKOUT_FILTER_ALLOW", "0"),
    ("OSTREE_REPO_CHECKOUT_FILTER_SKIP", "1"),
    ("OSTREE_REPO_CHECKOUT_MODE_NONE", "0"),
    ("OSTREE_REPO_CHECKOUT_MODE_USER", "1"),
    ("OSTREE_REPO_CHECKOUT_OVERWRITE_ADD_FILES", "2"),
    ("OSTREE_REPO_CHECKOUT_OVERWRITE_NONE", "0"),
    ("OSTREE_REPO_CHECKOUT_OVERWRITE_UNION_FILES", "1"),
    ("OSTREE_REPO_CHECKOUT_OVERWRITE_UNION_IDENTICAL", "3"),
    ("OSTREE_REPO_COMMIT_FILTER_ALLOW", "0"),
    ("OSTREE_REPO_COMMIT_FILTER_SKIP", "1"),
    ("OSTREE_REPO_COMMIT_ITER_RESULT_DIR", "3"),
    ("OSTREE_REPO_COMMIT_ITER_RESULT_END", "1"),
    ("OSTREE_REPO_COMMIT_ITER_RESULT_ERROR", "0"),
    ("OSTREE_REPO_COMMIT_ITER_RESULT_FILE", "2"),
    ("OSTREE_REPO_COMMIT_MODIFIER_FLAGS_CANONICAL_PERMISSIONS", "4"),
    ("OSTREE_REPO_COMMIT_MODIFIER_FLAGS_CONSUME", "16"),
    ("OSTREE_REPO_COMMIT_MODIFIER_FLAGS_DEVINO_CANONICAL", "32"),
    ("OSTREE_REPO_COMMIT_MODIFIER_FLAGS_ERROR_ON_UNLABELED", "8"),
    ("OSTREE_REPO_COMMIT_MODIFIER_FLAGS_GENERATE_SIZES", "2"),
    ("OSTREE_REPO_COMMIT_MODIFIER_FLAGS_NONE", "0"),
    ("OSTREE_REPO_COMMIT_MODIFIER_FLAGS_SKIP_XATTRS", "1"),
    ("OSTREE_REPO_COMMIT_STATE_NORMAL", "0"),
    ("OSTREE_REPO_COMMIT_STATE_PARTIAL", "1"),
    ("OSTREE_REPO_COMMIT_TRAVERSE_FLAG_NONE", "1"),
    ("OSTREE_REPO_LIST_OBJECTS_ALL", "4"),
    ("OSTREE_REPO_LIST_OBJECTS_LOOSE", "1"),
    ("OSTREE_REPO_LIST_OBJECTS_NO_PARENTS", "8"),
    ("OSTREE_REPO_LIST_OBJECTS_PACKED", "2"),
    ("OSTREE_REPO_LIST_REFS_EXT_ALIASES", "1"),
    ("OSTREE_REPO_LIST_REFS_EXT_EXCLUDE_REMOTES", "2"),
    ("OSTREE_REPO_LIST_REFS_EXT_NONE", "0"),
    ("OSTREE_REPO_METADATA_REF", "ostree-metadata"),
    ("OSTREE_REPO_MODE_ARCHIVE", "1"),
    ("OSTREE_REPO_MODE_ARCHIVE_Z2", "1"),
    ("OSTREE_REPO_MODE_BARE", "0"),
    ("OSTREE_REPO_MODE_BARE_USER", "2"),
    ("OSTREE_REPO_MODE_BARE_USER_ONLY", "3"),
    ("OSTREE_REPO_PRUNE_FLAGS_NONE", "0"),
    ("OSTREE_REPO_PRUNE_FLAGS_NO_PRUNE", "1"),
    ("OSTREE_REPO_PRUNE_FLAGS_REFS_ONLY", "2"),
    ("OSTREE_REPO_PULL_FLAGS_BAREUSERONLY_FILES", "8"),
    ("OSTREE_REPO_PULL_FLAGS_COMMIT_ONLY", "2"),
    ("OSTREE_REPO_PULL_FLAGS_MIRROR", "1"),
    ("OSTREE_REPO_PULL_FLAGS_NONE", "0"),
    ("OSTREE_REPO_PULL_FLAGS_TRUSTED_HTTP", "16"),
    ("OSTREE_REPO_PULL_FLAGS_UNTRUSTED", "4"),
    ("OSTREE_REPO_REMOTE_CHANGE_ADD", "0"),
    ("OSTREE_REPO_REMOTE_CHANGE_ADD_IF_NOT_EXISTS", "1"),
    ("OSTREE_REPO_REMOTE_CHANGE_DELETE", "2"),
    ("OSTREE_REPO_REMOTE_CHANGE_DELETE_IF_EXISTS", "3"),
    ("OSTREE_REPO_RESOLVE_REV_EXT_NONE", "0"),
    ("OSTREE_SEPOLICY_RESTORECON_FLAGS_ALLOW_NOLABEL", "1"),
    ("OSTREE_SEPOLICY_RESTORECON_FLAGS_KEEP_EXISTING", "2"),
    ("OSTREE_SEPOLICY_RESTORECON_FLAGS_NONE", "0"),
    ("OSTREE_SHA256_DIGEST_LEN", "32"),
    ("OSTREE_SHA256_STRING_LEN", "64"),
    ("OSTREE_STATIC_DELTA_GENERATE_OPT_LOWLATENCY", "0"),
    ("OSTREE_STATIC_DELTA_GENERATE_OPT_MAJOR", "1"),
    ("OSTREE_SUMMARY_GVARIANT_STRING", "(a(s(taya{sv}))a{sv})"),
    ("OSTREE_SUMMARY_SIG_GVARIANT_STRING", "a{sv}"),
    ("OSTREE_SYSROOT_SIMPLE_WRITE_DEPLOYMENT_FLAGS_NONE", "0"),
    ("OSTREE_SYSROOT_SIMPLE_WRITE_DEPLOYMENT_FLAGS_NOT_DEFAULT", "2"),
    ("OSTREE_SYSROOT_SIMPLE_WRITE_DEPLOYMENT_FLAGS_NO_CLEAN", "4"),
    ("OSTREE_SYSROOT_SIMPLE_WRITE_DEPLOYMENT_FLAGS_RETAIN", "1"),
    ("OSTREE_SYSROOT_SIMPLE_WRITE_DEPLOYMENT_FLAGS_RETAIN_PENDING", "8"),
    ("OSTREE_SYSROOT_SIMPLE_WRITE_DEPLOYMENT_FLAGS_RETAIN_ROLLBACK", "16"),
    ("OSTREE_SYSROOT_UPGRADER_FLAGS_IGNORE_UNCONFIGURED", "2"),
    ("OSTREE_SYSROOT_UPGRADER_PULL_FLAGS_ALLOW_OLDER", "1"),
    ("OSTREE_SYSROOT_UPGRADER_PULL_FLAGS_NONE", "0"),
    ("OSTREE_SYSROOT_UPGRADER_PULL_FLAGS_SYNTHETIC", "2"),
    ("OSTREE_TIMESTAMP", "0"),
    ("OSTREE_TREE_GVARIANT_STRING", "(a(say)a(sayay))"),
    ("OSTREE_VERSION", "2018.800000"),
    ("OSTREE_VERSION_S", "2018.8"),
    ("OSTREE_YEAR_VERSION", "2018"),
];

