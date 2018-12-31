# Shell API for coreos-run-qemu
#
# Copyright (C) 2018 Red Hat, Inc.
#
# This library is free software; you can redistribute it and/or
# modify it under the terms of the GNU Lesser General Public
# License as published by the Free Software Foundation; either
# version 2 of the License, or (at your option) any later version.
#
# This library is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
# Lesser General Public License for more details.
#
# You should have received a copy of the GNU Lesser General Public
# License along with this library; if not, write to the
# Free Software Foundation, Inc., 59 Temple Place - Suite 330,
# Boston, MA 02111-1307, USA.

# prepares the VM and library for action
VMDIR=
dn=$(cd $(dirname $0) && pwd)

coreos_qemutest_setup() {
    dir=${1:-vm}

    if [ -d "${dir}" ]; then
        echo "VM already running?  ${dir} exists"
        exit 1
    fi

    # https://stackoverflow.com/questions/8295908/how-to-use-a-variable-to-indicate-a-file-descriptor-in-bash
    # Kind of lame there's no bash primitive for pipe()
    # Anyways our qemu wrapper watches this fd and exits if it's closed, which
    # also kills qemu
    exec {bindfd}<>/dev/null
    ${dn}/coreos-run-qemu --bind-fd=${bindfd} --disk=${FEDORA_COREOS_DISK:-${dn}/fedora-coreos-qemu.qcow2} --dir="${dir}" --mount-ro ${dn}:/var/srv/testcode
}

# run command in vm
# - $@    command to run
vm_cmd() {
  python3 -c 'import os,sys,json,asyncio
vmdir = sys.argv[1]
cmd = sys.argv[2:]
async def run():
    (r,w) = await asyncio.open_unix_connection(f"{vmdir}/cmd.sock")
    w.write(json.dumps({"cmd": "exec", "args": cmd}).encode("UTF-8"))
    await w.drain()
    r = json.loads(await r.readline())
    sys.stderr.write(r["stderr"])
    sys.stdout.write(r["stdout"])
    sys.exit(int(r["rc"]))
asyncio.run(run())
' "${VMDIR}" "$@"
}

# run rpm-ostree in vm
# - $@    args
vm_rpmostree() {
    vm_cmd env ASAN_OPTIONS=detect_leaks=false rpm-ostree "$@"
}

vm_get_boot_id() {
  vm_cmd cat /proc/sys/kernel/random/boot_id
}

# Reboot via special API
vm_reboot() {
    vm_cmd sync
    local orig_bootid=$(vm_get_boot_id 2>/dev/null)
    python3 -c 'import os,sys,json,asyncio
vmdir = sys.argv[1]
async def run():
    (r,w) = await asyncio.open_unix_connection(f"{vmdir}/cmd.sock")
    w.write(json.dumps({"cmd": "reboot"}))
    await w.drain()
    await r.readline()
    sys.exit(0)
asyncio.run(run())
' "${VMDIR}"
    # This one should queue locally
    local bootid=$(vm_get_boot_id 2>/dev/null)
    if [ "${orig_bootid}" = "${bootid}" ]; then
        echo "failed to reboot?"
        exit 1
    fi
}


# check that the given files/dirs exist on the VM
# - $@    files/dirs to check for
vm_has_files() {
  for file in "$@"; do
    if ! vm_cmd test -e $file; then
        return 1
    fi
  done
}

# check that the packages are installed
# - $@    packages to check for
vm_has_packages() {
  for pkg in "$@"; do
    if ! vm_cmd rpm -q $pkg; then
        return 1
    fi
  done
}

