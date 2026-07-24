---
id: wiki:patterns:gitea-actions-release-api
title: Gitea Actions release without external GitHub actions
type: pattern
status: draft
relates_to:
  - {type: references, target: wiki:tasks:task-wm-reasonix-integration}
---
id: wiki:patterns:gitea-actions-release-api

## Problem

Gitea Actions runners often cannot reach GitHub's actions marketplace (no auth, no network access, self-hosted constraints). Yet many workflow examples depend on `actions/gitea-release@v1` or other GitHub-hosted actions.

## Solution

Use Gitea's built-in context variables and API directly via curl, bypassing any external action dependency:

```yaml
- name: Upload release asset
  run: |
    curl -s -X POST "${{ gitea.server_url }}/api/v1/repos/${{ gitea.repository }}/releases" \
      -H "Authorization: token ${{ gitea.token }}" \
      -H "Content-Type: application/json" \
      -d "{\"tag_name\": \"${{ gitea.ref_name }}\", \"name\": \"${{ gitea.ref_name }}\"}" \
      -o /tmp/release.json
    RELEASE_ID=$(grep -o '"id":[0-9]*' /tmp/release.json | head -1 | cut -d: -f2)
    curl -s -X POST "${{ gitea.server_url }}/api/v1/repos/${{ gitea.repository }}/releases/$RELEASE_ID/assets" \
      -H "Authorization: token ${{ gitea.token }}" \
      -F "attachment=@$BINARY" \
      -F "name=reasonix-orchestrate"
```

Key Gitea context variables:
- `${{ gitea.server_url }}` — the Gitea instance URL (e.g., https://gitea.gehenna.work)
- `${{ gitea.repository }}` — owner/repo (e.g., vpp/reasonix-config)
- `${{ gitea.ref_name }}` — tag or branch name (e.g., v0.1.0)
- `${{ gitea.token }}` — built-in runner auth token for API calls
- `${{ gitea.ref }}` — full ref path (e.g., refs/tags/v0.1.0)

## When to Use

- Self-hosted Gitea with no GitHub action marketplace access
- Any Gitea instance where minimizing external dependencies is preferred
- CI workflows that need release creation with asset upload

## When Not to Use

- If the runner has GitHub action marketplace access and actions/gitea-release@v1 works
- If complex release logic (GPG signing, multi-platform builds) is needed — consider a release.sh script instead

## Related
- @wiki/tasks/wm