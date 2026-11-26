import { readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';

const base = process.argv[2] || 'https://zzdevxx.github.io/kinglight-download/';
const expected = JSON.parse(readFileSync(new URL('../docs/contracts/release.json', import.meta.url), 'utf8'));
const version = readFileSync(new URL('../VERSION', import.meta.url), 'utf8').trim();
const revision = Date.now().toString();

async function get(path) {
  const url = new URL(path, base);
  url.searchParams.set('verify', revision);
  const response = await fetch(url, { cache: 'no-store' });
  const text = await response.text();
  return { url: url.toString(), status: response.status, text };
}

const checks = [];
const pages = [
  ['./', 'Kinglight'],
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
} catch (error) {
  checks.push({ path: 'release.parse', ok: false, detail: error.message });
  snapshot = null;
}
checks.push({
  path: 'release.schema',
  ok: snapshot !== null && snapshot.schemaVersion === 1 && typeof snapshot.status === 'string',
});

checks.push({
  path: 'release.current',
  ok: expected.version === version && snapshot?.status === 'available' &&
    ['version', 'tag', 'sourceCommit', 'publishedAt', 'channel', 'releaseUrl', 'notesUrl'].every((field) => snapshot[field] === expected[field]) &&
    JSON.stringify(snapshot.artifacts) === JSON.stringify(expected.artifacts),
  expectedVersion: version,
  actualVersion: snapshot?.version,
});

if (checks.every((check) => check.ok)) {
  const downloads = await Promise.allSettled(expected.artifacts.map(async (artifact) => {
    const response = await fetch(artifact.url, { signal: AbortSignal.timeout(30_000) });
    if (!response.ok) throw new Error(`HTTP ${response.status}: ${artifact.fileName}`);
    const bytes = Buffer.from(await response.arrayBuffer());
    const sha256 = createHash('sha256').update(bytes).digest('hex');
    return {
      path: artifact.fileName,
      ok: bytes.length === artifact.sizeBytes && sha256 === artifact.sha256,
      sizeBytes: bytes.length,
      sha256,
    };
  }));
  downloads.forEach((result) => checks.push(result.status === 'fulfilled'
    ? result.value
    : { path: 'artifact.download', ok: false, detail: result.reason.message }));
}

const failed = checks.filter((check) => !check.ok);
console.log(JSON.stringify({ base, checks, snapshot, ok: failed.length === 0 }, null, 2));
process.exit(failed.length === 0 ? 0 : 1);
