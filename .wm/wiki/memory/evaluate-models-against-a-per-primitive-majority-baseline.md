---
title: Evaluate models against a per-primitive majority baseline
type: memory
tags: [eval, baseline, uncertainty, model-selection]
status: active
---

Eval methodology: judge a candidate model against a PER-PRIMITIVE majority baseline (aggregates hide losses), use Wilson CIs (treat within-noise spreads as no effect), feed the model's trained channels (label_descriptions/examples), separate latency levers from accuracy levers, test the purpose-built model as an endpoint, persist per-config JSON. Full: @doc/patterns/eval-model-against-per-primitive-majority-baseline