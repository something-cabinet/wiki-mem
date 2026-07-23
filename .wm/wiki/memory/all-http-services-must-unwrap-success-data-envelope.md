---
title: All HTTP services must unwrap {success, data} envelope
type: memory
tags: [angular, http, api, consistency]
status: active
---

Inconsistent envelope handling between HttpEngineService and HttpCodeIntelService caused silent undefined data reads. Convention: extract {success, data}, throw on !success. Extract a shared httpCall helper. Full reference: @wiki/concepts/response-envelope-inconsistency