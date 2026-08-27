import { describe, expect, it } from 'vitest';
import { formatDate } from './api';

describe('formatDate', () => {
  it('marks absent activity clearly', () => expect(formatDate(null)).toBe('Not yet'));
  it('formats SQLite UTC dates for people', () => expect(formatDate('2026-08-27 12:30:00')).not.toContain('Invalid'));
});
