import type { ToolStep } from "../types";

// Keeps an answer the model wrote in a round that then made another tool call:
// if the final answer is empty, promote a long narration note back to the body.
export function promoteNote(answer: string, steps: ToolStep[]): string {
  if (answer.trim()) return answer;
  for (let index = steps.length - 1; index >= 0; index -= 1) {
    const note = steps[index].note?.trim();
    if (note && note.length > 200) {
      steps[index].note = undefined;
      return note;
    }
  }
  return answer;
}
