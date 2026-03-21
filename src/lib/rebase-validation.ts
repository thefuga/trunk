export interface ValidationError {
  index: number;
  message: string;
}

export function validateRebasePlan(
  items: { action: string }[]
): ValidationError[] {
  const errors: ValidationError[] = [];

  // Rule 1: Can't drop all commits
  if (items.every(i => i.action === 'drop')) {
    errors.push({ index: 0, message: 'Cannot drop all commits' });
  }

  // Rule 2: First non-dropped commit cannot be squash
  const firstNonDrop = items.findIndex(i => i.action !== 'drop');
  if (firstNonDrop >= 0 && items[firstNonDrop].action === 'squash') {
    errors.push({ index: firstNonDrop, message: 'Cannot squash the first commit' });
  }

  // Rule 3: Each squash must have a non-dropped predecessor above it
  for (let i = 0; i < items.length; i++) {
    if (items[i].action === 'squash') {
      const hasPredecessor = items.slice(0, i).some(p => p.action !== 'drop');
      if (!hasPredecessor) {
        // Avoid duplicate if already caught by Rule 2
        if (!errors.some(e => e.index === i)) {
          errors.push({ index: i, message: 'No preceding commit to squash into' });
        }
      }
    }
  }

  return errors;
}
