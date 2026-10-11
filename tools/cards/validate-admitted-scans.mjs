// Read-only check of cards.json ordinary card art against pinned originals.
// Separately registered societies and art-only IDs are outside this check.
// Existing art outside the pool is preserved; artwork does not admit a card.
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repository = fileURLToPath(new URL('../../', import.meta.url));
const hash = bytes => createHash('sha256').update(bytes).digest('hex');

export function validateAdmittedScans(root = repository, readFile = readFileSync) {
  const read = relative => readFile(path.join(root, relative));
  const pool = JSON.parse(read('rust-game/data/cards.json'));
  const specs = JSON.parse(read('docs/factions/card-specifications.json')).cards;
  const manifest = JSON.parse(read('web/public/card-scans.json'));
  const clientText = read('web/src/game/cardScans.ts').toString();
  const clientMatch = clientText.match(/const scans: Record<string, string> = (\{[\s\S]*?\});/);
  if (!clientMatch) throw new Error('Cannot read the existing client scan registry');
  const client = JSON.parse(clientMatch[1]);
  const errors = [];
  const checkedIds = [];

  for (const card of pool.cards) {
    const id = card.id;
    const pinned = specs[id]?.sourceVerification;
    const entry = manifest[id];
    const url = `/cards/${id}.jpg`;
    if (!/^[A-Z0-9]+$/.test(id) || pinned?.status !== 'completeGameplayFieldsForPinnedImage') {
      errors.push(`${id}: no complete pinned original`);
      continue;
    }
    if (!pinned.imagePath.startsWith('resource/') && !pinned.imagePath.startsWith('docs/factions/recovered-originals/')) {
      errors.push(`${id}: original is outside the checked source archive`);
      continue;
    }
    if (!entry) errors.push(`${id}: missing card-scans.json entry`);
    if (entry && (entry.url !== url || entry.source !== pinned.imagePath || entry.sha256 !== pinned.imageSha256)) {
      errors.push(`${id}: manifest differs from pinned original`);
    }
    if (client[id] !== url) errors.push(`${id}: missing or incorrect client URL`);
    const poolOriginalPaths = [card.source?.image, /\.(?:jpg|jpeg|png)$/i.test(card.source?.file || '') ? card.source.file : undefined];
    if (poolOriginalPaths.some(source => source && source !== pinned.imagePath)) {
      errors.push(`${id}: pool original path differs from pinned original`);
    }
    try {
      const original = read(pinned.imagePath);
      const published = read(`web/public/cards/${id}.jpg`);
      if (hash(original) !== pinned.imageSha256 || !original.equals(published)) {
        errors.push(`${id}: public scan differs from pinned original bytes`);
      }
      if ([card.sourceImageSha256, card.source?.sha256].some(declaredHash => declaredHash && declaredHash !== pinned.imageSha256)) {
        errors.push(`${id}: pool source hash differs from pinned original`);
      }
      checkedIds.push(id);
    } catch (error) {
      errors.push(`${id}: ${error.message}`);
    }
  }
  if (errors.length) throw new Error(errors.join('\n'));
  return { scope: 'cards.json ordinary definitions; societies and art-only IDs excluded',
    checkedCards: checkedIds.length, checkedIds, manifestEntries: Object.keys(manifest).length,
    rulesVersion: pool.rulesVersion, cardPoolVersion: pool.cardPoolVersion, engineVersion: pool.engineVersion };
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    console.log(JSON.stringify(validateAdmittedScans()));
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}
