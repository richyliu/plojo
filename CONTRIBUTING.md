## Git hooks

Run the following command to automatically run git hooks before commit:

```sh
git config core.hooksPath .githooks
```

Alternatively, you can manually run `./.githooks/pre-commit`.
