//! Test that interrupting a deploy is safe
use anyhow::{Context, Result};
use commandspec::sh_execute;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::Path;
use std::time;

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
/// Multiply the average cycle time by this to ensure we sometimes
/// fail to interrupt too.
const FORCE_REBOOT_AFTER_MUL: f64 = 1.2f64;
/// Amount of time in seconds we will delay each web request.
/// FIXME: this should be a function of total number of objects or so
const WEBSERVER_DELAY_SECS: f64 = 0.005;

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

#[derive(Serialize, Deserialize, Debug)]
struct RebootMark {
    iter: u32,
    before: u32,
    success: u32,
    interrupted: u32,
}

// fn transactional_test_kill<M: AsRef<str>, T: AsRef<str>>(cycle_time: &time::Duration, target_commit: T) -> Result<()> {
//     let target_commit = target_commit.as_ref();
//     sh_execute!("rpm-ostree cleanup -pr")?;
//     let status = rpmostree::query_status()?;
//     let firstdeploy = &status.deployments[0];
//     let orig_commit = &firstdeploy.checksum;
//     let ITERATIONS = 10u32;
//     let current_iteration = if let Some(mark) = mark {
//         mark.as_ref().parse()?
//     } else {
//         0
//     };
//     println!(
//         "Using upgrade cycle_time={:#?} on iteration={}/{}",
//         tdata.cycle_time, current_iteration, ITERATIONS
//     );
//     let elapsed_div = tdata.cycle_time / ITERATIONS;
//     let mut ok_staged = 0;
//     let mut ok_finalized = 0;
//     let mut errs_not_deployed = 0;
//     let mut errs_staged = 0;
//     let mut errs_finalized = 0;
//     let mut rng = rand::thread_rng();
//     for i in 0..ITERATIONS {
//         println!("iteration={}", i);
//         let r = thread::spawn(run_cycle);

//         thread::sleep(rng.gen_range(0, ITERATIONS) * elapsed_div);
//         match tdata.ttype {
//             TransactionalTestType::Kill => {
//                 let _ = sh_execute!(
//                 "systemctl kill -s KILL rpm-ostreed
//                 systemctl kill -s KILL ostree-finalize-staged"
//             );
//         },
//         TransactionalTestType::ForceReboot => {
//             prepare_reboot()
//         }

//         let res = r.join().expect("join");
//         // The systemctl stop above will exit successfully even if
//         // the process actually died from SIGKILL
//         let res = if let Ok(_) = res.as_ref() {
//             let finalize_res = process::Command::new("systemctl")
//                 .args(&["is-failed", "ostree-finalize-staged.service"])
//                 .output()?;
//             if std::str::from_utf8(&finalize_res.stdout)? == "failed\n" {
//                 Err(anyhow::anyhow!("ostree-finalize-staged.service failed"))
//             } else {
//                 res
//             }
//         } else {
//             res
//         };

//         // This effectively re-validates consistency; TODO
//         // add more validation periodically like e.g
//         // `ostree admin fsck`.
//         let status = rpmostree::query_status()?;
//         let firstdeploy = &status.deployments[0];
//         let deployresult = if firstdeploy.checksum == target_commit {
//             if let Some(true) = firstdeploy.staged {
//                 DeployResult::Staged
//             } else {
//                 DeployResult::Finalized
//             }
//         } else if &firstdeploy.checksum == orig_commit {
//             DeployResult::NotDeployed
//         } else {
//             anyhow::bail!("Unexpected target commit: {}", firstdeploy.checksum);
//         };
//         match (res, deployresult) {
//             (Ok(_), DeployResult::NotDeployed) => {
//                 anyhow::bail!("Got successful result but not deployed!")
//             }
//             (Ok(_), DeployResult::Staged) => ok_staged += 1,
//             (Ok(_), DeployResult::Finalized) => ok_finalized += 1,
//             (Err(_), DeployResult::NotDeployed) => errs_not_deployed += 1,
//             (Err(_), DeployResult::Staged) => errs_staged += 1,
//             (Err(_), DeployResult::Finalized) => errs_finalized += 1,
//         };
//     }
//     println!(
//         "ITERATIONS={} staged={} finalized={} errs=(undeployed={}, staged={}, finalized={})",
//         ITERATIONS, ok_staged, ok_finalized, errs_not_deployed, errs_staged, errs_finalized
//     );
//     Ok(())
// }

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

