#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
#
# setup-branch-protection.sh
#
# Sets PR-only branch protection for the default branch (usually main),
# restricts direct pushes to a maintainer team, and requires CI status checks.
#
# IMPORTANT:
# The required status check contexts MUST match the workflow/job names.
# This script assumes a workflow named "ci" with jobs: fmt, clippy, test.
# => contexts are: "ci / fmt", "ci / clippy", "ci / test"

set -euo pipefail

# ---------------------------
# Config (override via env)
# ---------------------------
ORG="${ORG:-XMV-Solutions-GmbH}"
REPO="${REPO:-talos-api-rs}"
BRANCH="${BRANCH:-main}"
TEAM_SLUG="${TEAM_SLUG:-open-source}"

# PR review rules
REQUIRED_APPROVALS="${REQUIRED_APPROVALS:-1}"
REQUIRE_CODEOWNER_REVIEWS="${REQUIRE_CODEOWNER_REVIEWS:-true}"
DISMISS_STALE_REVIEWS="${DISMISS_STALE_REVIEWS:-true}"

# Admin enforcement
ENFORCE_ADMINS="${ENFORCE_ADMINS:-true}"

# Required status checks (must match GitHub "checks" names exactly)
# Default: workflow "ci" + job names: fmt/clippy/test
STATUS_CHECKS_DEFAULT=("ci / fmt" "ci / clippy" "ci / test")
STATUS_CHECKS=("${STATUS_CHECKS_DEFAULT[@]}")

# Optional: allow certain actors to push to main besides team
EXTRA_USERS="${EXTRA_USERS:-}"   # e.g. "some-user,another-user"
EXTRA_APPS="${EXTRA_APPS:-}"     # e.g. "some-github-app"

FULL_REPO="${ORG}/${REPO}"

# ---------------------------
# Helpers
# ---------------------------
require_cmd() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "ERROR: Missing required command: $1" >&2
    exit 1
  }
}

split_csv_to_json_array() {
  local csv="${1:-}"
  if [[ -z "${csv}" ]]; then
    echo "[]"
    return
  fi
  local normalized
  normalized="$(echo "${csv}" | sed 's/[[:space:]]//g')"
  echo "${normalized}" | awk -F',' '{
    printf("[");
    for (i=1; i<=NF; i++) {
      printf("\"%s\"", $i);
      if (i<NF) printf(",");
    }
    printf("]");
  }'
}

json_array_from_bash_array() {
  # Print JSON array from current STATUS_CHECKS bash array
  printf '%s\n' "${STATUS_CHECKS[@]}" | jq -R . | jq -s .
}

# ---------------------------
# Preconditions
# ---------------------------
require_cmd gh
require_cmd jq

if ! gh auth status >/dev/null 2>&1; then
  echo "ERROR: gh is not authenticated. Run: gh auth login" >&2
  exit 1
fi

echo ">> Repo: ${FULL_REPO}"
gh repo view "${FULL_REPO}" >/dev/null

echo ">> Team: ${ORG}/${TEAM_SLUG}"
gh api "orgs/${ORG}/teams/${TEAM_SLUG}" >/dev/null

USERS_JSON="$(split_csv_to_json_array "${EXTRA_USERS}")"
APPS_JSON="$(split_csv_to_json_array "${EXTRA_APPS}")"
CHECKS_JSON="$(json_array_from_bash_array)"

echo ">> Required status checks:"
echo "${CHECKS_JSON}" | jq -r '.[]' | sed 's/^/   - /'

# ---------------------------
# Apply protection
# ---------------------------
echo ">> Applying branch protection to ${FULL_REPO}:${BRANCH}"

# Note: PUT replaces the configuration.
gh api -X PUT "repos/${FULL_REPO}/branches/${BRANCH}/protection" \
  -H "Accept: application/vnd.github+json" \
  -F "enforce_admins=${ENFORCE_ADMINS}" \
  -F "required_status_checks.strict=true" \
  -F "required_status_checks.contexts=$(echo "${CHECKS_JSON}" | jq -r '. | join(",")')" \
  -F "required_pull_request_reviews.required_approving_review_count=${REQUIRED_APPROVALS}" \
  -F "required_pull_request_reviews.dismiss_stale_reviews=${DISMISS_STALE_REVIEWS}" \
  -F "required_pull_request_reviews.require_code_owner_reviews=${REQUIRE_CODEOWNER_REVIEWS}" \
  -F "restrictions.teams[]=${TEAM_SLUG}" \
  $(jq -r '.[] | @sh' <<<"${USERS_JSON}" | sed -e "s/^/'restrictions.users[]=/;s/'$//" | xargs -I{} echo -F {}) \
  $(jq -r '.[] | @sh' <<<"${APPS_JSON}"  | sed -e "s/^/'restrictions.apps[]=/;s/'$//"  | xargs -I{} echo -F {}) \
  >/dev/null

echo ">> Requiring conversation resolution"
gh api -X PATCH "repos/${FULL_REPO}/branches/${BRANCH}/protection/required_conversation_resolution" \
  -H "Accept: application/vnd.github+json" \
  -F "enabled=true" \
  >/dev/null

echo ">> Disallowing force pushes"
gh api -X POST "repos/${FULL_REPO}/branches/${BRANCH}/protection/allow_force_pushes" \
  -H "Accept: application/vnd.github+json" \
  -F "enabled=false" \
  >/dev/null

echo ">> Disallowing branch deletions"
gh api -X POST "repos/${FULL_REPO}/branches/${BRANCH}/protection/allow_deletions" \
  -H "Accept: application/vnd.github+json" \
  -F "enabled=false" \
  >/dev/null

echo ">> Done."
echo ">> NOTE: If checks don't exist yet, GitHub will not be able to enforce them until the workflow runs at least once."