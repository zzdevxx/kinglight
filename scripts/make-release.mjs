import { createHash } from 'node:crypto';
import { readFileSync, writeFileSync, mkdirSync, existsSync } from 'node:fs';
import { basename, dirname, resolve } from 'node:path';

function arg(name, fallback = null) {
  const marker = `--${name}`;
  const index = process.argv.indexOf(marker);
  if (index < 0) return fallback;
  const value = process.argv[index + 1];
  if (!value || value.startsWith('--')) throw new Error(`${marker} memerlukan nilai`);
  return value;
}

const installerPath = resolve(arg('installer'));
const portableArg = arg('portable');
const artifactPaths = [installerPath, ...(portableArg ? [resolve(portableArg)] : [])];
const tag = arg('tag');
const sourceCommit = arg('commit');
const publishedAt = arg('published-at');
const portalPath = resolve(arg('portal'));
const version = readFileSync(resolve('VERSION'), 'utf8').trim();

if (tag !== `v${version}`) {
  throw new Error(`Tag ${tag} tidak cocok dengan VERSION ${version}`);
}
for (const artifactPath of artifactPaths) {
  if (!existsSync(artifactPath)) throw new Error(`Artefak tidak ditemukan: ${artifactPath}`);
}
if (!/^[0-9a-f]{40}$/.test(sourceCommit)) {
  throw new Error('sourceCommit harus SHA-1 40 karakter');
}
if (!/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d{3})?Z$/.test(publishedAt) ||
    Number.isNaN(Date.parse(publishedAt)) ||
    new Date(publishedAt).toISOString().replace('.000Z', 'Z') !== publishedAt.replace('.000Z', 'Z')) {
  throw new Error('publishedAt bukan waktu ISO yang valid');
}

const releaseUrl = `https://github.com/zzdevxx/kinglight/releases/tag/${tag}`;
const artifacts = artifactPaths.map((artifactPath) => {
  const bytes = readFileSync(artifactPath);
  const fileName = basename(artifactPath);
  return {
    platform: 'windows',
    arch: 'x64',
    fileName,
    url: `https://github.com/zzdevxx/kinglight/releases/download/${tag}/${encodeURIComponent(fileName)}`,
    sizeBytes: bytes.length,
    sha256: createHash('sha256').update(bytes).digest('hex'),
  };
});

const snapshot = {
  schemaVersion: 1,
  status: 'available',
  version,
  tag,
  sourceCommit,
  publishedAt,
  channel: 'stable',
  releaseUrl,
  artifacts,
};

const producerDir = resolve('docs/contracts');
mkdirSync(producerDir, { recursive: true });
writeFileSync(resolve(producerDir, 'release.json'), `${JSON.stringify(snapshot, null, 2)}\n`, 'utf8');
mkdirSync(dirname(portalPath), { recursive: true });
writeFileSync(portalPath, `${JSON.stringify(snapshot, null, 2)}\n`, 'utf8');

console.log(JSON.stringify({ version, artifacts }, null, 2));
