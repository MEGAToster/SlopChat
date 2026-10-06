use super::super::stream::ToolCall;
use super::events::is_fetch;
use crate::config::Advanced;

// Counts the turn's tool use and refuses calls past the configured limits.
pub(super) struct Budget {
    pub max_rounds: usize,
    pub max_searches: usize,
    pub max_fetches: usize,
    pub max_fetched_chars: u64,
    pub rounds: usize,
    pub searches: usize,
    pub fetches: usize,
    pub fetched_chars: u64,
    last_round_warned: bool,
}

impl Budget {
    pub fn from_settings(advanced: &Advanced) -> Self {
        Self {
            max_rounds: advanced.max_rounds.clamp(1, 10) as usize,
            max_searches: advanced.max_searches.clamp(1, 20) as usize,
            max_fetches: advanced.max_fetches.clamp(0, 100) as usize,
            max_fetched_chars: advanced.max_fetched_chars,
            rounds: 0,
            searches: 0,
            fetches: 0,
            fetched_chars: 0,
            last_round_warned: false,
        }
    }

    pub fn deep(advanced: &Advanced) -> Self {
        Self {
            max_rounds: advanced.deep.max_rounds.clamp(1, 30) as usize,
            max_searches: advanced.deep.max_searches.clamp(1, 60) as usize,
            max_fetches: advanced.deep.max_fetches.clamp(0, 100) as usize,
            max_fetched_chars: advanced.deep.max_fetched_chars,
            rounds: 0,
            searches: 0,
            fetches: 0,
            fetched_chars: 0,
            last_round_warned: false,
        }
    }

    // Allow-and-count: returns false (without counting) when the call exceeds a limit.
    pub fn allow(&mut self, call: &ToolCall) -> bool {
        let search = call.name == "web_search";
        let fetch = is_fetch(call);
        if search && self.searches >= self.max_searches {
            return false;
        }
        if fetch && (self.fetches >= self.max_fetches || self.fetched_chars >= self.max_fetched_chars)
        {
            return false;
        }
        if search {
            self.searches += 1;
        }
        if fetch {
            self.fetches += 1;
        }
        true
    }

    pub fn add_fetched(&mut self, chars: u64) {
        self.fetched_chars += chars;
    }

    // Appended to every tool result so the model can see its remaining budget.
    pub fn footer(&self) -> String {
        let mut parts = vec![format!(
            "searches left {}/{}",
            self.max_searches.saturating_sub(self.searches),
            self.max_searches
        )];
        if self.max_fetches > 0 && self.max_fetches < 50 {
            parts.push(format!(
                "fetches left {}/{}",
                self.max_fetches.saturating_sub(self.fetches),
                self.max_fetches
            ));
        }
        parts.push(format!(
            "steps left {}/{}",
            self.max_rounds.saturating_sub(self.rounds),
            self.max_rounds
        ));
        format!("\n\n[budget: {}]", parts.join(" · "))
    }

    // True exactly once, at the start of the final allowed round.
    pub fn take_last_round_warning(&mut self) -> bool {
        if self.rounds == self.max_rounds.saturating_sub(1) && !self.last_round_warned {
            self.last_round_warned = true;
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn call(name: &str) -> ToolCall {
        ToolCall {
            id: "1".to_string(),
            name: name.to_string(),
            arguments: "{}".to_string(),
        }
    }

    fn budget() -> Budget {
        Budget {
            max_rounds: 3,
            max_searches: 2,
            max_fetches: 4,
            max_fetched_chars: 100,
            rounds: 0,
            searches: 0,
            fetches: 0,
            fetched_chars: 0,
            last_round_warned: false,
        }
    }

    #[test]
    fn deep_uses_its_own_budgets() {
        let deep = Budget::deep(&Advanced::default());
        assert_eq!(deep.max_rounds, 15);
        assert_eq!(deep.max_searches, 30);
        assert_eq!(deep.max_fetches, 40);
        assert_eq!(deep.max_fetched_chars, 400_000);
    }

    #[test]
    fn deep_refuses_a_search_past_its_limit() {
        let mut deep = Budget::deep(&Advanced::default());
        deep.searches = deep.max_searches;
        assert!(!deep.allow(&call("web_search")));
        assert!(deep.allow(&call("research")));
    }

    #[test]
    fn allow_counts_and_refuses_past_maxima() {        let mut budget = budget();
        assert!(budget.allow(&call("web_search")));
        assert!(budget.allow(&call("web_search")));
        assert!(!budget.allow(&call("web_search")));
        assert_eq!(budget.searches, 2);

        assert!(budget.allow(&call("web_fetch")));
        budget.fetches = budget.max_fetches;
        assert!(!budget.allow(&call("web_fetch")));
    }

    #[test]
    fn allow_refuses_fetches_past_the_char_cap() {
        let mut budget = budget();
        budget.fetched_chars = 100;
        assert!(!budget.allow(&call("web_fetch")));
    }

    #[test]
    fn footer_reports_remaining_and_hides_unlimited_fetches() {
        let mut budget = budget();
        budget.searches = 1;
        budget.rounds = 1;
        assert_eq!(
            budget.footer(),
            "\n\n[budget: searches left 1/2 · fetches left 4/4 · steps left 2/3]"
        );

        budget.max_fetches = 50;
        assert!(!budget.footer().contains("fetches left"));
    }

    #[test]
    fn last_round_warning_fires_once() {
        let mut budget = budget();
        budget.rounds = budget.max_rounds - 1;
        assert!(budget.take_last_round_warning());
        assert!(!budget.take_last_round_warning());
    }
}
