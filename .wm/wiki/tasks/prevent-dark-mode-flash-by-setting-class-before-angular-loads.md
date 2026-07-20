---
title: Prevent dark mode flash by setting class before Angular loads
type: task
status: todo
priority: high
---

Add inline `<script>` in index.html right after `<html>` tag to read localStorage and toggle `.dark` class before Angular loads, preventing white flash for dark mode users.