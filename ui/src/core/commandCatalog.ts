import { z } from "zod";

import { ALL_COMMAND_IDS, STABLE_COMMAND_IDS, type CommandId } from "./commandIds";
import type { CommandCategory, CommandField, CommandPermission, CommandSpec } from "./types";

const ADMIN_COMMANDS = new Set<CommandId>([
  "repo.edit",
  "repo.delete",
  "release.delete",
  "settings.branch_protection.get",
  "settings.branch_protection.update",
  "settings.collaborators.add",
  "settings.collaborators.list",
  "settings.collaborators.remove",
  "settings.dependabot_alerts.list",
  "settings.deploy_keys.add",
  "settings.deploy_keys.delete",
  "settings.deploy_keys.list",
  "settings.secrets.delete",
  "settings.secrets.list",
  "settings.secrets.set",
  "settings.variables.delete",
  "settings.variables.list",
  "settings.variables.set",
  "settings.webhooks.create",
  "settings.webhooks.delete",
  "settings.webhooks.list",
  "settings.webhooks.ping",
  "wiki.update",
  "pages.create",
  "pages.update",
  "pages.delete",
  "rulesets.create",
  "rulesets.update",
  "rulesets.delete",
  "rulesets.get",
  "rulesets.list",
]);

const WRITE_COMMANDS = new Set<CommandId>([
  "repo.create",
  "repo.branch.create",
  "repo.branch.delete",
  "pr.close",
  "pr.comments.create",
  "pr.create",
  "pr.edit",
  "pr.merge",
  "pr.reopen",
  "pr.review",
  "pr.review_comments.create",
  "pr.review_comments.reply",
  "pr.review_threads.resolve",
  "pr.review_threads.unresolve",
  "issue.close",
  "issue.comment",
  "issue.create",
  "issue.edit",
  "issue.reopen",
  "run.cancel",
  "run.rerun",
  "release.asset.delete",
  "release.asset.upload",
  "release.create",
  "release.edit",
  "projects.items.add",
  "discussions.answer",
  "discussions.close",
  "discussions.create",
]);

const DESTRUCTIVE_COMMANDS = new Set<CommandId>([
  "repo.delete",
  "repo.branch.delete",
  "release.delete",
  "release.asset.delete",
  "settings.collaborators.remove",
  "settings.secrets.delete",
  "settings.variables.delete",
  "settings.webhooks.delete",
  "settings.deploy_keys.delete",
  "pages.delete",
  "rulesets.delete",
]);

const CONSOLE_ONLY_COMMANDS = new Set<CommandId>(["repo.topics.replace", "repo.branch.ref.get"]);

const REPO_SCOPE_FIELDS: CommandField[] = [
  { name: "owner", label: "owner", type: "text", required: true, placeholder: "octocat" },
  { name: "repo", label: "repo", type: "text", required: true, placeholder: "hello-world" },
];

const REPO_LIMIT_FIELDS: CommandField[] = [
  ...REPO_SCOPE_FIELDS,
  { name: "limit", label: "limit", type: "number", min: 1, placeholder: "20" },
];

const PR_NUMBER_FIELDS: CommandField[] = [
  ...REPO_SCOPE_FIELDS,
  { name: "number", label: "number", type: "number", required: true, min: 1 },
];

const RUN_ID_FIELDS: CommandField[] = [
  ...REPO_SCOPE_FIELDS,
  { name: "run_id", label: "run_id", type: "number", required: true, min: 1 },
];

const payloadAny = z.record(z.any());
const responseAny = z.any();