# retrieve info from a deployment
# - $1   index of deployment (or -1 for booted)
# - $2   key to retrieve
vm_get_deployment_info() {
  local idx=$1
  local key=$2
  vm_rpmostree status --json | \
    python -c "
import sys, json
deployments = json.load(sys.stdin)[\"deployments\"]
idx = $idx
if idx < 0:
  for i, depl in enumerate(deployments):
    if depl[\"booted\"]:
      idx = i
if idx < 0:
  print \"Failed to determine currently booted deployment\"
  exit(1)
if idx >= len(deployments):
  print \"Deployment index $idx is out of range\"
  exit(1)
depl = deployments[idx]
if \"$key\" in depl:
  data = depl[\"$key\"]
  if type(data) is list:
    print \" \".join(data)
  else:
    print data
"
}

# retrieve the deployment root
# - $1   index of deployment
vm_get_deployment_root() {
  local idx=$1
  local csum=$(vm_get_deployment_info $idx checksum)
  local serial=$(vm_get_deployment_info $idx serial)
  local osname=$(vm_get_deployment_info $idx osname)
  echo /ostree/deploy/$osname/deploy/$csum.$serial
}

# retrieve info from the booted deployment
# - $1   key to retrieve
vm_get_booted_deployment_info() {
  vm_get_deployment_info -1 $1
}

# print the layered packages
vm_get_layered_packages() {
  vm_get_booted_deployment_info packages
}

# print the requested packages
vm_get_requested_packages() {
  vm_get_booted_deployment_info requested-packages
}

vm_get_local_packages() {
  vm_get_booted_deployment_info requested-local-packages
}

# check that the packages are currently layered
# - $@    packages to check for
vm_has_layered_packages() {
  local pkgs=$(vm_get_layered_packages)
  for pkg in "$@"; do
    if [[ " $pkgs " != *$pkg* ]]; then
        return 1
    fi
  done
}

# check that the packages are currently requested
# - $@    packages to check for
vm_has_requested_packages() {
  local pkgs=$(vm_get_requested_packages)
  for pkg in "$@"; do
    if [[ " $pkgs " != *$pkg* ]]; then
        return 1
    fi
  done
}

vm_has_local_packages() {
  local pkgs=$(vm_get_local_packages)
  for pkg in "$@"; do
    if [[ " $pkgs " != *$pkg* ]]; then
        return 1
    fi
  done
}

vm_has_dormant_packages() {
  vm_has_requested_packages "$@" && \
    ! vm_has_layered_packages "$@"
}

vm_get_booted_stateroot() {
    vm_get_booted_deployment_info osname
}

# retrieve the checksum of the currently booted deployment
vm_get_booted_csum() {
  vm_get_booted_deployment_info checksum
}

# retrieve the checksum of the pending deployment
vm_get_pending_csum() {
  vm_get_deployment_info 0 checksum
}

# make multiple consistency checks on a test pkg
# - $1    package to check for
# - $2    either "present" or "absent"
vm_assert_layered_pkg() {
  local pkg=$1; shift
  local policy=$1; shift

  set +e
  vm_has_packages $pkg;         pkg_in_rpmdb=$?
  vm_has_layered_packages $pkg; pkg_is_layered=$?
  vm_has_local_packages $pkg;   pkg_is_layered_local=$?
  vm_has_requested_packages $pkg; pkg_is_requested=$?
  [ $pkg_in_rpmdb == 0 ] && \
  ( ( [ $pkg_is_layered == 0 ] &&
      [ $pkg_is_requested == 0 ] ) ||
    [ $pkg_is_layered_local == 0 ] ); pkg_present=$?
  [ $pkg_in_rpmdb != 0 ] && \
  [ $pkg_is_layered != 0 ] && \
  [ $pkg_is_layered_local != 0 ] && \
  [ $pkg_is_requested != 0 ]; pkg_absent=$?
  set -e

  if [ $policy == present ] && [ $pkg_present != 0 ]; then
    vm_cmd rpm-ostree status
    assert_not_reached "pkg $pkg is not present"
  fi

  if [ $policy == absent ] && [ $pkg_absent != 0 ]; then
    vm_cmd rpm-ostree status
    assert_not_reached "pkg $pkg is not absent"
  fi
}

# Takes a list of `jq` expressions, each of which should evaluate to a boolean,
# and asserts that they are true.
vm_assert_status_jq() {
    vm_rpmostree status --json > status.json
    vm_rpmostree status > status.txt
    for expression in "$@"; do
        if ! jq -e "${expression}" >/dev/null < status.json; then
            jq . < status.json | sed -e 's/^/# /' >&2
            echo 1>&2 "${expression} failed to match status.json"
            cat status.txt
            exit 1
        fi
    done
}

vm_pending_is_staged() {
    vm_rpmostree status --json > status-staged.json
    local rc=1
    if jq -e ".deployments[0][\"staged\"]" < status-staged.json; then
        rc=0
    fi
    rm -f status-staged.json
    return $rc
}

vm_get_journal_cursor() {
  vm_cmd journalctl -o json -n 1 | jq -r '.["__CURSOR"]'
}

# Minor helper that makes sure to get quoting right
vm_get_journal_after_cursor() {
  from_cursor=$1; shift
  to_file=$1; shift
  # add an extra helping of quotes for hungry ssh
  vm_cmd journalctl --after-cursor "'$from_cursor'" > $to_file
}

vm_assert_journal_has_content() {
  from_cursor=$1; shift
  vm_get_journal_after_cursor $from_cursor tmp-journal.txt
  assert_file_has_content tmp-journal.txt "$@"
  rm -f tmp-journal.txt
}

# $1 - service name
# $2 - dir to serve
# $3 - port to serve on
vm_start_httpd() {
  local name=$1; shift
  local dir=$1; shift
  local port=$1; shift

  # just nuke the service of the same name if it exists and is also transient
  if vm_cmd systemctl show $name | grep -q UnitFileState=transient; then
    vm_cmd systemctl stop $name
  fi

  # CentOS systemd is too old for -p WorkingDirectory
  vm_cmd systemd-run --unit $name sh -c \
    "'cd $dir && python -m SimpleHTTPServer $port'"

  # NB: the EXIT trap is used by libtest, but not the ERR trap
  trap "vm_stop_httpd $name" ERR
  set -E # inherit trap

  # Ideally systemd-run would support .socket units or something
  vm_cmd 'while ! curl --head http://127.0.0.1:8888 &>/dev/null; do sleep 1; done'
}

# $1 - service name
vm_stop_httpd() {
  local name=$1; shift
  vm_cmd systemctl stop $name
  set +E
  trap - ERR
}

# start up an ostree server to be used as an http remote
vm_ostreeupdate_prepare_repo() {
  # Really testing this like a user requires a remote ostree server setup.
  # Let's start by setting up the repo.
  REMOTE_OSTREE=/ostree/repo/tmp/vmcheck-remote
  vm_cmd mkdir -p $REMOTE_OSTREE
  vm_cmd ostree init --repo=$REMOTE_OSTREE --mode=archive
  vm_start_httpd ostree_server $REMOTE_OSTREE 8888
}
