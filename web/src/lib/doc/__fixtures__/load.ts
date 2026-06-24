import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

// Tests run with cwd = web/, so fixtures live one directory up.
const FIXTURE_DIR = resolve(process.cwd(), '..', 'fixtures');

export function loadFixture(name: string): string {
  return readFileSync(resolve(FIXTURE_DIR, name), 'utf8');
}

export function fixtureLines(name: string): string[] {
  return loadFixture(name).split('\n');
}