const payloadSchemas: Partial<Record<CommandId, z.ZodTypeAny>> = {
  "auth.organizations.list": z.object({}).passthrough(),
  "auth.status": z.object({}).passthrough(),
  "repo.list": z
    .object({ owner: z.string().min(1), limit: z.number().int().positive().optional() })
    .passthrough(),
  "repo.create": z
    .object({
      owner: z.string().min(1),
      name: z.string().min(1),
      private: z.boolean().optional(),
      description: z.string().optional(),
    })
    .passthrough(),
  "repo.edit": z
    .object({
      owner: z.string().min(1),
      repo: z.string().min(1),
      description: z.string().optional(),
      homepage: z.string().optional(),
      default_branch: z.string().optional(),
      visibility: z.string().optional(),
      add_topics: z.array(z.string()).optional(),
      remove_topics: z.array(z.string()).optional(),
      replace_topics: z.array(z.string()).optional(),
    })
    .passthrough(),
  "repo.topics.replace": z.object({ args: z.array(z.string()).default([]) }).passthrough(),
  "repo.branches.list": z
    .object({ owner: z.string().min(1), repo: z.string().min(1), limit: z.number().optional() })
    .passthrough(),
  "repo.branch.ref.get": z.object({ args: z.array(z.string()).default([]) }).passthrough(),
  "repo.branch.create": z
    .object({
      owner: z.string().min(1),
      repo: z.string().min(1),
      branch: z.string().min(1),
      from_branch: z.string().min(1),
    })
    .passthrough(),
  "repo.branch.delete": z
    .object({ owner: z.string().min(1), repo: z.string().min(1), branch: z.string().min(1) })
    .passthrough(),
  "repo.commits.list": z
    .object({
      owner: z.string().min(1),
      repo: z.string().min(1),
      branch: z.string().optional(),
      limit: z.number().optional(),
    })
    .passthrough(),
  "repo.delete": z.object({ owner: z.string().min(1), repo: z.string().min(1) }).passthrough(),
  "pr.list": z
    .object({ owner: z.string().min(1), repo: z.string().min(1), limit: z.number().optional() })
    .passthrough(),
  "pr.view": z
    .object({ owner: z.string().min(1), repo: z.string().min(1), number: z.number().positive() })
    .passthrough(),
  "pr.create": z
    .object({
      owner: z.string().min(1),
      repo: z.string().min(1),
      title: z.string().min(1),
      head: z.string().min(1),
      base: z.string().min(1),
      body: z.string().optional(),
      draft: z.boolean().optional(),
    })
    .passthrough(),
  "pr.review": z
    .object({
      owner: z.string().min(1),
      repo: z.string().min(1),
      number: z.number().positive(),
      event: z.string().min(1),
      body: z.string().optional(),
    })
    .passthrough(),
  "pr.edit": z
    .object({
      owner: z.string().min(1),
      repo: z.string().min(1),
      number: z.number().positive(),
      title: z.string().optional(),
      body: z.string().optional(),
      base: z.string().optional(),
    })
    .passthrough(),
  "pr.close": z
    .object({
      owner: z.string().min(1),
      repo: z.string().min(1),
      number: z.number().positive(),
      comment: z.string().optional(),
      delete_branch: z.boolean().optional(),
    })
    .passthrough(),
  "pr.reopen": z
    .object({
      owner: z.string().min(1),
      repo: z.string().min(1),
      number: z.number().positive(),
      comment: z.string().optional(),
    })
    .passthrough(),
  "pr.merge": z
    .object({
      owner: z.string().min(1),
      repo: z.string().min(1),
      number: z.number().positive(),
      method: z.string().optional(),
      delete_branch: z.boolean().optional(),
      auto: z.boolean().optional(),
    })
    .passthrough(),
  "pr.comments.list": z
    .object({ owner: z.string().min(1), repo: z.string().min(1), number: z.number().positive() })
    .passthrough(),
  "pr.comments.create": z
    .object({
      owner: z.string().min(1),
      repo: z.string().min(1),
      number: z.number().positive(),
      body: z.string().min(1),
    })
    .passthrough(),
  "pr.review_comments.list": z
    .object({ owner: z.string().min(1), repo: z.string().min(1), number: z.number().positive() })
    .passthrough(),
  "pr.review_comments.create": z
    .object({
      owner: z.string().min(1),
      repo: z.string().min(1),
      number: z.number().positive(),
      commit_id: z.string().min(1),
      path: z.string().min(1),
      line: z.number().positive(),
      body: z.string().min(1),
      side: z.string().optional(),
    })
    .passthrough(),
  "pr.review_comments.reply": z
    .object({
      owner: z.string().min(1),
      repo: z.string().min(1),
      number: z.number().positive(),
      comment_id: z.number().positive(),
      body: z.string().min(1),
    })
    .passthrough(),
  "pr.review_threads.list": z
    .object({ owner: z.string().min(1), repo: z.string().min(1), number: z.number().positive() })
    .passthrough(),
  "pr.review_threads.resolve": z.object({ thread_id: z.string().min(1) }).passthrough(),
  "pr.review_threads.unresolve": z.object({ thread_id: z.string().min(1) }).passthrough(),
  "pr.diff.files.list": z
    .object({ owner: z.string().min(1), repo: z.string().min(1), number: z.number().positive() })
    .passthrough(),
  "pr.diff.raw.get": z
    .object({ owner: z.string().min(1), repo: z.string().min(1), number: z.number().positive() })
    .passthrough(),
  "issue.list": z
    .object({ owner: z.string().min(1), repo: z.string().min(1), limit: z.number().optional() })
    .passthrough(),
  "issue.view": z
    .object({ owner: z.string().min(1), repo: z.string().min(1), number: z.number().positive() })
    .passthrough(),
  "issue.create": z
    .object({
      owner: z.string().min(1),
      repo: z.string().min(1),
      title: z.string().min(1),
      body: z.string().optional(),
    })
    .passthrough(),
  "issue.comment": z
    .object({
      owner: z.string().min(1),
      repo: z.string().min(1),
      number: z.number().positive(),
      body: z.string().min(1),
    })
    .passthrough(),
  "issue.edit": z
    .object({
      owner: z.string().min(1),
      repo: z.string().min(1),
      number: z.number().positive(),
      title: z.string().optional(),
      body: z.string().optional(),
      add_assignees: z.array(z.string()).optional(),
      remove_assignees: z.array(z.string()).optional(),
      add_labels: z.array(z.string()).optional(),
      remove_labels: z.array(z.string()).optional(),
    })
    .passthrough(),
  "issue.close": z
    .object({
      owner: z.string().min(1),
      repo: z.string().min(1),
      number: z.number().positive(),
      comment: z.string().optional(),
      reason: z.string().optional(),
    })
    .passthrough(),
  "issue.reopen": z
    .object({
      owner: z.string().min(1),
      repo: z.string().min(1),
      number: z.number().positive(),
      comment: z.string().optional(),
    })
    .passthrough(),
  "workflow.list": z
    .object({ owner: z.string().min(1), repo: z.string().min(1), limit: z.number().optional() })
    .passthrough(),
  "run.list": z
    .object({ owner: z.string().min(1), repo: z.string().min(1), limit: z.number().optional() })
    .passthrough(),
  "run.rerun": z
    .object({
      owner: z.string().min(1),
      repo: z.string().min(1),
      run_id: z.number().positive(),
      failed_only: z.boolean().optional(),
    })
    .passthrough(),
  "run.view": z.object({ owner: z.string().min(1), repo: z.string().min(1), run_id: z.number().positive() }).passthrough(),
  "run.logs": z.object({ owner: z.string().min(1), repo: z.string().min(1), run_id: z.number().positive() }).passthrough(),
  "run.cancel": z.object({ owner: z.string().min(1), repo: z.string().min(1), run_id: z.number().positive() }).passthrough(),
  "release.list": z
    .object({ owner: z.string().min(1), repo: z.string().min(1), limit: z.number().optional() })
    .passthrough(),
  "release.create": z
    .object({
      owner: z.string().min(1),
      repo: z.string().min(1),
      tag: z.string().min(1),
      title: z.string().optional(),
      notes: z.string().optional(),
      draft: z.boolean().optional(),
      prerelease: z.boolean().optional(),
      target: z.string().optional(),
    })
    .passthrough(),
  "release.edit": z
    .object({
      owner: z.string().min(1),
      repo: z.string().min(1),
      tag: z.string().min(1),
      title: z.string().optional(),
      notes: z.string().optional(),
      draft: z.boolean().optional(),
      prerelease: z.boolean().optional(),
    })
    .passthrough(),
  "release.asset.upload": z
    .object({
      owner: z.string().min(1),
      repo: z.string().min(1),
      tag: z.string().min(1),
      file_path: z.string().min(1),
      clobber: z.boolean().optional(),
    })
    .passthrough(),
  "release.asset.delete": z
    .object({
      owner: z.string().min(1),
      repo: z.string().min(1),
      tag: z.string().min(1),
      asset_name: z.string().min(1),
    })
    .passthrough(),
  "release.delete": z
    .object({
      owner: z.string().min(1),
      repo: z.string().min(1),
      tag: z.string().min(1),
      cleanup_tag: z.boolean().optional(),
    })
    .passthrough(),
};

