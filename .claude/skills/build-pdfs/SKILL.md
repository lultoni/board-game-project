---
name: build-pdfs
description: "Compile all test scenario .typ files to PDFs. Run this after editing any file in docs/test-scenarios/typ-files/."
disable-model-invocation: true
---

Run this command to build all PDFs:

```
docs/test-scenarios/build-pdfs.sh
```

That's it. The script compiles every `.typ` file in `docs/test-scenarios/typ-files/` to a PDF in `docs/test-scenarios/pdf-files/`.
