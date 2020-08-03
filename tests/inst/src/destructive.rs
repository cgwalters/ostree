//! Test that interrupting an upgrade is safe.
//!
//! This test builds on coreos-assembler's "external tests":
//! https://github.com/coreos/coreos-assembler/blob/master/mantle/kola/README-kola-ext.md
//! Key to this in particular is coreos-assembler implementing the Debian autopkgtest reboot API.
//!
//! The basic model of this test is:
//!
//! Copy the OS content in to an archive repository, and generate a "synthetic"
//! update for it by randomly mutating ELF files.  Time how long upgrading
//! to that takes, to use as a baseline in a range of time we will target
//! for interrupt.
//!
//! Start a webserver, pointing rpm-ostree at the updated content.  We
//! alternate between a few "interrupt strategies", from `kill -9` on
//! rpm-ostreed, or rebooting normally, or an immediate forced reboot
//! (with no filesystem sync).
//!
//! The state of the tests is passed by serializing JSON into the
//! AUTOPKGTEST_REBOOT_MARK.

use anyhow::{Context, Result};
use commandspec::sh_execute;
use rand::seq::SliceRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::Write;
use std::path::Path;
use std::time;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::rpmostree;
use crate::test::*;

const ORIGREF: &'static str = "orig-booted";
const TESTREF: &'static str = "testcontent";
const TDATAPATH: &'static str = "/var/tmp/ostree-test-transaction-data.json";
const SRVREPO: &'static str = "/var/tmp/ostree-test-srv";
// Percentage of ELF files to change per update
const TREEGEN_PERCENTAGE: u32 = 15;
/// Total number of reboots
const ITERATIONS: u32 = 10;
/// Try at most this number of times per iteration to interrupt
const ITERATION_RETRIES: u32 = 15;
// We mostly want to test forced interrupts since those are
// most likely to break.
const FORCE_INTERRUPT_PERCENTAGE: u32 = 85;
/// Multiply the average cycle time by this to ensure we sometimes
/// fail to interrupt too.
const FORCE_REBOOT_AFTER_MUL: f64 = 1.2f64;
/// Amount of time in seconds we will delay each web request.
/// FIXME: this should be a function of total number of objects or so
const WEBSERVER_DELAY_SECS: f64 = 0.005;

/// We choose between these at random
#[derive(EnumIter, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum PoliteInterruptStrategy {
    None,
    Stop,
    Reboot,
}

/// We choose between these at random
#[derive(EnumIter, Debug, PartialEq, Eq, Clone, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum ForceInterruptStrategy {
    Kill9,
    Reboot,
}

#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum InterruptStrategy {
    Polite(PoliteInterruptStrategy),
    Force(ForceInterruptStrategy),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum UpdateResult {
    NotCompleted,
    Staged,
    Completed,
}

/// The data passed across reboots by serializing
/// into the AUTOPKGTEST_REBOOT_MARK
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "kebab-case")]
struct RebootMark {
    // forced will be true if the reboot was `-ff`
    reboot_strategy: Option<InterruptStrategy>,
    iter: u32,
    before: u32,
    polite: BTreeMap<PoliteInterruptStrategy, BTreeMap<UpdateResult, u32>>,
    force: BTreeMap<ForceInterruptStrategy, BTreeMap<UpdateResult, u32>>,
}

impl RebootMark {
    fn get_results_map(
        &mut self,
        strategy: &InterruptStrategy,
    ) -> &mut BTreeMap<UpdateResult, u32> {
        match strategy {
            InterruptStrategy::Polite(t) => self
                .polite
                .entry(t.clone())
                .or_insert_with(|| BTreeMap::new()),
            InterruptStrategy::Force(t) => self
                .force
                .entry(t.clone())
                .or_insert_with(|| BTreeMap::new()),
        }
    }
}

/// TODO add readonly sysroot handling into base ostree
fn testinit() -> Result<()> {
    assert!(std::path::Path::new("/run/ostree-booted").exists());
    sh_execute!(
        r"if ! test -w /sysroot; then
   mount -o remount,rw /sysroot
fi"
    )?;
    Ok(())
}