const responseSchemas: Partial<Record<CommandId, z.ZodTypeAny>> = {
  "auth.status": z
    .object({
      logged_in: z.boolean(),
      account: z.string().nullable().optional(),
      scopes: z.array(z.string()).optional(),
      host: z.string().nullable().optional(),
    })
    .passthrough(),
  "repo.list": z.array(
    z
      .object({
        name: z.string(),
        nameWithOwner: z.string(),
        viewerPermission: z.string().optional(),
      })
      .passthrough(),
  ),
};

const fieldOverrides: Partial<Record<CommandId, CommandField[]>> = {
  "auth.organizations.list": [],
  "auth.status": [],
  "repo.list": [
    { name: "owner", label: "owner", type: "text", required: true, placeholder: "octocat" },
    { name: "limit", label: "limit", type: "number", min: 1, placeholder: "20" },
  ],
  "repo.create": [
    { name: "owner", label: "owner", type: "text", required: true },
    { name: "name", label: "name", type: "text", required: true },
    { name: "private", label: "private", type: "boolean" },
    { name: "description", label: "description", type: "textarea" },
  ],
  "repo.edit": [
    ...REPO_SCOPE_FIELDS,
    { name: "description", label: "description", type: "textarea" },
    { name: "homepage", label: "homepage", type: "text" },
    { name: "default_branch", label: "default_branch", type: "text" },
    {
      name: "visibility",
      label: "visibility",
      type: "select",
      options: [
        { label: "public", value: "public" },
        { label: "private", value: "private" },
        { label: "internal", value: "internal" },
      ],
    },
    { name: "add_topics", label: "add_topics", type: "string_list" },
    { name: "remove_topics", label: "remove_topics", type: "string_list" },
    { name: "replace_topics", label: "replace_topics", type: "string_list" },
  ],
  "repo.topics.replace": [{ name: "args", label: "args", type: "string_list", required: true }],
  "repo.branches.list": REPO_LIMIT_FIELDS,
  "repo.branch.ref.get": [{ name: "args", label: "args", type: "string_list", required: true }],
  "repo.branch.create": [
    ...REPO_SCOPE_FIELDS,
    { name: "branch", label: "branch", type: "text", required: true },
    { name: "from_branch", label: "from_branch", type: "text", required: true },
  ],
  "repo.branch.delete": [...REPO_SCOPE_FIELDS, { name: "branch", label: "branch", type: "text", required: true }],
  "repo.commits.list": [...REPO_LIMIT_FIELDS, { name: "branch", label: "branch", type: "text" }],
  "repo.delete": REPO_SCOPE_FIELDS,

  "pr.list": REPO_LIMIT_FIELDS,
  "pr.view": PR_NUMBER_FIELDS,
  "pr.create": [
    ...REPO_SCOPE_FIELDS,
    { name: "title", label: "title", type: "text", required: true },
    { name: "head", label: "head", type: "text", required: true, placeholder: "feature/my-branch" },
    { name: "base", label: "base", type: "text", required: true, placeholder: "main" },
    { name: "body", label: "body", type: "textarea" },
    { name: "draft", label: "draft", type: "boolean" },
  ],
  "pr.review": [
    ...PR_NUMBER_FIELDS,
    {
      name: "event",
      label: "event",
      type: "select",
      required: true,
      options: [
        { label: "approve", value: "approve" },
        { label: "request_changes", value: "request_changes" },
        { label: "comment", value: "comment" },
      ],
    },
    { name: "body", label: "body", type: "textarea" },
  ],
  "pr.edit": [
    ...PR_NUMBER_FIELDS,
    { name: "title", label: "title", type: "text" },
    { name: "body", label: "body", type: "textarea" },
    { name: "base", label: "base", type: "text" },
  ],
  "pr.close": [
    ...PR_NUMBER_FIELDS,
    { name: "comment", label: "comment", type: "textarea" },
    { name: "delete_branch", label: "delete_branch", type: "boolean" },
  ],
  "pr.reopen": [...PR_NUMBER_FIELDS, { name: "comment", label: "comment", type: "textarea" }],
  "pr.merge": [
    ...PR_NUMBER_FIELDS,
    {
      name: "method",
      label: "method",
      type: "select",
      options: [
        { label: "merge", value: "merge" },
        { label: "squash", value: "squash" },
        { label: "rebase", value: "rebase" },
      ],
    },
    { name: "delete_branch", label: "delete_branch", type: "boolean" },
    { name: "auto", label: "auto", type: "boolean" },
  ],
  "pr.comments.list": PR_NUMBER_FIELDS,
  "pr.comments.create": [...PR_NUMBER_FIELDS, { name: "body", label: "body", type: "textarea", required: true }],
  "pr.review_comments.list": PR_NUMBER_FIELDS,
  "pr.review_comments.create": [
    ...PR_NUMBER_FIELDS,
    { name: "commit_id", label: "commit_id", type: "text", required: true },
    { name: "path", label: "path", type: "text", required: true },
    { name: "line", label: "line", type: "number", required: true, min: 1 },
    { name: "side", label: "side", type: "select", options: [{ label: "RIGHT", value: "RIGHT" }, { label: "LEFT", value: "LEFT" }] },
    { name: "body", label: "body", type: "textarea", required: true },
  ],
  "pr.review_comments.reply": [
    ...PR_NUMBER_FIELDS,
    { name: "comment_id", label: "comment_id", type: "number", required: true, min: 1 },
    { name: "body", label: "body", type: "textarea", required: true },
  ],
  "pr.review_threads.list": PR_NUMBER_FIELDS,
  "pr.review_threads.resolve": [{ name: "thread_id", label: "thread_id", type: "text", required: true }],
  "pr.review_threads.unresolve": [{ name: "thread_id", label: "thread_id", type: "text", required: true }],
  "pr.diff.files.list": PR_NUMBER_FIELDS,
  "pr.diff.raw.get": PR_NUMBER_FIELDS,

  "issue.list": REPO_LIMIT_FIELDS,
  "issue.view": PR_NUMBER_FIELDS,
  "issue.create": [...REPO_SCOPE_FIELDS, { name: "title", label: "title", type: "text", required: true }, { name: "body", label: "body", type: "textarea" }],
  "issue.comment": [...PR_NUMBER_FIELDS, { name: "body", label: "body", type: "textarea", required: true }],
  "issue.edit": [
    ...PR_NUMBER_FIELDS,
    { name: "title", label: "title", type: "text" },
    { name: "body", label: "body", type: "textarea" },
    { name: "add_assignees", label: "add_assignees", type: "string_list" },
    { name: "remove_assignees", label: "remove_assignees", type: "string_list" },
    { name: "add_labels", label: "add_labels", type: "string_list" },
    { name: "remove_labels", label: "remove_labels", type: "string_list" },
  ],
  "issue.close": [
    ...PR_NUMBER_FIELDS,
    { name: "comment", label: "comment", type: "textarea" },
    {
      name: "reason",
      label: "reason",
      type: "select",
      options: [
        { label: "completed", value: "completed" },
        { label: "not planned", value: "not planned" },
      ],
    },
  ],
  "issue.reopen": [...PR_NUMBER_FIELDS, { name: "comment", label: "comment", type: "textarea" }],

  "workflow.list": REPO_LIMIT_FIELDS,
  "run.list": REPO_LIMIT_FIELDS,
  "run.rerun": [...RUN_ID_FIELDS, { name: "failed_only", label: "failed_only", type: "boolean" }],
  "run.view": RUN_ID_FIELDS,
  "run.logs": RUN_ID_FIELDS,
  "run.cancel": RUN_ID_FIELDS,

  "release.list": REPO_LIMIT_FIELDS,
  "release.create": [
    ...REPO_SCOPE_FIELDS,
    { name: "tag", label: "tag", type: "text", required: true },
    { name: "title", label: "title", type: "text" },
    { name: "notes", label: "notes", type: "textarea" },
    { name: "draft", label: "draft", type: "boolean" },
    { name: "prerelease", label: "prerelease", type: "boolean" },
    { name: "target", label: "target", type: "text" },
  ],
  "release.edit": [
    ...REPO_SCOPE_FIELDS,
    { name: "tag", label: "tag", type: "text", required: true },
    { name: "title", label: "title", type: "text" },
    { name: "notes", label: "notes", type: "textarea" },
    { name: "draft", label: "draft", type: "boolean" },
    { name: "prerelease", label: "prerelease", type: "boolean" },
  ],
  "release.asset.upload": [
    ...REPO_SCOPE_FIELDS,
    { name: "tag", label: "tag", type: "text", required: true },
    { name: "file_path", label: "file_path", type: "text", required: true },
    { name: "clobber", label: "clobber", type: "boolean" },
  ],
  "release.asset.delete": [
    ...REPO_SCOPE_FIELDS,
    { name: "tag", label: "tag", type: "text", required: true },
    { name: "asset_name", label: "asset_name", type: "text", required: true },
  ],
  "release.delete": [...REPO_SCOPE_FIELDS, { name: "tag", label: "tag", type: "text", required: true }, { name: "cleanup_tag", label: "cleanup_tag", type: "boolean" }],

  "settings.collaborators.list": REPO_SCOPE_FIELDS,
  "settings.collaborators.add": [
    ...REPO_SCOPE_FIELDS,
    { name: "username", label: "username", type: "text", required: true },
    {
      name: "permission",
      label: "permission",
      type: "select",
      required: true,
      options: [
        { label: "pull", value: "pull" },
        { label: "push", value: "push" },
        { label: "admin", value: "admin" },
        { label: "maintain", value: "maintain" },
        { label: "triage", value: "triage" },
      ],
    },
  ],
  "settings.collaborators.remove": [...REPO_SCOPE_FIELDS, { name: "username", label: "username", type: "text", required: true }],
  "settings.secrets.list": [
    ...REPO_SCOPE_FIELDS,
    {
      name: "app",
      label: "app",
      type: "select",
      options: [
        { label: "actions", value: "actions" },
        { label: "dependabot", value: "dependabot" },
        { label: "codespaces", value: "codespaces" },
      ],
    },
  ],
  "settings.secrets.set": [
    ...REPO_SCOPE_FIELDS,
    { name: "name", label: "name", type: "text", required: true },
    { name: "value", label: "value", type: "textarea", required: true },
    {
      name: "app",
      label: "app",
      type: "select",
      options: [
        { label: "actions", value: "actions" },
        { label: "dependabot", value: "dependabot" },
        { label: "codespaces", value: "codespaces" },
      ],
    },
  ],
  "settings.secrets.delete": [
    ...REPO_SCOPE_FIELDS,
    { name: "name", label: "name", type: "text", required: true },
    {
      name: "app",
      label: "app",
      type: "select",
      options: [
        { label: "actions", value: "actions" },
        { label: "dependabot", value: "dependabot" },
        { label: "codespaces", value: "codespaces" },
      ],
    },
  ],
  "settings.variables.list": REPO_SCOPE_FIELDS,
  "settings.variables.set": [
    ...REPO_SCOPE_FIELDS,
    { name: "name", label: "name", type: "text", required: true },
    { name: "value", label: "value", type: "textarea", required: true },
  ],
  "settings.variables.delete": [...REPO_SCOPE_FIELDS, { name: "name", label: "name", type: "text", required: true }],
  "settings.webhooks.list": REPO_SCOPE_FIELDS,
  "settings.webhooks.create": [
    ...REPO_SCOPE_FIELDS,
    { name: "target_url", label: "target_url", type: "text", required: true },
    { name: "events", label: "events", type: "string_list" },
    { name: "active", label: "active", type: "boolean" },
    {
      name: "content_type",
      label: "content_type",
      type: "select",
      options: [
        { label: "json", value: "json" },
        { label: "form", value: "form" },
      ],
    },
    { name: "secret", label: "secret", type: "text" },
  ],
  "settings.webhooks.ping": [...REPO_SCOPE_FIELDS, { name: "hook_id", label: "hook_id", type: "number", required: true, min: 1 }],
  "settings.webhooks.delete": [...REPO_SCOPE_FIELDS, { name: "hook_id", label: "hook_id", type: "number", required: true, min: 1 }],
  "settings.branch_protection.get": [...REPO_SCOPE_FIELDS, { name: "branch", label: "branch", type: "text", required: true }],
  "settings.branch_protection.update": [
    ...REPO_SCOPE_FIELDS,
    { name: "branch", label: "branch", type: "text", required: true },
    { name: "enforce_admins", label: "enforce_admins", type: "boolean" },
    { name: "dismiss_stale_reviews", label: "dismiss_stale_reviews", type: "boolean" },
    { name: "require_code_owner_reviews", label: "require_code_owner_reviews", type: "boolean" },
    { name: "required_approving_review_count", label: "required_approving_review_count", type: "number", min: 0 },
  ],
  "settings.deploy_keys.list": REPO_SCOPE_FIELDS,
  "settings.deploy_keys.add": [
    ...REPO_SCOPE_FIELDS,
    { name: "title", label: "title", type: "text", required: true },
    { name: "key", label: "key", type: "textarea", required: true },
    { name: "read_only", label: "read_only", type: "boolean" },
  ],
  "settings.deploy_keys.delete": [...REPO_SCOPE_FIELDS, { name: "key_id", label: "key_id", type: "number", required: true, min: 1 }],
  "settings.dependabot_alerts.list": REPO_LIMIT_FIELDS,

  "insights.views.get": REPO_SCOPE_FIELDS,
  "insights.clones.get": REPO_SCOPE_FIELDS,

  "projects.list": REPO_LIMIT_FIELDS,
  "projects.items.list": [...REPO_SCOPE_FIELDS, { name: "project_number", label: "project_number", type: "number", required: true, min: 1 }, { name: "limit", label: "limit", type: "number", min: 1 }],
  "projects.items.add": [
    { name: "project_id", label: "project_id", type: "text", required: true },
    { name: "content_id", label: "content_id", type: "text", required: true },
  ],

  "discussions.categories.list": REPO_LIMIT_FIELDS,
  "discussions.list": REPO_LIMIT_FIELDS,
  "discussions.create": [
    ...REPO_SCOPE_FIELDS,
    { name: "category_slug", label: "category_slug", type: "text", required: true },
    { name: "title", label: "title", type: "text", required: true },
    { name: "body", label: "body", type: "textarea", required: true },
  ],
  "discussions.close": [{ name: "discussion_id", label: "discussion_id", type: "text", required: true }],
  "discussions.answer": [{ name: "comment_id", label: "comment_id", type: "text", required: true }],

  "wiki.get": REPO_SCOPE_FIELDS,
  "wiki.update": [...REPO_SCOPE_FIELDS, { name: "enabled", label: "enabled", type: "boolean", required: true }],

  "pages.get": REPO_SCOPE_FIELDS,
  "pages.create": [
    ...REPO_SCOPE_FIELDS,
    { name: "branch", label: "branch", type: "text", required: true },
    { name: "path", label: "path", type: "text", required: true },
    { name: "build_type", label: "build_type", type: "text" },
    { name: "cname", label: "cname", type: "text" },
  ],
  "pages.update": [
    ...REPO_SCOPE_FIELDS,
    { name: "branch", label: "branch", type: "text", required: true },
    { name: "path", label: "path", type: "text", required: true },
    { name: "build_type", label: "build_type", type: "text" },
    { name: "cname", label: "cname", type: "text" },
  ],
  "pages.delete": REPO_SCOPE_FIELDS,

  "rulesets.list": REPO_SCOPE_FIELDS,
  "rulesets.get": [...REPO_SCOPE_FIELDS, { name: "ruleset_id", label: "ruleset_id", type: "number", required: true, min: 1 }],
  "rulesets.create": [...REPO_SCOPE_FIELDS, { name: "fields", label: "fields", type: "json", required: true, placeholder: '[{"key":"name","value":"value"}]' }],
  "rulesets.update": [
    ...REPO_SCOPE_FIELDS,
    { name: "ruleset_id", label: "ruleset_id", type: "number", required: true, min: 1 },
    { name: "fields", label: "fields", type: "json", required: true, placeholder: '[{"key":"name","value":"value"}]' },
  ],
  "rulesets.delete": [...REPO_SCOPE_FIELDS, { name: "ruleset_id", label: "ruleset_id", type: "number", required: true, min: 1 }],
};

