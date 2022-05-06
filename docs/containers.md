---
nav_order: 10
---

# Using ostree-native containers
{: .no_toc }

1. TOC
{:toc}

## Docker/OCI containers and ostree

It is quite common to pair a distinct host update mechanism (dpkg/rpm, ostree, a dm-verity system, etc.)
with a container mechanism (podman/docker/kubernetes, etc).  This was the model
used by the original [CoreOS Container Linux](https://en.wikipedia.org/wiki/Container_Linux)
for example (using a dual-partition scheme).

The [OCI image standard](https://github.com/opencontainers/image-spec/) is basically
a standardization of the Docker container image schema.  

There is a lot of overlap specifically between ostree and how OCI containers work
at a high level.  Conceptually, an ostree commit can be thought of much like
a tarball, just with content-addressed files and built in signing (e.g. GPG).
OSTree uses GVariant (a somewhat standardized binary format) for metadata.

The OSTree default model is heavily oriented around having a *single* final "flattened"
filesystem tree for booting, but the storage system is much more flexible; it
can accomodate a large number of independently versioned filesystem trees.

OCI containers are basically a series of tarballs wrapped with JSON for metadata
that are usually managed on disk by `overlayfs` or equivalent - they are also
effectively partial versioned filesystem trees.

## "ostree-native containers"

As of recently, the [ostree-rs-ext project](https://github.com/ostreedev/ostree-rs-ext/)
implements sophisticated tooling which maps between OSTree and (OCI) container images.

Crucially, it becomes possible to directly deploy and upgrade from a container image.

Key advantages of this:

 - One can use any Docker/OCI registry to store, version and manage OS images
 - Any container build system can be used to make derived images

In this model, ostree can be thought of primarily as a tool to boot and
upgrade from a container image.  Perhaps a bit like a `bootc`, with analogy
to `runc/crun`.

## Encapsulating an ostree commit

For users already using ostree "natively", the new `ostree container encapsulate`
command can be used to take an ostree commit, and generate an OCI
container which wraps it.  This is a relatively naive tool - it generates
a container image with a single tar layer by default

<!-- SPDX-License-Identifier: (CC-BY-SA-3.0 OR GFDL-1.3-or-later) -->

