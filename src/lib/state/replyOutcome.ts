type ReplyResult =
  | { kind: "store" }
  | { kind: "keep"; content: string }
  | { kind: "remove" }
  | { kind: "toast" };

type ReplyInput = {
  content: string;
  error?: string;
  hasDetail: boolean;
  placeholderVisible: boolean;
};

// The end-of-stream decision, kept pure so every branch is unit-testable.
export function replyOutcome(input: ReplyInput): ReplyResult {
  if (input.content) return { kind: "store" };
  if (input.error) {
    return input.placeholderVisible ? { kind: "keep", content: "" } : { kind: "toast" };
  }
  return input.hasDetail ? { kind: "keep", content: "_No text response._" } : { kind: "remove" };
}
