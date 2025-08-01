/*
 * Copyright (C) 2012,2013 Colin Walters <walters@verbum.org>
 *
 * SPDX-License-Identifier: LGPL-2.0+
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Lesser General Public
 * License as published by the Free Software Foundation; either
 * version 2 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public
 * License along with this library. If not, see <https://www.gnu.org/licenses/>.
 */

#pragma once

#include "libglnx.h"
#include "ostree-bootloader.h"
#include "ostree.h"

G_BEGIN_DECLS

typedef enum
{

  /* Don't flag deployments as immutable. */
  OSTREE_SYSROOT_DEBUG_MUTABLE_DEPLOYMENTS = 1 << 0,
  /* See https://github.com/ostreedev/ostree/pull/759 */
  OSTREE_SYSROOT_DEBUG_NO_XATTRS = 1 << 1,
  /* https://github.com/ostreedev/ostree/pull/1049 */
  OSTREE_SYSROOT_DEBUG_TEST_FIFREEZE = 1 << 2,
  /* This is a temporary flag until we fully drop the explicit `systemctl start
   * ostree-finalize-staged.service` so that tests can exercise the new path unit. */
  OSTREE_SYSROOT_DEBUG_TEST_NO_DTB = 1 << 3, /* https://github.com/ostreedev/ostree/issues/2154 */
} OstreeSysrootDebugFlags;

typedef enum
{
  /* Skip invoking `sync()` */
  OSTREE_SYSROOT_GLOBAL_OPT_SKIP_SYNC = 1 << 0,
  /* See https://github.com/ostreedev/ostree/pull/2847 */
  OSTREE_SYSROOT_GLOBAL_OPT_NO_EARLY_PRUNE = 1 << 1,
  OSTREE_SYSROOT_GLOBAL_OPT_BOOTLOADER_NAMING_1 = 1 << 2,
} OstreeSysrootGlobalOptFlags;

typedef enum
{
  OSTREE_SYSROOT_LOAD_STATE_NONE,   /* ostree_sysroot_new() was called */
  OSTREE_SYSROOT_LOAD_STATE_INIT,   /* We've loaded basic sysroot state and have an fd */
  OSTREE_SYSROOT_LOAD_STATE_LOADED, /* We've loaded all of the deployments */
} OstreeSysrootLoadState;

/**
 * OstreeSysroot:
 * Internal struct
 */
struct OstreeSysroot
{
  GObject parent;

  GFile *path;
  // File descriptor for the sysroot. Only valid after `ostree_sysroot_ensure_initialized()`
  // has been invoked (directly by a calling program, or transitively from another public API).
  int sysroot_fd;
  // File descriptor for the boot partition. Should be initialized on demand internally
  // by a public API eventually invoking `_ostree_sysroot_ensure_boot_fd()`.
  int boot_fd;
  // Set if the /boot filesystem is VFAT.
  // Only initialized if boot_fd is set.
  gboolean boot_is_vfat;
  // Lock for this sysroot.
  GLnxLockFile lock;

  OstreeSysrootLoadState loadstate;
  gboolean mount_namespace_in_use; /* TRUE if caller has told us they used CLONE_NEWNS */
  gboolean root_is_ostree_booted;  /* TRUE if sysroot is / and we are booted via ostree */
  /* The device/inode for / and /etc, used to detect booted deployment */
  dev_t root_device;
  ino_t root_inode;
  /* The device inode for a queued soft reboot deployment */
  gboolean have_nextroot;
  dev_t nextroot_device;
  ino_t nextroot_inode;

  // The parsed data from /run/ostree
  GVariantDict *run_ostree_metadata;

  gboolean is_physical; /* TRUE if we're pointed at physical storage root and not a deployment */
  GPtrArray *deployments;
  int bootversion;
  int subbootversion;
  OstreeDeployment *booted_deployment;
  OstreeDeployment *staged_deployment;
  GVariant *staged_deployment_data;
  // True if loaded_ts is initialized
  gboolean has_loaded;
  struct timespec loaded_ts;

  /* Only access through ostree_sysroot_[_get]repo() */
  OstreeRepo *repo;

  OstreeSysrootGlobalOptFlags opt_flags;
  OstreeSysrootDebugFlags debug_flags;
};

/* Key in staged deployment variant for finalization locking */
#define _OSTREE_SYSROOT_STAGED_KEY_LOCKED "locked"

#define OSTREE_SYSROOT_LOCKFILE "ostree/lock"
/* We keep some transient state in /run */
#define _OSTREE_SYSROOT_RUNSTATE_STAGED "/run/ostree/staged-deployment"
#define _OSTREE_SYSROOT_RUNSTATE_STAGED_LOCKED "/run/ostree/staged-deployment-locked"
#define _OSTREE_SYSROOT_RUNSTATE_STAGED_INITRDS_DIR "/run/ostree/staged-initrds/"
#define _OSTREE_SYSROOT_DEPLOYMENT_RUNSTATE_DIR "/run/ostree/deployment-state/"
#define _OSTREE_SYSROOT_DEPLOYMENT_RUNSTATE_FLAG_DEVELOPMENT "unlocked-development"
#define _OSTREE_SYSROOT_DEPLOYMENT_RUNSTATE_FLAG_TRANSIENT "unlocked-transient"

