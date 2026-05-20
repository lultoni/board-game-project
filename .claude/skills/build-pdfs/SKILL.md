---
name: build-pdfs
description: "Compile all test scenario .typ files to PDFs. Run this after editing any file in docs/test-scenarios/."
disable-model-invocation: true
---

Run this command to build all PDFs:

```
docs/test-scenarios/build-pdfs.sh
```

That's it. The script discovers every standalone `.typ` file under `docs/test-scenarios/` (excluding `shared/` libraries) and compiles each to a PDF next to its source.
