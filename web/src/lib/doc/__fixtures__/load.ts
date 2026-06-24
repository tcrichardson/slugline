import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { resolve } from 'node:path';

// Resolve relative to this source file so it is invariant to the test runner's cwd.
// Path: __fixtures__ -> doc -> lib -> src -> web -> repo root -> fixtures
const FIXTURE_DIR = fileURLToPath(new URL('../../../../../fixtures', import.meta.url));

export function loadFixture(name: string): string {
  return readFileSync(resolve(FIXTURE_DIR, name), 'utf8');
}

export function fixtureLines(name: string): string[] {
  return loadFixture(name).split(/\r?\n/);
}
