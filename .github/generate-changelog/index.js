/*
 * Usage (ref is a commit or tag):
 *
 * Full changelog: node generate-changelog.js
 * Changelog from ref 1 til now: node generate-changelog ref1
 * Changelog from ref 1 til ref2: node generate-changelog ref1 ref2
 * Print to file: node generate-changelog [args] > filename.md
 *
 * The range filter logic is from (excluding) to (including) - the inconsistency
 * is out of my control.
 */

const typeLabels = {
  fix: 'Bug fixes',
  revert: 'Reverted commits',
  refactor: 'Refactorings',
  config: 'Configuration',
  chore: 'Maintenance',
  feat: 'Features',
  docs: 'Documentation',
  perf: 'Performance improvements',
  test: 'Testing',
  ux: 'UX',
  build: 'Build'
};
// Aliases
typeLabels.deprecate = typeLabels.deprecation = typeLabels.rm;
typeLabels.ci = typeLabels.build;
typeLabels.feature = typeLabels.feat;
typeLabels.tests = typeLabels.test;

const git = require('simple-git/promise')(__dirname);
const groupBy = require('lodash/groupBy');
const {EOL} = require('os');

let from = process.argv[2];
let to = process.argv[3];

const messageRegex = /^([a-z]+)(\(([a-zA-Z0-9-_\s]+)\))?:\s*(.+)/;
const {closesRegexGlobal, closesRegexLocal} = (() => {
  const base = '#([0-9]+)';

  return {
    closesRegexGlobal: new RegExp(base, 'igm'),
    closesRegexLocal: new RegExp(base, 'i')
  }
})();

function mapMatch(m) {
  return `#${m.match(closesRegexLocal)[1]}`;
}

async function run() {
  if (from && !to) {
    // git.log() will ignore from if to is absent - default to latest commit
    to = 'HEAD';
  }

  // run git log command
  const listLogSummary = await git.log({from, to});
  if (!listLogSummary || !listLogSummary.total) {
    // No commits in the given range
    return;
  }

  const ungrouped = listLogSummary.all
    // filter + map
    .reduce(
      (acc, item) => {
        // Only consider commits matching the format
        const matched = item.message.match(messageRegex);
        if (matched) {
          let type = matched[1];
          // map type code, e.g. "fix" to a label, e.g. "Bug Fixes"
          type = typeLabels[type] || `[UNFORMATTED TYPE] ${type}`;
          const scope = matched[3];
          const msg = matched[4];
          const hash = item.hash;
          // Issues this commit closes
          let closes = [];

          // Find issue links in the commit body
          let closesMatches = item.body.match(closesRegexGlobal);
          if (closesMatches) {
            closes.push(...Array.from(closesMatches).map(mapMatch))
          }
          // and the header
          closesMatches = item.message.match(closesRegexGlobal);
          if (closesMatches) {
            closes.push(...Array.from(closesMatches).map(mapMatch));
          }
          // If we found any, dedupe just in case
          if (closes.length) {
            closes = Array.from(new Set(closes));
          }

          acc.push({type, scope, msg, hash, closes})
        }

        return acc;
      },
      []
    );

  /*
   * Commit sort:
   * - Commits with a valid type
   * - By commit type
   * - By commit scope
   * - By commit order
   */
  ungrouped.sort((a, b) => {
    const aUnformat = a.type.startsWith('[UNFORMATTED TYPE]');
    const bUnformat = b.type.startsWith('[UNFORMATTED TYPE]');
    if (aUnformat && !bUnformat) {
      return 1;
    } else if (bUnformat && !aUnformat) {
      return -1;
    } else if (a.type > b.type) {
      return 1;
    } else if (b.type > a.type) {
      return -1;
    } else if (a.scope && !b.scope) {
      return -1;
    } else if (b.scope && !a.scope) {
      return 1;
    } else {
      return (a.scope || '') > (b.scope || '');
    }
  });

  // Group commits by type
  const grouped = groupBy(ungrouped, 'type');
  const out = []; // each entry is a new line

  // Generate markdown
  for (const [type, commits] of Object.entries(grouped)) {
    out.push('', `### ${type}`, '');

    for (const {scope, msg, hash, closes} of commits) {
      let append = '- ';
      if (scope) {
        append += `**${scope}**: `
      }
      append += `${msg} \\[${hash}`;
      if (closes.length) {
        append += `, ${closes.join(', ')}`
      }
      append += ']';

      out.push(append);
    }
  }

  console.log(out.join(EOL))
}

run()
  .catch(e => {
    console.error(e);
    process.exit(1);
  });
