use reedline::Prompt;
use std::borrow::Cow;

pub struct FlutPrompt;

impl Prompt for FlutPrompt {
    fn render_prompt_left(&self) -> Cow<'_, str> {
        "".into()
    }

    fn render_prompt_right(&self) -> Cow<'_, str> {
        "".into()
    }

    fn render_prompt_history_search_indicator(
        &self,
        _: reedline::PromptHistorySearch,
    ) -> Cow<'_, str> {
        "".into()
    }

    fn render_prompt_indicator(&self, _: reedline::PromptEditMode) -> Cow<'_, str> {
        "$ ".into()
    }

    fn render_prompt_multiline_indicator(&self) -> Cow<'_, str> {
        "".into()
    }
}

pub struct Validator;

impl reedline::Validator for Validator {
    fn validate(&self, line: &str) -> reedline::ValidationResult {
        if line.ends_with('\\') || line.ends_with('|') {
            reedline::ValidationResult::Incomplete
        } else {
            reedline::ValidationResult::Complete
        }
    }
}