function inferCategory(id: CommandId): CommandCategory {
  if (
    id === "auth.organizations.list" ||
    id === "auth.status" ||
    id === "repo.list"
  ) {
    return "dashboard";
  }

  if (id.startsWith("repo.")) {
    return "repositories";
  }

  if (id.startsWith("pr.")) {
    return "pull_requests";
  }

  if (id.startsWith("issue.")) {
    return "issues";
  }

  if (id.startsWith("workflow.") || id.startsWith("run.")) {
    return "actions";
  }

  if (id.startsWith("release.")) {
    return "releases";
  }

  if (id.startsWith("settings.")) {
    return "settings";
  }

  if (
    id.startsWith("projects.") ||
    id.startsWith("discussions.") ||
    id.startsWith("wiki.") ||
    id.startsWith("pages.") ||
    id.startsWith("rulesets.") ||
    id.startsWith("insights.")
  ) {
    return "p2";
  }

  return "console";
}

function inferPermission(id: CommandId): CommandPermission {
  if (ADMIN_COMMANDS.has(id)) {
    return "admin";
  }

  if (WRITE_COMMANDS.has(id)) {
    return "write";
  }

  return "viewer";
}

function inferNeedsRepoContext(id: CommandId): boolean {
  if (id === "auth.organizations.list" || id === "auth.status") {
    return false;
  }

  if (id === "repo.list" || id === "repo.create") {
    return false;
  }

  if (id === "projects.items.add" || id === "discussions.close" || id === "discussions.answer") {
    return false;
  }

  return true;
}

