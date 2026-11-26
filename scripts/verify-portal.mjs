const base = process.argv[2] || 'https://zzdevxx.github.io/kinglight-download/';

async function get(path) {
  const url = new URL(path, base).toString();
  const response = await fetch(url);
  const text = await response.text();
  return { url, status: response.status, text };
}

const checks = [];
const pages = [
  ['./', 'status rilis'],
  ['./privacy.html', 'Privasi'],
  ['./terms.html', 'Ketentuan'],
  ['./data-deletion.html', 'Hapus data'],
  ['./release.json', 'schemaVersion'],
];

for (const [path, marker] of pages) {
  const result = await get(path);
  checks.push({
    path,
    status: result.status,
    ok: result.status === 200 && result.text.toLowerCase().includes(marker.toLowerCase()),
  });
}

const snapshotResponse = await get('./release.json');
let snapshot = null;
try {
  snapshot = JSON.parse(snapshotResponse.text);
} catch {
  snapshot = null;
}
checks.push({
  path: 'release.schema',
  ok: snapshot !== null && snapshot.schemaVersion === 1 && typeof snapshot.status === 'string',
});

const failed = checks.filter((check) => !check.ok);
console.log(JSON.stringify({ base, checks, snapshot, ok: failed.length === 0 }, null, 2));
process.exit(failed.length === 0 ? 0 : 1);
