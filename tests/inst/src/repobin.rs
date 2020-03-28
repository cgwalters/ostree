//! Tests that mostly use the CLI and operate on temporary
//! repositories.

use std::path::Path;

use anyhow::{Context, Result};
use commandspec::{sh_command, sh_execute};
use tokio::runtime::Runtime;

use crate::test::*;

pub(crate) fn tests() -> impl IntoIterator<Item = Test> {
    crate::deftests_map!(
        crate::test::with_tmpdir,
        test_nofifo,
        test_mtime,
        test_extensions,
        test_pull_basicauth
    )
}

fn test_nofifo(tmp_dir: &Path) -> Result<()> {
    sh_execute!(
        r"cd {tmp_dir}
    ostree --repo=repo init --mode=archive
    mkdir tmproot
    mkfifo tmproot/afile
",
        tmp_dir = tmp_dir.to_str()
    )?;
    cmd_fails_with(
        sh_command!(
            r#"cd {tmp_dir}
ls -al
ostree --repo=repo commit -b fifotest -s "commit fifo" --tree=dir=./tmproot"#,
            tmp_dir = tmp_dir.to_str()
        )
        .unwrap(),
        "Not a regular file or symlink",
    )?;
    Ok(())
}

fn test_mtime(tmp_dir: &Path) -> Result<()> {
    sh_execute!(
        r"cd {tmp_dir}
    ostree --repo=repo init --mode=archive
    mkdir tmproot
    echo afile > tmproot/afile
    ostree --repo=repo commit -b test --tree=dir=tmproot >/dev/null
",
        tmp_dir = tmp_dir.to_str()
    )?;
    let ts = tmp_dir.join("repo").metadata()?.modified().unwrap();
    sh_execute!(
        r#"cd {tmp_dir}
    ostree --repo=repo commit -b test -s "bump mtime" --tree=dir=tmproot >/dev/null"#,
        tmp_dir = tmp_dir.to_str()
    )?;
    assert_ne!(ts, tmp_dir.join("repo").metadata()?.modified().unwrap());
    Ok(())
}

fn test_extensions(tmp_dir: &Path) -> Result<()> {
    sh_execute!(
        r"ostree --repo={tmp_dir}/repo init --mode=bare",
        tmp_dir = tmp_dir.to_str()
    )?;
    assert!(tmp_dir.join("repo/extensions").exists());
    Ok(())
}

async fn impl_test_pull_basicauth(tmp_dir: &Path) -> Result<()> {
    let opts = TestHttpServerOpts {
        basicauth: true,
        ..Default::default()
    };
    let serverrepo = tmp_dir.join("server/repo");
    std::fs::create_dir_all(&serverrepo)?;
    let addr = http_server(&serverrepo, opts).await?;
    let tmp_dir = tmp_dir.to_path_buf();
    tokio::task::spawn_blocking(move || -> Result<()> {
        let baseuri = http::Uri::from_maybe_shared(format!("http://{}/", addr).into_bytes())?;
        let unauthuri =
            http::Uri::from_maybe_shared(format!("http://unknown:badpw@{}/", addr).into_bytes())?;
        let authuri = http::Uri::from_maybe_shared(
            format!("http://{}@{}/", TEST_HTTP_BASIC_AUTH, addr).into_bytes(),
        )?;
        let osroot = tmp_dir.join("osroot");
        mkroot(&osroot)?;
        sh_execute!(
            r#"cd {tmp_dir}
        ostree --repo={serverrepo} init --mode=archive
        ostree --repo={serverrepo} commit -b os --tree=dir={osroot} >/dev/null
        mkdir client
        cd client
        ostree --repo=repo init --mode=archive
        ostree --repo=repo remote add --set=gpg-verify=false origin-unauth {baseuri}
        ostree --repo=repo remote add --set=gpg-verify=false origin-badauth {unauthuri}
        ostree --repo=repo remote add --set=gpg-verify=false origin-goodauth {authuri}
        "#,
            tmp_dir = tmp_dir.to_str(),
            osroot = osroot.to_str(),
            serverrepo = serverrepo.to_str(),
            baseuri = baseuri.to_string(),
            unauthuri = unauthuri.to_string(),
            authuri = authuri.to_string()
        )?;
        for rem in &["unauth", "badauth"] {
            cmd_fails_with(
                sh_command!(
                    r#"ostree --repo={tmp_dir}/client/repo pull origin-{rem} os >/dev/null"#,
                    tmp_dir = tmp_dir.to_str(),
                    rem = *rem
                )
                .unwrap(),
                "HTTP 403",
            )
            .context(rem)?;
        }
        sh_execute!(
            r#"ostree --repo={tmp_dir}/client/repo pull origin-goodauth os >/dev/null"#,
            tmp_dir = tmp_dir.to_str()
        )?;
        Ok(())
    })
    .await??;
    Ok(())
}

fn test_pull_basicauth(tmp_dir: &Path) -> Result<()> {
    let mut rt = Runtime::new()?;
    rt.block_on(async move { impl_test_pull_basicauth(tmp_dir).await })?;
    Ok(())
}
