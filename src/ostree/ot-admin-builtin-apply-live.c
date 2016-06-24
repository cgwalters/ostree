/* -*- mode: C; c-file-style: "gnu"; indent-tabs-mode: nil; -*-
 *
 * Copyright (C) 2016 Colin Walters <walters@verbum.org>
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
 * License along with this library; if not, write to the
 * Free Software Foundation, Inc., 59 Temple Place - Suite 330,
 * Boston, MA 02111-1307, USA.
 */

#include "config.h"

#include "ot-main.h"
#include "ot-admin-builtins.h"
#include "ot-admin-functions.h"
#include "ostree.h"
#include "otutil.h"

#include "../libostree/ostree-kernel-args.h"

#include <glib/gi18n.h>
#include <err.h>

static GOptionEntry options[] = {
  { NULL }
};

gboolean
ot_admin_builtin_apply_live (int argc, char **argv, GCancellable *cancellable, GError **error)
{
  gboolean ret = FALSE;
  GOptionContext *context;
  glnx_unref_object OstreeSysroot *sysroot = NULL;
  glnx_unref_object OstreeRepo *repo = NULL;
  g_autoptr(GPtrArray) current_deployments = NULL;
  OstreeDeployment *booted_deployment = NULL;

  context = g_option_context_new ("Apply all filesystem changes from new deployment to current one");

  if (!ostree_admin_option_context_parse (context, options, &argc, &argv,
                                          OSTREE_ADMIN_BUILTIN_FLAG_SUPERUSER,
                                          &sysroot, cancellable, error))
    goto out;
  
  if (argc > 1)
    {
      ot_util_usage_error (context, "This command takes no extra arguments", error);
      goto out;
    }

  if (!ostree_sysroot_load (sysroot, cancellable, error))
    goto out;
  current_deployments = ostree_sysroot_get_deployments (sysroot);

  booted_deployment = ostree_sysroot_get_booted_deployment (sysroot);
  if (!booted_deployment)
    {
      g_set_error_literal (error, G_IO_ERROR, G_IO_ERROR_FAILED,
                           "Not currently booted into an OSTree system");
      goto out;
    }

  
  if (current_deployments->len < 2)
    {
      g_set_error_literal (error, G_IO_ERROR, G_IO_ERROR_FAILED,
                           "No new deployment to use as source for live update");
      goto out;
    }
  else
    {
      OstreeDeployment *src_deployment = current_deployments->pdata[0];
      if (src_deployment == booted_deployment)
        {
          g_set_error_literal (error, G_IO_ERROR, G_IO_ERROR_FAILED,
                               "No new deployment to use as source for live update");
          goto out;
        }
    }

  if (!ostree_sysroot_deployment_live_replace (sysroot, current_deployments->pdata[0],
                                               booted_deployment,
                                               cancellable, error))
    goto out;

  ret = TRUE;
 out:
  if (context)
    g_option_context_free (context);
  return ret;
}
