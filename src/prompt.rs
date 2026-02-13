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
