Code Review
===========

## Code Review Practices

PR to `libtock-rs` can be divided into two categories:

1. **Upkeep pull requests** are self-contained changes that do not introduce
   significant code churn, and are unlikely to cause major merge conflicts.
1. **Significant pull requests** are pull requests that are too substantial to
   be considered upkeep pull requests. Significant pull requests may represent a
   significant change in `libtock-rs`'s design or include large refactorings
   that are likely to cause merge conflicts for other pull requests.

The owners of `libtock-rs` (listed [below](#owners)) determine whether a PR is
an upkeep PR or a significant PR. PRs should be merged by the `libtock-rs`
owners rather than the PR's author. In general, PRs should be merged using a
`bors r+` command rather than the GitHub UI (see the [bors
documentation](https://bors.tech/documentation/) for more information on bors).

A PR may only be merged when all of the following are true:

1. At least one `libtock-rs` owner (who is not the PR author) has approved the PR.
1. All outstanding review discussions have been resolved.
1. If the pull request is significant, a 7 day waiting period has passed since
   the PR was opened.

We recommend that authors of significant PRs comment on the PR when they believe
the above criteria have been satisfied (including the waiting period). This is
primarily to remind the owners to merge the PR. Secondarily, it should help
identify confusion about a PR review's status.

## Owners

The owners of `libtock-rs` are:

* The [Tock Core Working
  Group](https://github.com/tock/tock/tree/master/doc/wg/core#members).
* Alistair Francis, [alistair23](https://github.com/alistair23), Western Digital
