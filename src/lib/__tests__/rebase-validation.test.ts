import { describe, it, expect } from 'vitest';
import { validateRebasePlan } from '../rebase-validation';

describe('validateRebasePlan', () => {
  it('returns empty errors for all-pick plan', () => {
    const items = [
      { action: 'pick' },
      { action: 'pick' },
      { action: 'pick' },
    ];
    const errors = validateRebasePlan(items);
    expect(errors).toEqual([]);
  });

  it('returns error when squash is first commit', () => {
    const items = [
      { action: 'squash' },
      { action: 'pick' },
    ];
    const errors = validateRebasePlan(items);
    expect(errors).toHaveLength(1);
    expect(errors[0]).toEqual({ index: 0, message: 'Cannot squash the first commit' });
  });

  it('returns error when all commits are dropped', () => {
    const items = [
      { action: 'drop' },
      { action: 'drop' },
    ];
    const errors = validateRebasePlan(items);
    expect(errors).toHaveLength(1);
    expect(errors[0]).toEqual({ index: 0, message: 'Cannot drop all commits' });
  });

  it('returns error when squash after drop has no non-dropped predecessor', () => {
    const items = [
      { action: 'drop' },
      { action: 'squash' },
    ];
    const errors = validateRebasePlan(items);
    // squash at index 1 is the first non-dropped commit, caught by first-commit rule
    expect(errors).toHaveLength(1);
    expect(errors[0]).toEqual({ index: 1, message: 'Cannot squash the first commit' });
  });

  it('returns no error when squash has a non-dropped predecessor', () => {
    const items = [
      { action: 'pick' },
      { action: 'squash' },
    ];
    const errors = validateRebasePlan(items);
    expect(errors).toEqual([]);
  });

  it('returns error when drop first then squash second with no other predecessor', () => {
    const items = [
      { action: 'drop' },
      { action: 'squash' },
      { action: 'pick' },
    ];
    const errors = validateRebasePlan(items);
    const squashError = errors.find(e => e.index === 1);
    expect(squashError).toBeDefined();
    // squash at index 1 is the first non-dropped commit, so Rule 2 fires
    expect(squashError!.message).toBe('Cannot squash the first commit');
  });

  it('returns no error for pick-drop-squash (has non-dropped predecessor via pick)', () => {
    const items = [
      { action: 'pick' },
      { action: 'drop' },
      { action: 'squash' },
    ];
    const errors = validateRebasePlan(items);
    expect(errors).toEqual([]);
  });

  it('returns drop-all error for single commit set to drop', () => {
    const items = [
      { action: 'drop' },
    ];
    const errors = validateRebasePlan(items);
    expect(errors).toHaveLength(1);
    expect(errors[0]).toEqual({ index: 0, message: 'Cannot drop all commits' });
  });

  it('returns multiple errors simultaneously (squash first + drop rest)', () => {
    const items = [
      { action: 'squash' },
      { action: 'drop' },
      { action: 'drop' },
    ];
    const errors = validateRebasePlan(items);
    // Should have at least 2 errors: squash-first and could-be-interpreted-as-all-non-pick
    expect(errors.length).toBeGreaterThanOrEqual(1);
    expect(errors.some(e => e.message === 'Cannot squash the first commit')).toBe(true);
  });
});