#define _OSTREE_SYSROOT_BOOT_INITRAMFS_OVERLAYS "ostree/initramfs-overlays"
#define _OSTREE_SYSROOT_INITRAMFS_OVERLAYS "boot/" _OSTREE_SYSROOT_BOOT_INITRAMFS_OVERLAYS

// Relative to /boot, consumed by ostree-boot-complete.service
#define _OSTREE_FINALIZE_STAGED_FAILURE_PATH "ostree/finalize-failure.stamp"

gboolean _ostree_sysroot_ensure_writable (OstreeSysroot *self, GError **error);

// Should be preferred over ostree_deployment_new
OstreeDeployment *_ostree_sysroot_new_deployment_object (OstreeSysroot *self, const char *osname,
                                                         const char *csum, int deployserial,
                                                         const char *bootcsum, int bootserial,
                                                         GError **error);

void _ostree_sysroot_emit_journal_msg (OstreeSysroot *self, const char *msg);

gboolean _ostree_sysroot_read_boot_loader_configs (OstreeSysroot *self, int bootversion,
                                                   GPtrArray **out_loader_configs,
                                                   GCancellable *cancellable, GError **error);

gboolean _ostree_sysroot_read_current_subbootversion (OstreeSysroot *self, int bootversion,
                                                      int *out_subbootversion,
                                                      GCancellable *cancellable, GError **error);

gboolean _ostree_sysroot_parse_deploy_path_name (const char *name, char **out_csum, int *out_serial,
                                                 GError **error);

gboolean _ostree_sysroot_list_deployment_dirs_for_os (OstreeSysroot *self, int deploydir_dfd,
                                                      const char *osname,
                                                      GPtrArray *inout_deployments,
                                                      GCancellable *cancellable, GError **error);

void _ostree_deployment_set_bootconfig_from_kargs (OstreeDeployment *deployment,
                                                   char **override_kernel_argv);

gboolean _ostree_sysroot_reload_staged (OstreeSysroot *self, GError **error);

gboolean _ostree_sysroot_finalize_staged (OstreeSysroot *self, GCancellable *cancellable,
                                          GError **error);
gboolean _ostree_sysroot_boot_complete (OstreeSysroot *self, GCancellable *cancellable,
                                        GError **error);

gboolean _ostree_prepare_soft_reboot (GError **error);

OstreeDeployment *_ostree_sysroot_deserialize_deployment_from_variant (OstreeSysroot *self,
                                                                       GVariant *v, GError **error);

char *_ostree_sysroot_get_deployment_backing_relpath (OstreeDeployment *deployment);

gboolean _ostree_sysroot_rmrf_deployment (OstreeSysroot *sysroot, OstreeDeployment *deployment,
                                          GCancellable *cancellable, GError **error);

gboolean _ostree_sysroot_stateroot_legacy_var_init (int dfd, GError **error);

char *_ostree_sysroot_get_runstate_path (OstreeDeployment *deployment, const char *key);

gboolean _ostree_sysroot_run_in_deployment (int deployment_dfd, const char *const *bwrap_argv,
                                            const gchar *const *child_argv, gint *exit_status,
                                            gchar **stdout, GError **error);

char *_ostree_sysroot_join_lines (GPtrArray *lines);

gboolean _ostree_sysroot_ensure_boot_fd (OstreeSysroot *self, GError **error);

gboolean _ostree_sysroot_query_bootloader (OstreeSysroot *sysroot,
                                           OstreeBootloader **out_bootloader,
                                           GCancellable *cancellable, GError **error);

gboolean _ostree_sysroot_bump_mtime (OstreeSysroot *sysroot, GError **error);

gboolean _ostree_sysroot_cleanup_internal (OstreeSysroot *sysroot, gboolean prune_repo,
                                           GCancellable *cancellable, GError **error);

gboolean _ostree_sysroot_cleanup_bootfs (OstreeSysroot *self, GCancellable *cancellable,
                                         GError **error);

gboolean _ostree_sysroot_parse_bootdir_name (const char *name, char **out_osname, char **out_csum);

gboolean _ostree_sysroot_list_all_boot_directories (OstreeSysroot *self, char ***out_bootdirs,
                                                    GCancellable *cancellable, GError **error);

gboolean _ostree_sysroot_parse_bootlink (const char *bootlink, int *out_entry_bootversion,
                                         char **out_osname, char **out_bootcsum,
                                         int *out_treebootserial, GError **error);

G_END_DECLS
