# Version bumps

Do not bump the package version in any of your commits. This operation happens automatically during releases and is
based on the commit message rules below.

# Commits & commit messages

Keep in mind that every commit message will show up as an entry in the changelog and controls the
[type of release](https://semver.org/) the next package version will be.

The commit message format is `type: Message`, e.g.
`fix: Fixed new database connections sometimes sending a declaration of war to a foreign nation`. An up-to-date list of
commit types can be found in the
[changelog action](https://github.com/Alorel/rust-indexed-db/blob/master/.github/actions/changelog/action.yml#L39).

In order for your commit to close an issue on merge and show up as the commit that resolved the issue in the changelog,
add a blank line followed by `Closes #issue_number` to your commit message, e.g.:

```
feat: Add support for inserting pandas into the DB

Closes #11111
```

In order to mark your commit as having breaking changes and result in a major version release, add a blank line
followed by `BREAKING CHANGE: Description of breaking changes`, e.g.:

```
rm: Remove support for inserting pandas into the DB.

BREAKING CHANGE: The panda union's opposition to the earlier feature was too great; Pandas may no longer be stored in the DB.
```