// Given a booted ostree, generate a modified version and write it
// into our srvrepo.  This is fairly hacky; it'd be better if we
// reworked the tree mutation to operate on an ostree repo
// rather than a filesystem.
fn generate_update(commit: &str) -> Result<()> {
    println!("Generating update from {}", commit);
    crate::treegen::update_os_tree(SRVREPO, TESTREF, TREEGEN_PERCENTAGE)
        .context("Failed to generate new content")?;
    // Amortize the prune across multiple runs; we don't want to leak space,
    // but traversing all the objects is expensive.  So here we only prune 1/5 of the time.
    if rand::thread_rng().gen_ratio(1, 5) {
        sh_execute!(
            "ostree --repo={srvrepo} prune --refs-only --depth=1",
            srvrepo = SRVREPO
        )?;
    }
    Ok(())
}

/// Create an archive repository of current OS content.  This is a bit expensive;
/// in the future we should try a trick using the `parent` property on this repo,
/// and then teach our webserver to redirect to the system for objects it doesn't
/// have.
fn generate_srv_repo(commit: &str) -> Result<()> {
    sh_execute!(
        r#"
        ostree --repo={srvrepo} init --mode=archive
        ostree --repo={srvrepo} config set archive.zlib-level 1
        ostree --repo={srvrepo} pull-local /sysroot/ostree/repo {commit}
        ostree --repo={srvrepo} refs --create={testref} {commit}
        "#,
        srvrepo = SRVREPO,
        commit = commit,
        testref = TESTREF
    )
    .context("Failed to generate srv repo")?;
    generate_update(commit)?;
    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct TransactionalTestInfo {
    cycle_time: time::Duration,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct Kill9Stats {
    interrupted: u32,
    staged: u32,
    success: u32,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct RebootStats {
    interrupted: u32,
    success: u32,
}

async fn run_upgrade_or_timeout(timeout: time::Duration) -> Result<bool> {
    let upgrade = tokio::task::spawn_blocking(move || -> Result<()> {
        sh_execute!(
            "rpm-ostree upgrade
            systemctl start ostree-finalize-staged"
        )
        .context("Upgrade failed")?;
        Ok(())
    });
    Ok(tokio::select! {
        res = upgrade => {
            let _res = res?;
            true
        },
        _ = tokio::time::delay_for(timeout) => {
            false
        }
    })
}

/// The set of commits that we should see
#[derive(Debug)]
struct CommitStates {
    booted: String,
    orig: String,
    prev: String,
    target: String,
}

impl CommitStates {
    pub(crate) fn describe(&self, commit: &str) -> Option<&'static str> {
        if commit == self.booted {
            Some("booted")
        } else if commit == self.orig {
            Some("orig")
        } else if commit == self.prev {
            Some("prev")
        } else if commit == self.target {
            Some("target")
        } else {
            None
        }
    }
}

/// In the case where we've entered via a reboot, this function
/// checks the state of things, and also generates a new update
/// if everything was successful.
fn parse_and_validate_reboot_mark<M: AsRef<str>>(
    commitstates: &mut CommitStates,
    mark: M,
) -> Result<RebootMark> {
    let markstr = mark.as_ref();
    let mut mark: RebootMark = serde_json::from_str(markstr)
        .with_context(|| format!("Failed to parse reboot mark {:?}", markstr))?;
    // The first failed reboot may be into the original booted commit
    let status = rpmostree::query_status()?;
    let firstdeploy = &status.deployments[0];
    // The first deployment should not be staged
    assert!(!firstdeploy.staged.unwrap_or(false));
    assert!(firstdeploy.booted);
    assert_eq!(firstdeploy.checksum, commitstates.booted);
    let reboot_type = if let Some(t) = mark.reboot_strategy.as_ref() {
        t.clone()
    } else {
        anyhow::bail!("No reboot strategy in mark");
    };
    if commitstates.booted == commitstates.target {
        mark.get_results_map(&reboot_type)
            .entry(UpdateResult::Completed)
            .and_modify(|result_e| {
                *result_e += 1;
            })
            .or_insert(1);
        println!("Successfully updated to {}", commitstates.target);
        // Since we successfully updated, generate a new commit to target
        generate_update(&firstdeploy.checksum)?;
        // Update the target state
        let srvrepo_obj = ostree::Repo::new(&gio::File::new_for_path(SRVREPO));
        srvrepo_obj.open(gio::NONE_CANCELLABLE)?;
        commitstates.target = srvrepo_obj.resolve_rev(TESTREF, false)?.into();
    } else if commitstates.booted == commitstates.orig || commitstates.booted == commitstates.prev {
        println!(
            "Failed update to {} (booted={})",
            commitstates.target, commitstates.booted
        );
        mark.get_results_map(&reboot_type)
            .entry(UpdateResult::NotCompleted)
            .and_modify(|result_e| {
                *result_e += 1;
            })
            .or_insert(1);
    } else {
        anyhow::bail!("Unexpected target commit: {}", firstdeploy.checksum);
    };
    // Empty this out
    mark.reboot_strategy = None;
    Ok(mark)
}

fn validate_pending_commit(pending_commit: &str, commitstates: &CommitStates) -> Result<()> {
    if pending_commit != commitstates.target {
        sh_execute!("rpm-ostree status -v")?;
        sh_execute!(
            "ostree show {pending_commit}",
            pending_commit = pending_commit
        )?;
        anyhow::bail!(
            "Expected target commit={} but pending={} ({:?})",
            commitstates.target,
            pending_commit,
            commitstates.describe(pending_commit)
        );
    }
    Ok(())
}

/// In the case where we did a kill -9 of rpm-ostree, check the state
fn validate_live_interrupted_upgrade(commitstates: &CommitStates) -> Result<UpdateResult> {
    let status = rpmostree::query_status()?;
    let firstdeploy = &status.deployments[0];
    let pending_commit = firstdeploy.checksum.as_str();
    let res = if firstdeploy.staged.unwrap_or(false) {
        assert!(!firstdeploy.booted);
        validate_pending_commit(pending_commit, &commitstates)?;
        UpdateResult::Staged
    } else {
        if pending_commit == commitstates.booted {
            UpdateResult::NotCompleted
        } else if pending_commit == commitstates.target {
            UpdateResult::Completed
        } else {
            anyhow::bail!(
                "Unexpected pending commit: {} ({:?})",
                pending_commit,
                commitstates.describe(pending_commit)
            );
        }
    };
    Ok(res)
}

fn impl_transaction_test<M: AsRef<str>>(
    booted_commit: &str,
    tdata: &TransactionalTestInfo,
    mark: Option<M>,
) -> Result<()> {
    let cancellable = Some(gio::Cancellable::new());
    let polite_strategies = PoliteInterruptStrategy::iter().collect::<Vec<_>>();
    let force_strategies = ForceInterruptStrategy::iter().collect::<Vec<_>>();

    // Gather the expected possible commits
    let mut commitstates = {
        let srvrepo_obj = ostree::Repo::new(&gio::File::new_for_path(SRVREPO));
        srvrepo_obj.open(cancellable.as_ref())?;
        let sysrepo_obj = ostree::Repo::new(&gio::File::new_for_path("/sysroot/ostree/repo"));
        sysrepo_obj.open(gio::NONE_CANCELLABLE)?;

        CommitStates {
            booted: booted_commit.to_string(),
            orig: sysrepo_obj.resolve_rev(ORIGREF, false)?.into(),
            prev: srvrepo_obj
                .resolve_rev(&format!("{}^", TESTREF), false)?
                .into(),
            target: srvrepo_obj.resolve_rev(TESTREF, false)?.into(),
        }
    };

    let mut mark = if let Some(mark) = mark {
        let markstr = mark.as_ref();
        // In the successful case, this generates a new target commit,
        // so we pass via &mut.
        parse_and_validate_reboot_mark(&mut commitstates, markstr)
            .context("Failed to parse reboot mark")?
    } else {
        RebootMark {
            ..Default::default()
        }
    };
    // Drop the &mut
    let commitstates = commitstates;

    let mut rt = tokio::runtime::Runtime::new()?;
    let cycle_time_ms = (tdata.cycle_time.as_secs_f64() * 1000f64 * FORCE_REBOOT_AFTER_MUL) as u64;
    let mut last_strategy: Option<InterruptStrategy> = None;
    let mut retries = 0;
    // This loop is for the non-rebooting strategies - we might use kill -9
    // or not interrupt at all.  But if we choose a reboot strategy
    // then we'll exit implicitly via the reboot, and reenter the function
    // above.
    loop {
        if let Some(last_strategy) = last_strategy {
            mark.iter += 1;
            retries = 0;
            let res = validate_live_interrupted_upgrade(&commitstates)?;
            mark.get_results_map(&last_strategy)
                .entry(res)
                .and_modify(|result_e| {
                    *result_e += 1;
                })
                .or_insert(1);
        }
        if mark.iter == ITERATIONS {
            // TODO also add ostree admin fsck to check the deployment directories
            sh_execute!(
                "echo Performing final validation...
                ostree fsck"
            )?;
            return Ok(());
        }
        let mut rng = rand::thread_rng();
        let strategy: InterruptStrategy = if rand::thread_rng()
            .gen_ratio(FORCE_INTERRUPT_PERCENTAGE, 100)
        {
            InterruptStrategy::Force(force_strategies.choose(&mut rng).expect("strategy").clone())
        } else {
            InterruptStrategy::Polite(
                polite_strategies
                    .choose(&mut rng)
                    .expect("strategy")
                    .clone(),
            )
        };
        println!("Using interrupt strategy: {:?}", strategy);
        // Initiate a forced reboot usually the upgrade would
        // complete, but also ~30% of the time after.
        let sleeptime = time::Duration::from_millis(rng.gen_range(0, cycle_time_ms));
        println!(
            "force-reboot-time={:?} cycle={:?} status:{:?}",
            sleeptime, tdata.cycle_time, &mark
        );
        sh_execute!(
            "
            rpm-ostree status -v
            systemctl stop rpm-ostreed
            ostree reset testrepo:{testref} {booted_commit}
            rpm-ostree cleanup -pbrm
            ",
            testref = TESTREF,
            booted_commit = booted_commit
        )
        .context("Failed pre-upgrade cleanup")?;
        let res: Result<bool> = rt.block_on(async move { run_upgrade_or_timeout(sleeptime).await });
        let res = res.context("Failed during upgrade")?;
        if res {
            println!(
                "Failed to interrupt upgrade, attempt {}/{}",
                retries, ITERATION_RETRIES
            );
            mark.before += 1;
            let status = rpmostree::query_status()?;
            let firstdeploy = &status.deployments[0];
            let pending_commit = firstdeploy.checksum.as_str();
            validate_pending_commit(pending_commit, &commitstates)
                .context("Failed to validate pending commit")?;
        } else {
            match strategy {
                InterruptStrategy::Force(ForceInterruptStrategy::Kill9) => {
                    sh_execute!(
                        "systemctl kill -s KILL rpm-ostreed || true
                      systemctl kill -s KILL ostree-finalize-staged || true"
                    )?;
                }
                InterruptStrategy::Force(ForceInterruptStrategy::Reboot) => {
                    mark.reboot_strategy = Some(strategy.clone());
                    prepare_reboot(serde_json::to_string(&mark)?)?;
                    // This is a forced reboot - no syncing of the filesystem.
                    sh_execute!("reboot -ff")?;
                    anyhow::bail!("failed to reboot");
                }
                InterruptStrategy::Polite(PoliteInterruptStrategy::None) => {}
                InterruptStrategy::Polite(PoliteInterruptStrategy::Reboot) => {
                    mark.reboot_strategy = Some(strategy.clone());
                    Err(reboot(serde_json::to_string(&mark)?))?;
                }
                InterruptStrategy::Polite(PoliteInterruptStrategy::Stop) => {
                    sh_execute!(
                        "systemctl stop rpm-ostreed || true
                      systemctl stop ostree-finalize-staged || true"
                    )?;
                }
            }
        }
        last_strategy = Some(strategy);
    }
}

#[itest(destructive = true)]
fn transactionality() -> Result<()> {
    testinit()?;
    let mark = get_reboot_mark()?;
    let cancellable = Some(gio::Cancellable::new());
    let sysroot = ostree::Sysroot::new_default();
    sysroot.load(cancellable.as_ref())?;
    assert!(sysroot.is_booted());
    let booted = sysroot.get_booted_deployment().expect("booted deployment");
    let commit: String = booted.get_csum().expect("booted csum").into();
    // We need this static across reboots
    let srvrepo = Path::new(SRVREPO);
    let firstrun = !srvrepo.exists();
    if let Some(_) = mark.as_ref() {
        if firstrun {
            anyhow::bail!("Missing {:?}", srvrepo);
        }
    } else {
        if !firstrun {
            anyhow::bail!("Unexpected {:?}", srvrepo);
        }
        generate_srv_repo(&commit)?;
    }

    // Let's assume we're changing about 200 objects each time;
    // that leads to probably 300 network requests, so we want
    // a low average delay.
    let webserver_opts = TestHttpServerOpts {
        random_delay: Some(time::Duration::from_secs_f64(WEBSERVER_DELAY_SECS)),
        ..Default::default()
    };
    with_webserver_in(&srvrepo, &webserver_opts, move |addr| {
        let url = format!("http://{}", addr);
        sh_execute!(
            "ostree remote delete --if-exists testrepo
             ostree remote add --set=gpg-verify=false testrepo {url}",
            url = url
        )?;

        if firstrun {
            // Also disable some services (like zincati) because we don't want automatic updates
            // in our reboots, and it currently fails to start.  The less
            // we have in each reboot, the faster reboots are.
            sh_execute!("systemctl disable --now zincati fedora-coreos-pinger")?;
            // And prepare for updates
            sh_execute!("rpm-ostree cleanup -pr")?;
            generate_update(&commit)?;
            // Directly set the origin, so that we're not dependent on the pending deployment.
            // FIXME: make this saner
            sh_execute!(
                "
                ostree admin set-origin testrepo {url} {testref}
                ostree refs --create testrepo:{testref} {commit}
                ostree refs --create={origref} {commit}
                ",
                url = url,
                origref = ORIGREF,
                testref = TESTREF,
                commit = commit
            )?;
            // We gather a single "cycle time" at start as a way of gauging how
            // long an upgrade should take, so we know when to interrupt.  This
            // obviously has some pitfalls, mainly when there are e.g. other competing
            // VMs when we start but not after (or vice versa) we can either
            // interrupt almost always too early, or too late.
            let start = time::Instant::now();
            sh_execute!("rpm-ostree upgrade").context("Firstrun rebase failed")?;
            let end = time::Instant::now();
            let cycle_time = end.duration_since(start);
            let tdata = TransactionalTestInfo {
                cycle_time: cycle_time,
            };
            let mut f = std::io::BufWriter::new(std::fs::File::create(&TDATAPATH)?);
            serde_json::to_writer(&mut f, &tdata)?;
            f.flush()?;
            sh_execute!(
                "
                systemctl stop ostree-finalize-staged.service
                rpm-ostree status
                "
            )?;
        }

        let tdata = {
            let mut f = std::io::BufReader::new(std::fs::File::open(&TDATAPATH)?);
            serde_json::from_reader(&mut f).context("Failed to parse test info JSON")?
        };

        impl_transaction_test(commit.as_str(), &tdata, mark.as_ref())?;

        Ok(())
    })?;
    Ok(())
}

#[itest(destructive = true)]
fn other() -> Result<()> {
    testinit()?;
    Ok(())
}
