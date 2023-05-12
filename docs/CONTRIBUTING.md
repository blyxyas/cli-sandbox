# Thanks for contributing to `cli-sandbox`!

**All contributions and welcomed!** From big changes and reviews, to fixing typos. Any person, from any level of experience is encouraged to contribute and open issues.

## Forking the repository

The first step to merge your change is to fork this repo.
You can do that from GitHub, clicking the "Fork" button near the "Star" button and entering some minor details.

Now, clone your fork:

```
git clone https://github.com/<Your username>/cli-sandbox
```

Or, using SSH

```
git clone git@github.com:<Your username>/cli-sandbox
```

Make sure to make a new branch, **do not make your changes in `main`**

```
git branch <your_change e.g. fix-this-feature>
```

---

## Making a change

Making a change is the easy part, do your change (probably on `lib.rs`) and make some tests for it.

## Testing your changes

Before merging your PR, we'll run a series of tests (CI) over it. It's very helpful that you've already tested your branch locally before opening a PR.

```sh
cargo build --features deny-warnings # Build the library
cargo test # Test your change 
cargo clippy --features deny-warnings # Run Clippy (Quality Assurance)
```

If all tests pass, you're ready to go, commit your changes and publish the branch!

## Opening a PR

Opening a PR is the most daunting part of the process, even more if it's your first.

1. Go to the [PR tab](https://github.com/blyxyas/cli-sandbox/pulls).
2. Click "New pull request"
3. Select the branch that you want to open a PR about.
4. Write a descriptive title
5. Write a descriptive body, with why is that a problem and what did you do to fix it.

If your PR fixes an issue, you can also add "Fixes #issue-number" to the body, and that issue will be closed if the PR is merged.

6. Click the button to create the new PR.
7. Wait until I review it; if it's all good, even merge it!

Congratulations, now part of `cli-sandbox`'s code is your creation!