#!/bin/bash
#
# Copyright (C) 2017 Colin Walters <walters@verbum.org>
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

set -euo pipefail

. $(dirname $0)/libtest.sh

setup_exampleos_repo

echo '1..1'

cd ${test_tmpdir}
set -x

echo "$(date): Pulling content..."
rev=$(${CMD_PREFIX} ostree --repo=ostree-srv/exampleos/repo rev-parse ${REF})
${CMD_PREFIX} ostree --repo=repo pull --disable-static-deltas --mirror origin ${REF}
${CMD_PREFIX} ostree --repo=repo fsck
assert_streq ${rev} $(${CMD_PREFIX} ostree --repo=repo rev-parse ${REF})

echo "ok"
