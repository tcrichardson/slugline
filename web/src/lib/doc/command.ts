export interface ParsedCommand {
  name: string;
  arg: string;
}

/** Parse the text typed after the leading ':' (the colon is not included). */
export function parseCommandLine(input: string): ParsedCommand {
  const trimmed = input.replace(/^\s+/, '');
  const sp = trimmed.indexOf(' ');
  if (sp === -1) return { name: trimmed.toLowerCase(), arg: '' };
  return { name: trimmed.slice(0, sp).toLowerCase(), arg: trimmed.slice(sp + 1).trim() };
}

export type CommandName =
  | 'meeting' | 'note' | 'section' | 'todo'
  | 'start' | 'end' | 'scheduled' | 'purpose' | 'topic' | 'people'
  | 'goto' | 'today' | 'tab' | 'close' | 'w' | 'theme';

export type ArgKind = 'none' | 'text' | 'time' | 'date' | 'theme';

export interface CommandSpec {
  name: CommandName;
  argKind: ArgKind;
  argRequired: boolean;
}

export const COMMANDS: Record<CommandName, CommandSpec> = {
  meeting:   { name: 'meeting',   argKind: 'text',  argRequired: true  },
  note:      { name: 'note',      argKind: 'text',  argRequired: true  },
  section:   { name: 'section',   argKind: 'text',  argRequired: true  },
  todo:      { name: 'todo',      argKind: 'text',  argRequired: true  },
  start:     { name: 'start',     argKind: 'none',  argRequired: false },
  end:       { name: 'end',       argKind: 'none',  argRequired: false },
  scheduled: { name: 'scheduled', argKind: 'time',  argRequired: true  },
  purpose:   { name: 'purpose',   argKind: 'text',  argRequired: true  },
  topic:     { name: 'topic',     argKind: 'text',  argRequired: true  },
  people:    { name: 'people',    argKind: 'text',  argRequired: true  },
  goto:      { name: 'goto',      argKind: 'date',  argRequired: true  },
  today:     { name: 'today',     argKind: 'none',  argRequired: false },
  tab:       { name: 'tab',       argKind: 'date',  argRequired: true  },
  close:     { name: 'close',     argKind: 'none',  argRequired: false },
  w:         { name: 'w',         argKind: 'none',  argRequired: false },
  theme:     { name: 'theme',     argKind: 'theme', argRequired: false },
};

/** Short aliases resolved before COMMANDS lookup. Add future shortcuts here. */
export const ALIASES: Record<string, CommandName> = {
  p: 'people',
};

export type ValidationResult =
  | { ok: true; command: CommandName; arg: string }
  | { ok: false; error: string };

const TIME = /^([01]\d|2[0-3]):[0-5]\d$/;
const DATE = /^\d{4}-\d{2}-\d{2}$/;

export function isValidDate(s: string): boolean {
  if (!DATE.test(s)) return false;
  const [y, m, d] = s.split('-').map(Number);
  const dt = new Date(Date.UTC(y, m - 1, d));
  return dt.getUTCFullYear() === y && dt.getUTCMonth() === m - 1 && dt.getUTCDate() === d;
}

export function validateCommand(input: string): ValidationResult {
  const { name, arg } = parseCommandLine(input);
  const resolved = ALIASES[name] ?? name;
  if (!(resolved in COMMANDS)) return { ok: false, error: `Unknown command: :${name}` };
  const spec = COMMANDS[resolved as CommandName];

  if (spec.argRequired && arg === '') return { ok: false, error: `:${name} requires an argument` };
  if (spec.argKind === 'time' && !TIME.test(arg)) return { ok: false, error: 'Expected HH:MM' };
  if (spec.argKind === 'date' && !isValidDate(arg)) return { ok: false, error: 'Expected YYYY-MM-DD' };
  if (spec.argKind === 'theme' && arg !== '' && arg !== 'light' && arg !== 'dark') {
    return { ok: false, error: 'Expected light or dark' };
  }

  return { ok: true, command: spec.name, arg };
}