function inferTitle(id: CommandId): string {
  return id;
}

function inferDescription(id: CommandId): string {
  return `Execute ${id} through frontend envelope contract`;
}

function buildSpec(id: CommandId): CommandSpec {
  return {
    id,
    title: inferTitle(id),
    description: inferDescription(id),
    category: inferCategory(id),
    requiredPermission: inferPermission(id),
    exposure: CONSOLE_ONLY_COMMANDS.has(id) ? "console" : "screen",
    destructive: DESTRUCTIVE_COMMANDS.has(id),
    needsRepoContext: inferNeedsRepoContext(id),
    payloadSchema: payloadSchemas[id] ?? payloadAny,
    responseSchema: responseSchemas[id] ?? responseAny,
    fields: fieldOverrides[id] ?? [],
  };
}

export const COMMAND_CATALOG: Record<CommandId, CommandSpec> = ALL_COMMAND_IDS.reduce(
  (acc, id) => {
    acc[id] = buildSpec(id);
    return acc;
  },
  {} as Record<CommandId, CommandSpec>,
);

export function listCommandsByCategory(category: CommandCategory): CommandSpec[] {
  return STABLE_COMMAND_IDS.map((id) => COMMAND_CATALOG[id]).filter(
    (spec) => spec.category === category || (category === "dashboard" && spec.id === "repo.list"),
  );
}

export function listConsoleCommands(): CommandSpec[] {
  return STABLE_COMMAND_IDS.map((id) => COMMAND_CATALOG[id]);
}

export function isDestructiveCommand(id: CommandId): boolean {
  return COMMAND_CATALOG[id].destructive;
}
