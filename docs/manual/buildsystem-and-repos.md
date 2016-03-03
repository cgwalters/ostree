# Writing a buildsystem and managing repositories

OSTree is not a package system.  It does not directly support building
source code, etc.  Rather, it is a tool for transporting and managing
content, along with package-system independent aspects like bootloader
management for updates.

We'll assume here that we're targeting doing commits on a build
server. having client-side tools assemble filesystem trees is also
possible of course, and it does carry some concerns

## Build vs buy

Therefore, you need to either use an existing tool for writing content
into an OSTree repository, or to write your own.  An example tool is
[rpm-ostree](https://github.com/projectatomic/rpm-ostree) - it takes
as input RPMs, and commits them (currently oriented for a server side,
but aiming to do client side too).

## Initializing

For this initial discussion, we're assuming you have a single
`archive-z2` repository:

```
mkdir repo
ostree --repo=repo init --mode=archive-z2
```

## Writing your own OSTree buildsystem

There exist many, many systems that basically follow this pattern:

```
$pkg --installroot=/path/to/tmpdir install foo bar baz
$imagesystem commit --root=/path/to/tmpdir
```

For various values of `$pkg` such as `yum`, `apt-get`, etc., and
values of `$imagesystem` could be simple tarballs, Amazon Machine
Images, ISOs, etc.

Now obviously in this document, we're going to talk about the
situation where `$imagesystem` is OSTree.  The general idea with
OSTree is that wherever you might store a series of tarballs for
applications or OS images, OSTree likely going to be better.  For
example, it supports GPG signatures, binary deltas, writing bootloader
configuration, etc.

OSTree does not include a component build system simply because there
already exist plenty of good ones - rather, it is intended to provide
an infrastructure layer.

The above mentioned `rpm-ostree compose tree` chooses RPM as the value
of `$pkg` - so binaries are built as RPMs, then committed as a whole
into an OSTree commit.

But let's discuss building our own.  If you're just experimenting,
it's quite easy to start with the command line.  We'll assume for this
purpose that you have a build process that outputs a directory tree -
we'll call this tool `$pkginstallroot` (which could be `yum
--installroot` or `dbootstrap`, etc.).

Your initial prototype is going to look like:

```
$pkginstallroot /path/to/tmpdir
ostree --repo=repo


Or more
generally, if your build system can generate a tarball, you can commit
that tarball into OSTree.  For example,
[OpenEmbedded](http://www.openembedded.org/) can output a tarball, and
one can commit it via `ostree commit -b myos/x86_64/branch
--tree=tar=myos.tar`.


## Building your own