fn transactional_test_forcepoweroff<M: AsRef<str>>(
    booted_commit: &str,
    tdata: &TransactionalTestInfo,
    mark: Option<M>,
) -> Result<()> {
    let cancellable = Some(gio::Cancellable::new());
    let srvrepo_obj = ostree::Repo::new(&gio::File::new_for_path(SRVREPO));
    srvrepo_obj.open(cancellable.as_ref())?;
    let target_commit: String = srvrepo_obj.resolve_rev(TESTREF, false)?.into();
    let first_commit: String = {
        let sysrepo_obj = ostree::Repo::new(&gio::File::new_for_path("/sysroot/ostree/repo"));
        sysrepo_obj.open(gio::NONE_CANCELLABLE)?;
        sysrepo_obj.resolve_rev(ORIGREF, false)?.into()
    };
    let orig_commit: String = srvrepo_obj
        .resolve_rev(&format!("{}^", TESTREF), false)?
        .into();
    let mut mark = if let Some(mark) = mark {
        let markstr = mark.as_ref();
        let mut mark: RebootMark = serde_json::from_str(markstr)
            .with_context(|| format!("Failed to parse reboot mark {:?}", markstr))?;
        // The first failed reboot may be into the original booted commit
        let status = rpmostree::query_status()?;
        let firstdeploy = &status.deployments[0];
        // The first deployment should not be staged
        assert!(!firstdeploy.staged.unwrap_or(false));
        assert!(firstdeploy.booted);
        assert_eq!(firstdeploy.checksum, booted_commit);
        if booted_commit == target_commit {
            mark.success += 1;
            println!("Successfully updated to {}", target_commit);
            generate_update(&firstdeploy.checksum)?;
        } else if booted_commit == orig_commit || booted_commit == first_commit {
            println!(
                "Failed update to {} (booted={})",
                target_commit, booted_commit
            );
            mark.interrupted += 1
        } else {
            anyhow::bail!("Unexpected target commit: {}", firstdeploy.checksum);
        };
        mark
    } else {
        RebootMark {
            iter: 0,
            before: 0,
            success: 0,
            interrupted: 0,
        }
    };
    if mark.iter == ITERATIONS {
        // TODO also add ostree admin fsck to check the deployment directories
        sh_execute!(
            "echo Performing final validation...
            ostree fsck"
        )?;
        return Ok(());
    }

    let mut rt = tokio::runtime::Runtime::new()?;
    for i in 0..ITERATION_RETRIES {
        let mut rng = rand::thread_rng();
        let cycle_time_ms =
            (tdata.cycle_time.as_secs_f64() * 1000f64 * FORCE_REBOOT_AFTER_MUL) as u64;
        // Initiate a forced reboot usually the upgrade would
        // complete, but also ~30% of the time after.
        let sleeptime = time::Duration::from_millis(rng.gen_range(0, cycle_time_ms));
        println!(
            "force-reboot-time={:?} cycle={:?} status:{:?}",
            sleeptime, tdata.cycle_time, &mark
        );
        sh_execute!(
            "
            rpm-ostree status
            systemctl stop rpm-ostreed
            ostree reset testrepo:{testref} {booted_commit}
            rpm-ostree cleanup -pbrm
            ",
            testref = TESTREF,
            booted_commit = booted_commit
        )?;
        let res: Result<bool> = rt.block_on(async move { run_upgrade_or_timeout(sleeptime).await });
        let res = res?;
        if res {
            println!(
                "Failed to interrupt upgrade, attempt {}/{}",
                i, ITERATION_RETRIES
            );
            mark.before += 1;
        } else {
            mark.iter += 1;
            prepare_reboot(serde_json::to_string(&mark)?)?;
            // This is a forced reboot - no syncing of the filesystem.
            sh_execute!("reboot -ff")?;
            anyhow::bail!("failed to reboot");
        }
    }
    Err(anyhow::anyhow!(
        "Failed to interrupt upgrade {} times",
        ITERATION_RETRIES
    ))
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

        transactional_test_forcepoweroff(commit.as_str(), &tdata, mark.as_ref())?;

        Ok(())
    })?;
    Ok(())
}

#[itest(destructive = true)]
fn other() -> Result<()> {
    testinit()?;
    Ok(())
}
