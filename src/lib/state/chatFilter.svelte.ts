import { mergeMatches } from "../filterChats";
import { searchChats } from "../ipc";
import type { Chat } from "../types";
import { chats } from "./chatStore.svelte";

const SEARCH_DELAY = 150;

const store = $state({
  filtering: false,
  filterQuery: "",
  contentMatches: [] as Chat[],
});

let timer: ReturnType<typeof setTimeout> | undefined;
let request = 0;

export function filtering(): boolean {
  return store.filtering;
}

export function filterQuery(): string {
  return store.filterQuery;
}

export function filteredChats(): Chat[] {
  return mergeMatches(chats(), store.filterQuery, store.contentMatches);
}

export function setFiltering(value: boolean): void {
  store.filtering = value;
  if (!value) {
    store.filterQuery = "";
    store.contentMatches = [];
    request += 1;
    clearTimeout(timer);
  }
}

export function setFilterQuery(value: string): void {
  store.filterQuery = value;
  clearTimeout(timer);

  const query = value.trim();
  if (!query) {
    request += 1;
    store.contentMatches = [];
    return;
  }

  const id = ++request;
  timer = setTimeout(() => {
    void searchChats(query)
      .then((matches) => {
        if (id === request) store.contentMatches = matches;
      })
      .catch(() => {
        if (id === request) store.contentMatches = [];
      });
  }, SEARCH_DELAY);
}
