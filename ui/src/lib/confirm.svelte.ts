// The inline confirmation of a table row (spec §11.9): one row at a time
// swaps its actions for the question; Confirm runs the action, Cancel
// puts the actions back.
export class RowConfirm {
  /** The key of the row asking, or null. */
  confirming = $state<string | null>(null);
  error = $state<string | null>(null);
  busy = $state(false);

  ask(key: string): void {
    this.confirming = key;
    this.error = null;
  }

  cancel(): void {
    this.confirming = null;
  }

  /** Runs `action` for the row asking; the row stays asking on failure. */
  async confirm(action: () => Promise<void>): Promise<boolean> {
    if (!this.confirming || this.busy) {
      return false;
    }
    this.busy = true;
    try {
      await action();
      this.confirming = null;
      this.error = null;
      return true;
    } catch (error) {
      this.error =
        error instanceof Error
          ? error.message
          : String((error as { message?: string })?.message ?? error);
      return false;
    } finally {
      this.busy = false;
    }
  }
}
