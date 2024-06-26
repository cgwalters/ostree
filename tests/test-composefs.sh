#!/bin/bash
#
# SPDX-License-Identifier: LGPL-2.0+
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
# License along with this library. If not, see <https://www.gnu.org/licenses/>.

set -euo pipefail

. $(dirname $0)/libtest.sh

skip_without_ostree_feature composefs
skip_without_user_xattrs

setup_test_repository "bare-user"

cd ${test_tmpdir}
$OSTREE checkout test2 test2-co
rm test2-co/whiteouts -rf  # This may or may not exist
COMMIT_ARGS="--owner-uid=0 --owner-gid=0 --no-xattrs --canonical-permissions"
$OSTREE commit ${COMMIT_ARGS} -b test-composefs --generate-composefs-metadata test2-co
# If the test fails we'll dump this out
$OSTREE ls -RCX test-composefs /
orig_composefs_digest=$($OSTREE show --print-hex --print-metadata-key ostree.composefs.digest.v0 test-composefs)
$OSTREE commit ${COMMIT_ARGS} -b test-composefs2 --generate-composefs-metadata test2-co
new_composefs_digest=$($OSTREE show --print-hex --print-metadata-key ostree.composefs.digest.v0 test-composefs2)
assert_streq "${orig_composefs_digest}" "${new_composefs_digest}"
assert_streq "${new_composefs_digest}" "be956966c70970ea23b1a8043bca58cfb0d011d490a35a7817b36d04c0210954"
tap_ok "composefs metadata"

rm test2-co -rf
$OSTREE checkout --composefs test-composefs test2-co.cfs
digest=$(sha256sum < test2-co.cfs | cut -f 1 -d ' ')
# This file should be reproducible bit for bit across environments; per above
# we're operating on predictable data (fixed uid, gid, timestamps, xattrs, permissions).
assert_streq "${digest}" "031fab2c7f390b752a820146dc89f6880e5739cba7490f64024e0c7d11aad7c9"
# Verify it with composefs tooling
composefs-info dump test2-co.cfs >/dev/null
tap_ok "checkout composefs"

tap_end
