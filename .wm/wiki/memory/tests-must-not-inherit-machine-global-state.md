---
title: Tests must not inherit machine-global state
type: memory
tags: [testing, hermeticity, model]
status: active
---

A test suite whose result depends on the developer's $HOME is not a gate. Three failures came from the ONNX model in ~/.wm/models: slow CLI tests, a degradation test that passed only because another deleted the model, and a golden eval whose precondition disagreed with its assertion. A security test even deleted the real 134 MB model via wm_model remove. Fix: tools take the path as an explicit input, and tests construct the absence they assert. Full reference: @wiki/concepts/tests-must-not-inherit-machine-global-state