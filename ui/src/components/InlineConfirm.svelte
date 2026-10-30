<script lang="ts">
// Inline confirmation (spec §11.9): the row's actions are replaced by a
// question and [Confirm] [Cancel], sliding in from the right in 180 ms.
import type { Snippet } from "svelte";
import Button from "./Button.svelte";

type Props = {
  confirming: boolean;
  question: string;
  confirmLabel: string;
  cancelLabel: string;
  onconfirm: () => void;
  oncancel: () => void;
  /** The row's usual actions, shown while not confirming. */
  children?: Snippet;
};

let { confirming, question, confirmLabel, cancelLabel, onconfirm, oncancel, children }: Props =
  $props();
</script>

<div class="acts">
  {#if confirming}
    <div class="confirm" role="alertdialog">
      <span>{question}</span>
      <Button variant="danger" size="sm" onclick={onconfirm}>{confirmLabel}</Button>
      <Button size="sm" onclick={oncancel}>{cancelLabel}</Button>
    </div>
  {:else}
    {@render children?.()}
  {/if}
</div>

<style>
  .acts {
    display: flex;
    gap: 6px;
    justify-content: flex-end;
    align-items: center;
    min-height: 24px;
  }

  .confirm {
    display: flex;
    gap: 8px;
    align-items: center;
    font-size: 12px;
    font-weight: 500;
    animation: slidein 0.18s ease-out;
  }

  @keyframes slidein {
    from {
      opacity: 0;
      transform: translateX(24px);
    }

    to {
      opacity: 1;
      transform: none;
    }
  }
</style>
