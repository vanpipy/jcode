use super::*;
use crate::desktop_rich_text::{
    AnsiColor, AnsiStyle, RichLine, RichLineStyle, RichSpanStyle, SyntaxTokenKind,
};
use crate::single_session::{
    InlineWidgetKind, MODEL_PICKER_INLINE_ROW_LIMIT, SingleSessionInlineSpan,
    SingleSessionInlineSpanKind, SingleSessionToolLineKind, SingleSessionToolLineMetadata,
    SingleSessionToolVisualState, SingleSessionTypography, single_session_assistant_font_family,
    single_session_trimmed_line_end_preserving_inline_code_whitespace,
    single_session_user_font_family,
};

mod handwriting;
mod lucide;
mod math;
mod motion;
mod text_style;
mod wrapping;
mod welcome_hero;
mod inline_widget_chrome;
mod motion_transcript;
mod motion_composer;
mod motion_inline_widget;
mod transcript_cards;
mod inline_markdown;
mod scrollbar;
mod selection_caret;
mod body_viewport;
mod text_buffers;
mod text_areas;

use handwriting::handwritten_welcome_paths_for_phrase;
use lucide::{LucideIcon, push_lucide_icon};
use math::*;
pub(crate) use motion::*;
use std::collections::{HashMap, hash_map::DefaultHasher};
use std::hash::{Hash, Hasher};
pub(crate) use text_style::*;
use wrapping::*;
pub(crate) use welcome_hero::*;
pub(crate) use inline_widget_chrome::*;
pub(crate) use motion_transcript::*;
pub(crate) use motion_composer::*;
pub(crate) use motion_inline_widget::*;
pub(crate) use transcript_cards::*;
pub(crate) use inline_markdown::*;
pub(crate) use scrollbar::*;
pub(crate) use selection_caret::*;
pub(crate) use body_viewport::*;
pub(crate) use text_buffers::*;
pub(crate) use text_areas::*;

pub(crate) const INLINE_MATH_BACKGROUND_COLOR: [f32; 4] = [0.035, 0.220, 0.155, 0.115];
pub(crate) const MARKDOWN_HEADING_BACKGROUND_COLOR: [f32; 4] = [0.060, 0.180, 0.520, 0.055];
pub(crate) const MARKDOWN_MEDIA_BACKGROUND_COLOR: [f32; 4] = [0.030, 0.255, 0.185, 0.070];
pub(crate) const MARKDOWN_RULE_COLOR: [f32; 4] = [0.060, 0.130, 0.260, 0.220];
pub(crate) const MARKDOWN_LIST_MARKER_COLOR: [f32; 4] = [0.060, 0.110, 0.240, 0.960];
pub(crate) const MARKDOWN_TASK_DONE_COLOR: [f32; 4] = [0.025, 0.350, 0.190, 1.000];
pub(crate) const MARKDOWN_TASK_OPEN_COLOR: [f32; 4] = [0.420, 0.320, 0.075, 0.980];
pub(crate) const MARKDOWN_STRIKE_TEXT_COLOR: [f32; 4] = [0.310, 0.330, 0.380, 0.880];
pub(crate) const STREAMING_ACTIVITY_PILL_COLOR: [f32; 4] = [0.965, 0.985, 1.000, 0.58];
pub(crate) const STREAMING_ACTIVITY_PILL_BORDER_COLOR: [f32; 4] = [0.000, 0.260, 0.720, 0.18];
const INLINE_WIDGET_CARD_SHADOW_COLOR: [f32; 4] = [0.020, 0.035, 0.070, 0.080];
pub(crate) const INLINE_WIDGET_CARD_BACKGROUND_COLOR: [f32; 4] = [0.992, 0.996, 1.000, 0.72];
const INLINE_WIDGET_CARD_BORDER_COLOR: [f32; 4] = [0.105, 0.185, 0.360, 0.20];
const INLINE_WIDGET_CARD_HIGHLIGHT_COLOR: [f32; 4] = [1.000, 1.000, 1.000, 0.52];
const INLINE_WIDGET_CARD_ACCENT_COLOR: [f32; 4] = [0.125, 0.420, 0.920, 0.34];
pub(crate) const SLASH_SUGGESTIONS_INLINE_CARD_BACKGROUND_COLOR: [f32; 4] =
    [0.948, 0.966, 1.000, 0.90];
const SLASH_SUGGESTIONS_INLINE_CARD_BORDER_COLOR: [f32; 4] = [0.090, 0.230, 0.620, 0.32];
const SLASH_SUGGESTIONS_INLINE_CARD_HIGHLIGHT_COLOR: [f32; 4] = [1.000, 1.000, 1.000, 0.62];
const SLASH_SUGGESTIONS_INLINE_CARD_ACCENT_COLOR: [f32; 4] = [0.105, 0.355, 0.950, 0.48];
pub(crate) const SLASH_SUGGESTIONS_INLINE_SELECTION_BACKGROUND_COLOR: [f32; 4] =
    [0.215, 0.420, 0.900, 0.155];
pub(crate) const MODEL_PICKER_CARD_BACKGROUND_COLOR: [f32; 4] = [0.946, 0.962, 0.988, 0.975];
const MODEL_PICKER_CARD_BORDER_COLOR: [f32; 4] = [0.105, 0.140, 0.235, 0.26];
const MODEL_PICKER_CARD_HIGHLIGHT_COLOR: [f32; 4] = [1.000, 1.000, 1.000, 0.55];
const MODEL_PICKER_CARD_ACCENT_COLOR: [f32; 4] = [0.110, 0.310, 0.760, 0.40];
const MODEL_PICKER_SELECTION_BACKGROUND_COLOR: [f32; 4] = [0.160, 0.330, 0.760, 0.105];
const SINGLE_SESSION_SCROLLBAR_TRACK_WIDTH: f32 = 3.0;
const SINGLE_SESSION_SCROLLBAR_GAP: f32 = 8.0;
const SINGLE_SESSION_SCROLLBAR_THUMB_TRANSITION_DURATION: Duration = Duration::from_millis(140);
const SINGLE_SESSION_SCROLLBAR_FADE_IDLE_DURATION: Duration = Duration::from_millis(620);
const SINGLE_SESSION_SCROLLBAR_FADE_DURATION: Duration = Duration::from_millis(260);
const SINGLE_SESSION_SCROLLBAR_TRACK_COLOR: [f32; 4] = [0.040, 0.055, 0.090, 0.075];
const SINGLE_SESSION_SCROLLBAR_THUMB_COLOR: [f32; 4] = [0.035, 0.065, 0.145, 0.34];
const TRANSCRIPT_CARD_ENTRY_DURATION: Duration = Duration::from_millis(170);
const TRANSCRIPT_CARD_SHIFT_DURATION: Duration = Duration::from_millis(150);
const TRANSCRIPT_CARD_EXIT_DURATION: Duration = Duration::from_millis(145);
const TRANSCRIPT_CARD_ENTRY_OFFSET_PIXELS: f32 = 10.0;
const TRANSCRIPT_CARD_ENTRY_SCALE: f32 = 0.988;
const TRANSCRIPT_MESSAGE_ENTRY_DURATION: Duration = Duration::from_millis(150);
const TRANSCRIPT_MESSAGE_SHIFT_DURATION: Duration = Duration::from_millis(135);
const TRANSCRIPT_MESSAGE_ENTRY_OFFSET_PIXELS: f32 = 7.0;
const TRANSCRIPT_MESSAGE_ENTRY_SCALE: f32 = 0.992;
const TRANSCRIPT_MESSAGE_ASSISTANT_HIGHLIGHT_COLOR: [f32; 4] = [0.070, 0.125, 0.260, 0.038];
const TRANSCRIPT_MESSAGE_USER_HIGHLIGHT_COLOR: [f32; 4] = [0.060, 0.210, 0.650, 0.058];
const TRANSCRIPT_MESSAGE_META_HIGHLIGHT_COLOR: [f32; 4] = [0.075, 0.160, 0.260, 0.046];
const TRANSCRIPT_MESSAGE_ERROR_HIGHLIGHT_COLOR: [f32; 4] = [0.700, 0.080, 0.100, 0.060];
const TRANSCRIPT_MESSAGE_ACCENT_ALPHA_MULTIPLIER: f32 = 2.8;
const INLINE_MARKDOWN_PILL_ENTRY_DURATION: Duration = Duration::from_millis(145);
const INLINE_MARKDOWN_PILL_SHIFT_DURATION: Duration = Duration::from_millis(130);
const INLINE_MARKDOWN_PILL_EXIT_DURATION: Duration = Duration::from_millis(125);
const INLINE_MARKDOWN_PILL_ENTRY_OFFSET_PIXELS: f32 = 4.0;
const INLINE_MARKDOWN_PILL_ENTRY_SCALE: f32 = 0.94;
const INLINE_WIDGET_SELECTION_TRANSITION_DURATION: Duration = Duration::from_millis(170);
const INLINE_WIDGET_PREVIEW_PANE_FOCUS_DURATION: Duration = Duration::from_millis(150);
const INLINE_WIDGET_PREVIEW_PANE_CONTENT_DURATION: Duration = Duration::from_millis(145);
pub(crate) const INLINE_WIDGET_PREVIEW_PANE_BACKGROUND_COLOR: [f32; 4] =
    [0.968, 0.984, 1.000, 0.430];
const INLINE_WIDGET_PREVIEW_PANE_BORDER_COLOR: [f32; 4] = [0.090, 0.205, 0.480, 0.180];
pub(crate) const INLINE_WIDGET_PREVIEW_PANE_FOCUS_COLOR: [f32; 4] = [0.100, 0.340, 0.920, 0.180];
const INLINE_WIDGET_PREVIEW_PANE_CONTENT_COLOR: [f32; 4] = [0.125, 0.420, 0.920, 0.105];
const INLINE_WIDGET_LIST_REFLOW_ENTRY_DURATION: Duration = Duration::from_millis(170);
const INLINE_WIDGET_LIST_REFLOW_SHIFT_DURATION: Duration = Duration::from_millis(170);
const INLINE_WIDGET_LIST_REFLOW_EXIT_DURATION: Duration = Duration::from_millis(135);
const INLINE_WIDGET_LIST_REFLOW_COLOR: [f32; 4] = [0.105, 0.355, 0.950, 0.090];
const COMPOSER_MOTION_DURATION: Duration = Duration::from_millis(165);
pub(crate) const COMPOSER_FOCUS_RING_COLOR: [f32; 4] = [0.090, 0.250, 0.680, 0.185];
pub(crate) const COMPOSER_PLACEHOLDER_RAIL_COLOR: [f32; 4] = [0.105, 0.185, 0.360, 0.185];
pub(crate) const COMPOSER_SUBMIT_READY_COLOR: [f32; 4] = [0.105, 0.355, 0.950, 0.700];
pub(crate) const COMPOSER_SUBMIT_BUSY_COLOR: [f32; 4] = [0.055, 0.540, 0.360, 0.700];
const ATTACHMENT_CHIP_ENTRY_DURATION: Duration = Duration::from_millis(150);
const ATTACHMENT_CHIP_SHIFT_DURATION: Duration = Duration::from_millis(140);
const ATTACHMENT_CHIP_EXIT_DURATION: Duration = Duration::from_millis(130);
const ATTACHMENT_CHIP_WIDTH: f32 = 42.0;
const ATTACHMENT_CHIP_HEIGHT: f32 = 20.0;
const ATTACHMENT_CHIP_GAP: f32 = 6.0;
const ATTACHMENT_CHIP_VISIBLE_LIMIT: usize = 4;
pub(crate) const ATTACHMENT_CHIP_BACKGROUND_COLOR: [f32; 4] = [0.940, 0.972, 1.000, 0.720];
pub(crate) const ATTACHMENT_CHIP_ACCENT_COLOR: [f32; 4] = [0.090, 0.355, 0.900, 0.620];
pub(crate) const ATTACHMENT_CHIP_EXIT_COLOR: [f32; 4] = [0.530, 0.590, 0.690, 0.430];
const STDIN_OVERLAY_ENTRY_DURATION: Duration = Duration::from_millis(165);
const STDIN_OVERLAY_RESIZE_DURATION: Duration = Duration::from_millis(155);
const STDIN_OVERLAY_EXIT_DURATION: Duration = Duration::from_millis(145);
const STDIN_OVERLAY_ENTRY_OFFSET_PIXELS: f32 = 9.0;
const STDIN_OVERLAY_ENTRY_SCALE: f32 = 0.985;
pub(crate) const STDIN_OVERLAY_BACKGROUND_COLOR: [f32; 4] = [0.966, 0.982, 1.000, 0.640];
pub(crate) const STDIN_OVERLAY_BORDER_COLOR: [f32; 4] = [0.085, 0.270, 0.760, 0.250];
pub(crate) const STDIN_OVERLAY_INPUT_RAIL_COLOR: [f32; 4] = [0.115, 0.410, 0.940, 0.300];
pub(crate) const STDIN_OVERLAY_SUBMIT_COLOR: [f32; 4] = [0.060, 0.500, 0.340, 0.660];
pub(crate) const STDIN_OVERLAY_EXIT_COLOR: [f32; 4] = [0.500, 0.570, 0.680, 0.420];
const TOOL_CARD_ENTRY_DURATION: Duration = Duration::from_millis(180);
const TOOL_CARD_EXIT_DURATION: Duration = Duration::from_millis(160);
const TOOL_CARD_STATE_TRANSITION_DURATION: Duration = Duration::from_millis(160);
const TOOL_CARD_OUTPUT_REVEAL_DURATION: Duration = Duration::from_millis(180);
const TOOL_CARD_RESOLUTION_FLASH_DURATION: Duration = Duration::from_millis(320);
const TOOL_CARD_ENTRY_OFFSET_PIXELS: f32 = 12.0;
const TOOL_CARD_ENTRY_SCALE: f32 = 0.985;

#[cfg(test)]
pub(crate) fn build_single_session_vertices(
    app: &SingleSessionApp,
    size: PhysicalSize<u32>,
    focus_pulse: f32,
    spinner_tick: u64,
) -> Vec<Vertex> {
    build_single_session_vertices_with_scroll(app, size, focus_pulse, spinner_tick, 0.0)
}

#[cfg(test)]
pub(crate) fn build_single_session_vertices_with_scroll(
    app: &SingleSessionApp,
    size: PhysicalSize<u32>,
    focus_pulse: f32,
    spinner_tick: u64,
    smooth_scroll_lines: f32,
) -> Vec<Vertex> {
    let welcome_hero_reveal_progress = welcome_hero_reveal_progress_for_tick(spinner_tick);
    build_single_session_vertices_with_scroll_and_reveal(
        app,
        size,
        focus_pulse,
        spinner_tick,
        smooth_scroll_lines,
        welcome_hero_reveal_progress,
    )
}

pub(crate) fn build_single_session_vertices_with_scroll_and_reveal(
    app: &SingleSessionApp,
    size: PhysicalSize<u32>,
    focus_pulse: f32,
    spinner_tick: u64,
    smooth_scroll_lines: f32,
    welcome_hero_reveal_progress: f32,
) -> Vec<Vertex> {
    let width = size.width as f32;
    let height = size.height as f32;
    let mut vertices = Vec::new();
    let rendered_body_lines = single_session_rendered_body_lines_for_tick(app, size, spinner_tick);

    push_gradient_rect(
        &mut vertices,
        Rect {
            x: 0.0,
            y: 0.0,
            width,
            height,
        },
        BACKGROUND_TOP_LEFT,
        BACKGROUND_BOTTOM_LEFT,
        BACKGROUND_BOTTOM_RIGHT,
        BACKGROUND_TOP_RIGHT,
        size,
    );

    let rect = Rect {
        x: 0.0,
        y: 0.0,
        width: width.max(1.0),
        height: height.max(1.0),
    };
    let surface = single_session_surface(app.session.as_ref());
    push_single_session_surface_without_bottom_rule(
        &mut vertices,
        rect,
        surface.color_index,
        focus_pulse,
        size,
    );

    let layout = single_session_layout_for_total_lines(app, size, rendered_body_lines.len());
    push_single_session_composer_chrome(&mut vertices, app, size, None, None, Some(layout));

    let welcome_chrome_offset = if app.is_welcome_timeline_visible() {
        welcome_timeline_visual_offset_pixels_for_total_lines(
            app,
            size,
            smooth_scroll_lines,
            rendered_body_lines.len(),
        )
    } else {
        0.0
    };
    if welcome_timeline_chrome_visible(app, size, welcome_chrome_offset) {
        push_fresh_welcome_ambient(&mut vertices, size, spinner_tick, welcome_chrome_offset);
        push_handwritten_welcome_hero_with_offset(
            &mut vertices,
            &app.welcome_hero_text(),
            size,
            app.text_scale(),
            welcome_hero_reveal_progress,
            welcome_chrome_offset,
        );
    }

    push_single_session_inline_widget_card(
        &mut vertices,
        app,
        size,
        welcome_chrome_offset,
        rendered_body_lines.len(),
        None,
        None,
        None,
    );
    push_single_session_stdin_overlay(&mut vertices, app, size, &rendered_body_lines, None);
    let viewport = single_session_body_viewport_from_lines(
        app,
        size,
        smooth_scroll_lines,
        &rendered_body_lines,
    );
    push_single_session_transcript_message_highlights_from_viewport(
        &mut vertices,
        app,
        size,
        &viewport,
        rendered_body_lines.len(),
        None,
    );
    push_single_session_transcript_cards(
        &mut vertices,
        app,
        size,
        spinner_tick,
        smooth_scroll_lines,
    );
    push_single_session_tool_cards(
        &mut vertices,
        app,
        size,
        spinner_tick,
        smooth_scroll_lines,
        None,
    );
    push_single_session_inline_code_cards(
        &mut vertices,
        app,
        size,
        spinner_tick,
        smooth_scroll_lines,
    );
    push_single_session_markdown_rule_lines(
        &mut vertices,
        app,
        size,
        spinner_tick,
        smooth_scroll_lines,
    );
    if app.streaming_activity_pill_visible() {
        push_streaming_activity_cue(&mut vertices, app, size, spinner_tick, None, None);
    }
    if app.has_activity_indicator() && !app.streaming_response.is_empty() {
        let viewport = single_session_body_viewport_for_tick(app, size, spinner_tick, 0.0);
        push_single_session_streaming_tail_cursor(
            &mut vertices,
            app,
            size,
            &viewport,
            None,
            None,
            spinner_tick as f32 * (DESKTOP_SPINNER_FRAME_MS as f32 / 1000.0),
        );
    }
    push_single_session_selection(&mut vertices, app, size, None);
    push_single_session_scrollbar(
        &mut vertices,
        app,
        size,
        spinner_tick,
        smooth_scroll_lines,
        None,
    );

    vertices
}

pub(crate) fn build_single_session_vertices_with_cached_body(
    app: &SingleSessionApp,
    size: PhysicalSize<u32>,
    focus_pulse: f32,
    spinner_tick: u64,
    smooth_scroll_lines: f32,
    welcome_hero_reveal_progress: f32,
    rendered_body_lines: &[SingleSessionStyledLine],
) -> Vec<Vertex> {
    build_single_session_vertices_with_cached_body_internal(
        app,
        size,
        focus_pulse,
        spinner_tick,
        smooth_scroll_lines,
        welcome_hero_reveal_progress,
        rendered_body_lines,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn build_single_session_vertices_with_cached_body_and_tool_motion(
    app: &SingleSessionApp,
    size: PhysicalSize<u32>,
    focus_pulse: f32,
    spinner_tick: u64,
    smooth_scroll_lines: f32,
    welcome_hero_reveal_progress: f32,
    rendered_body_lines: &[SingleSessionStyledLine],
    inline_selection_motion: Option<&InlineWidgetSelectionMotionFrame>,
    inline_list_reflow_motion: Option<&InlineWidgetListReflowMotionFrame>,
    inline_preview_pane_motion: Option<&InlineWidgetPreviewPaneMotionFrame>,
    composer_motion: Option<&ComposerMotionFrame>,
    attachment_chip_motion: Option<&AttachmentChipMotionFrame>,
    stdin_overlay_motion: Option<&StdinOverlayMotionFrame>,
    transcript_message_motion: Option<&TranscriptMessageMotionFrame>,
    transcript_motion: Option<&TranscriptCardMotionFrame>,
    inline_markdown_motion: Option<&InlineMarkdownPillMotionFrame>,
    activity_cue_motion: Option<&StreamingActivityCueMotionFrame>,
    tool_motion: &ToolCardMotionFrame,
    scrollbar_motion: Option<&SingleSessionScrollbarMotionFrame>,
) -> Vec<Vertex> {
    build_single_session_vertices_with_cached_body_internal(
        app,
        size,
        focus_pulse,
        spinner_tick,
        smooth_scroll_lines,
        welcome_hero_reveal_progress,
        rendered_body_lines,
        inline_selection_motion,
        inline_list_reflow_motion,
        inline_preview_pane_motion,
        composer_motion,
        attachment_chip_motion,
        stdin_overlay_motion,
        transcript_message_motion,
        transcript_motion,
        inline_markdown_motion,
        activity_cue_motion,
        Some(tool_motion),
        scrollbar_motion,
    )
}

#[allow(clippy::too_many_arguments)]
fn build_single_session_vertices_with_cached_body_internal(
    app: &SingleSessionApp,
    size: PhysicalSize<u32>,
    focus_pulse: f32,
    spinner_tick: u64,
    smooth_scroll_lines: f32,
    welcome_hero_reveal_progress: f32,
    rendered_body_lines: &[SingleSessionStyledLine],
    inline_selection_motion: Option<&InlineWidgetSelectionMotionFrame>,
    inline_list_reflow_motion: Option<&InlineWidgetListReflowMotionFrame>,
    inline_preview_pane_motion: Option<&InlineWidgetPreviewPaneMotionFrame>,
    composer_motion: Option<&ComposerMotionFrame>,
    attachment_chip_motion: Option<&AttachmentChipMotionFrame>,
    stdin_overlay_motion: Option<&StdinOverlayMotionFrame>,
    transcript_message_motion: Option<&TranscriptMessageMotionFrame>,
    transcript_motion: Option<&TranscriptCardMotionFrame>,
    inline_markdown_motion: Option<&InlineMarkdownPillMotionFrame>,
    activity_cue_motion: Option<&StreamingActivityCueMotionFrame>,
    tool_motion: Option<&ToolCardMotionFrame>,
    scrollbar_motion: Option<&SingleSessionScrollbarMotionFrame>,
) -> Vec<Vertex> {
    let width = size.width as f32;
    let height = size.height as f32;
    let mut vertices = Vec::with_capacity(2048);

    push_gradient_rect(
        &mut vertices,
        Rect {
            x: 0.0,
            y: 0.0,
            width,
            height,
        },
        BACKGROUND_TOP_LEFT,
        BACKGROUND_BOTTOM_LEFT,
        BACKGROUND_BOTTOM_RIGHT,
        BACKGROUND_TOP_RIGHT,
        size,
    );

    let rect = Rect {
        x: 0.0,
        y: 0.0,
        width: width.max(1.0),
        height: height.max(1.0),
    };
    let surface = single_session_surface(app.session.as_ref());
    push_single_session_surface_without_bottom_rule(
        &mut vertices,
        rect,
        surface.color_index,
        focus_pulse,
        size,
    );

    let layout = single_session_layout_for_total_lines(app, size, rendered_body_lines.len());
    push_single_session_composer_chrome(
        &mut vertices,
        app,
        size,
        composer_motion,
        attachment_chip_motion,
        Some(layout),
    );

    let welcome_chrome_offset = if app.is_welcome_timeline_visible() {
        welcome_timeline_visual_offset_pixels_for_total_lines(
            app,
            size,
            smooth_scroll_lines,
            rendered_body_lines.len(),
        )
    } else {
        0.0
    };
    if welcome_timeline_chrome_visible(app, size, welcome_chrome_offset) {
        push_fresh_welcome_ambient(&mut vertices, size, spinner_tick, welcome_chrome_offset);
        push_handwritten_welcome_hero_with_offset(
            &mut vertices,
            &app.welcome_hero_text(),
            size,
            app.text_scale(),
            welcome_hero_reveal_progress,
            welcome_chrome_offset,
        );
    }

    push_single_session_inline_widget_card(
        &mut vertices,
        app,
        size,
        welcome_chrome_offset,
        rendered_body_lines.len(),
        inline_selection_motion,
        inline_list_reflow_motion,
        inline_preview_pane_motion,
    );

    push_single_session_stdin_overlay(
        &mut vertices,
        app,
        size,
        rendered_body_lines,
        stdin_overlay_motion,
    );

    let viewport = single_session_body_viewport_from_lines(
        app,
        size,
        smooth_scroll_lines,
        rendered_body_lines,
    );
    push_single_session_transcript_message_highlights_from_viewport(
        &mut vertices,
        app,
        size,
        &viewport,
        rendered_body_lines.len(),
        transcript_message_motion,
    );
    push_single_session_transcript_cards_from_viewport(
        &mut vertices,
        app,
        size,
        &viewport,
        rendered_body_lines.len(),
        transcript_motion,
    );
    push_single_session_tool_cards_from_viewport(
        &mut vertices,
        app,
        size,
        &viewport,
        rendered_body_lines.len(),
        spinner_tick,
        tool_motion,
    );
    push_single_session_inline_code_cards_from_viewport(
        &mut vertices,
        app,
        size,
        &viewport,
        rendered_body_lines.len(),
        inline_markdown_motion,
    );
    push_single_session_markdown_rule_lines_from_viewport(
        &mut vertices,
        app,
        size,
        &viewport,
        rendered_body_lines.len(),
    );
    if app.streaming_activity_pill_visible()
        || activity_cue_motion.is_some_and(|motion| motion.exiting().is_some())
    {
        push_streaming_activity_cue(
            &mut vertices,
            app,
            size,
            spinner_tick,
            Some(&viewport),
            activity_cue_motion,
        );
    }
    push_single_session_selection(&mut vertices, app, size, Some(&viewport.lines));
    push_single_session_scrollbar_for_total_lines(
        &mut vertices,
        app,
        size,
        smooth_scroll_lines,
        rendered_body_lines.len(),
        scrollbar_motion,
    );

    vertices
}

fn single_session_scrollbar_track_x(size: PhysicalSize<u32>) -> f32 {
    size.width as f32 - PANEL_TITLE_LEFT_PADDING - 4.0
}

fn single_session_content_right(size: PhysicalSize<u32>) -> f32 {
    (single_session_scrollbar_track_x(size) - SINGLE_SESSION_SCROLLBAR_GAP)
        .max(PANEL_TITLE_LEFT_PADDING + 1.0)
}

fn single_session_content_width(size: PhysicalSize<u32>) -> f32 {
    (single_session_content_right(size) - PANEL_TITLE_LEFT_PADDING).max(1.0)
}

#[derive(Clone, Copy, Debug)]
struct SingleSessionLayoutMetrics {
    body_line_height: f32,
    composer_line_height: f32,
}

#[derive(Clone, Copy, Debug)]
struct SingleSessionLayout {
    body: Rect,
    draft_top: f32,
    composer: Rect,
    activity_lane: Option<Rect>,
    metrics: SingleSessionLayoutMetrics,
}

impl SingleSessionLayout {
    #[inline]
    fn body_bottom(self) -> f32 {
        rect_bottom(self.body)
    }

    #[inline]
    fn body_text_bounds_bottom(self) -> i32 {
        text_bounds_bottom(self.body_bottom())
    }
}

fn single_session_layout_metrics(app: &SingleSessionApp) -> SingleSessionLayoutMetrics {
    let typography = single_session_typography_for_scale(app.text_scale());
    SingleSessionLayoutMetrics {
        body_line_height: typography.body_size * typography.body_line_height,
        composer_line_height: typography.code_size * typography.code_line_height,
    }
}

fn single_session_layout_for_app(
    app: &SingleSessionApp,
    size: PhysicalSize<u32>,
) -> SingleSessionLayout {
    single_session_layout_from_bounds(
        app,
        size,
        single_session_draft_top_for_app(app, size),
        single_session_body_bottom_base_for_app(app, size),
    )
}

fn single_session_layout_for_total_lines(
    app: &SingleSessionApp,
    size: PhysicalSize<u32>,
    total_lines: usize,
) -> SingleSessionLayout {
    single_session_layout_from_bounds(
        app,
        size,
        single_session_draft_top_for_total_lines(app, size, total_lines),
        single_session_body_bottom_base_for_total_lines(app, size, total_lines),
    )
}

fn single_session_layout_from_bounds(
    app: &SingleSessionApp,
    size: PhysicalSize<u32>,
    draft_top: f32,
    body_base_bottom: f32,
) -> SingleSessionLayout {
    let metrics = single_session_layout_metrics(app);
    let body_top = PANEL_BODY_TOP_PADDING;
    let body_base_bottom = body_base_bottom.max(body_top);
    let inline_widget_reserved_height = inline_widget_reserved_height(app);
    let activity_reserved_height = streaming_activity_reserved_height(app);
    let body_bottom =
        (body_base_bottom - inline_widget_reserved_height - activity_reserved_height).max(body_top);
    let activity_lane = (activity_reserved_height > 0.0).then(|| {
        let activity_top = (body_base_bottom - activity_reserved_height).max(body_top);
        Rect {
            x: PANEL_TITLE_LEFT_PADDING,
            y: activity_top,
            width: single_session_content_width(size),
            height: (body_base_bottom - activity_top).max(0.0),
        }
    });
    let composer_target = composer_motion_target(app);
    let composer_visual = ComposerMotionVisual::settled(composer_target);
    let composer_height = single_session_composer_height(size, metrics, composer_visual);

    SingleSessionLayout {
        body: Rect {
            x: PANEL_TITLE_LEFT_PADDING,
            y: body_top,
            width: single_session_content_width(size),
            height: (body_bottom - body_top).max(0.0),
        },
        draft_top,
        composer: Rect {
            x: PANEL_TITLE_LEFT_PADDING - 10.0,
            y: draft_top - 9.0,
            width: single_session_content_width(size) + 20.0,
            height: composer_height,
        },
        activity_lane,
        metrics,
    }
}

fn inline_widget_bottom_limit_for_layout(
    app: &SingleSessionApp,
    layout: SingleSessionLayout,
    welcome_chrome_visible: bool,
) -> f32 {
    if welcome_chrome_visible
        && app.render_inline_widget_line_count() > 0
        && !app.has_welcome_timeline_transcript()
    {
        return layout.draft_top;
    }

    layout
        .activity_lane
        .map(|activity| activity.y)
        .unwrap_or(layout.draft_top)
}

fn single_session_composer_height(
    size: PhysicalSize<u32>,
    metrics: SingleSessionLayoutMetrics,
    visual: ComposerMotionVisual,
) -> f32 {
    (visual.height_lines.max(1.0) * metrics.composer_line_height + 18.0)
        .min((size.height as f32 * 0.34).max(metrics.composer_line_height + 18.0))
}

#[inline]
fn rect_bottom(rect: Rect) -> f32 {
    rect.y + rect.height
}

fn push_single_session_surface_without_bottom_rule(
    vertices: &mut Vec<Vertex>,
    rect: Rect,
    color_index: usize,
    focus_pulse: f32,
    size: PhysicalSize<u32>,
) {
    let accent = panel_accent_color(color_index, true);
    push_rounded_rect(
        vertices,
        rect,
        PANEL_RADIUS,
        with_alpha(accent, 0.105),
        size,
    );
    push_rounded_rect(
        vertices,
        Rect {
            x: rect.x,
            y: rect.y,
            width: 5.0_f32.min(rect.width),
            height: rect.height,
        },
        PANEL_RADIUS,
        with_alpha(accent, 0.78),
        size,
    );

    let stroke_width = FOCUSED_BORDER_WIDTH + focus_pulse * 2.5;
    push_top_and_side_surface_outline(vertices, rect, stroke_width, accent, size);

    if focus_pulse > 0.0 {
        let pulse_rect = inset_rect(rect, -3.0 * focus_pulse);
        push_top_and_side_surface_outline(
            vertices,
            pulse_rect,
            1.0,
            with_alpha(FOCUS_RING_COLOR, 0.32 * focus_pulse),
            size,
        );
    }
}

fn push_top_and_side_surface_outline(
    vertices: &mut Vec<Vertex>,
    rect: Rect,
    stroke_width: f32,
    color: [f32; 4],
    size: PhysicalSize<u32>,
) {
    let stroke_width = stroke_width.max(1.0).min(rect.width).min(rect.height);
    push_rect(
        vertices,
        Rect {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: stroke_width,
        },
        color,
        size,
    );
    push_rect(
        vertices,
        Rect {
            x: rect.x,
            y: rect.y,
            width: stroke_width,
            height: rect.height,
        },
        color,
        size,
    );
    push_rect(
        vertices,
        Rect {
            x: rect.x + rect.width - stroke_width,
            y: rect.y,
            width: stroke_width,
            height: rect.height,
        },
        color,
        size,
    );
}

fn push_single_session_composer_chrome(
    vertices: &mut Vec<Vertex>,
    app: &SingleSessionApp,
    size: PhysicalSize<u32>,
    composer_motion: Option<&ComposerMotionFrame>,
    attachment_chip_motion: Option<&AttachmentChipMotionFrame>,
    layout: Option<SingleSessionLayout>,
) {
    if welcome_status_lane_visible(app) {
        return;
    }

    let typography = single_session_typography();
    let layout = layout.unwrap_or_else(|| single_session_layout_for_app(app, size));
    let target = composer_motion_target(app);
    let visual = composer_motion
        .map(|frame| frame.visual())
        .unwrap_or_else(|| ComposerMotionVisual::settled(target));
    let line_height = layout.metrics.composer_line_height;
    let draft_top = layout.draft_top;
    let content_width = layout.body.width;
    let rect = Rect {
        height: single_session_composer_height(size, layout.metrics, visual),
        ..layout.composer
    };
    if rect.width <= 12.0 || rect.height <= 10.0 {
        return;
    }

    push_single_session_attachment_chips(vertices, app, size, rect, attachment_chip_motion);

    let accent_alpha =
        (0.18 + 0.22 * visual.focus_opacity) * (1.0 - visual.blocked_progress * 0.55);
    push_rounded_rect(
        vertices,
        Rect {
            x: rect.x + 7.0,
            y: rect.y + 7.0,
            width: 3.0,
            height: (rect.height - 14.0).max(1.0),
        },
        2.0,
        with_alpha(COMPOSER_SUBMIT_READY_COLOR, accent_alpha),
        size,
    );

    if visual.placeholder_opacity > 0.001 {
        let prompt_width =
            app.composer_prompt().chars().count() as f32 * typography.code_size * 0.58;
        let rail_width = (content_width * 0.32).clamp(96.0, 260.0);
        push_rounded_rect(
            vertices,
            Rect {
                x: PANEL_TITLE_LEFT_PADDING + prompt_width + 8.0,
                y: draft_top + line_height * 0.50,
                width: rail_width,
                height: 4.0,
            },
            2.0,
            with_alpha(
                COMPOSER_PLACEHOLDER_RAIL_COLOR,
                COMPOSER_PLACEHOLDER_RAIL_COLOR[3] * visual.placeholder_opacity,
            ),
            size,
        );
    }

    if visual.submit_opacity > 0.001 {
        let pill_height = 22.0 * visual.submit_scale.max(0.72);
        let pill_width = 36.0 * visual.submit_scale.max(0.72);
        let pill_x = single_session_content_right(size) - pill_width;
        let pill_y = draft_top + (line_height - pill_height) * 0.5;
        let submit_color = mix_color(
            COMPOSER_SUBMIT_READY_COLOR,
            COMPOSER_SUBMIT_BUSY_COLOR,
            visual.processing_progress,
        );
        push_rounded_rect(
            vertices,
            Rect {
                x: pill_x,
                y: pill_y,
                width: pill_width,
                height: pill_height,
            },
            pill_height * 0.5,
            with_alpha(submit_color, submit_color[3] * visual.submit_opacity),
            size,
        );
        let arrow_alpha = (0.54 + 0.26 * visual.focus_opacity) * visual.submit_opacity;
        let arrow_y = pill_y + pill_height * 0.5 - 1.0;
        push_rect(
            vertices,
            Rect {
                x: pill_x + pill_width * 0.30,
                y: arrow_y,
                width: pill_width * 0.36,
                height: 2.0,
            },
            [1.0, 1.0, 1.0, arrow_alpha],
            size,
        );
        push_rect(
            vertices,
            Rect {
                x: pill_x + pill_width * 0.55,
                y: arrow_y - 4.0,
                width: 2.0,
                height: 10.0,
            },
            [1.0, 1.0, 1.0, arrow_alpha],
            size,
        );
    }
}

fn push_single_session_attachment_chips(
    vertices: &mut Vec<Vertex>,
    app: &SingleSessionApp,
    size: PhysicalSize<u32>,
    composer_rect: Rect,
    attachment_chip_motion: Option<&AttachmentChipMotionFrame>,
) {
    let runs = attachment_chip_runs(&app.pending_images);
    if runs.is_empty() && attachment_chip_motion.is_none_or(|motion| motion.exiting().is_empty()) {
        return;
    }

    for run in runs {
        let visual = attachment_chip_motion
            .and_then(|motion| motion.visual_for_key(run.key))
            .unwrap_or_else(AttachmentChipVisual::settled);
        push_single_session_attachment_chip(vertices, composer_rect, run, visual, false, size);
    }

    if let Some(motion) = attachment_chip_motion {
        for (run, visual) in motion.exiting() {
            push_single_session_attachment_chip(vertices, composer_rect, *run, *visual, true, size);
        }
    }
}

fn push_single_session_attachment_chip(
    vertices: &mut Vec<Vertex>,
    composer_rect: Rect,
    run: AttachmentChipRun,
    visual: AttachmentChipVisual,
    exiting: bool,
    size: PhysicalSize<u32>,
) {
    if visual.opacity <= 0.001 || visual.scale <= 0.05 {
        return;
    }
    let scaled_width = ATTACHMENT_CHIP_WIDTH * visual.scale;
    let scaled_height = ATTACHMENT_CHIP_HEIGHT * visual.scale;
    let step = ATTACHMENT_CHIP_WIDTH + ATTACHMENT_CHIP_GAP;
    let x = composer_rect.x
        + 18.0
        + run.index as f32 * step
        + visual.x_offset_pixels
        + (ATTACHMENT_CHIP_WIDTH - scaled_width) * 0.5;
    let y = (composer_rect.y - ATTACHMENT_CHIP_HEIGHT - 8.0).max(PANEL_BODY_TOP_PADDING + 8.0)
        + visual.y_offset_pixels
        + (ATTACHMENT_CHIP_HEIGHT - scaled_height) * 0.5;
    let max_right = composer_rect.x + composer_rect.width - 16.0;
    if x >= max_right || y >= composer_rect.y + composer_rect.height {
        return;
    }
    let chip_rect = Rect {
        x,
        y,
        width: scaled_width.min((max_right - x).max(0.0)),
        height: scaled_height,
    };
    if chip_rect.width <= 5.0 || chip_rect.height <= 5.0 {
        return;
    }
    let fill = if exiting {
        ATTACHMENT_CHIP_EXIT_COLOR
    } else {
        ATTACHMENT_CHIP_BACKGROUND_COLOR
    };
    push_rounded_rect(
        vertices,
        chip_rect,
        chip_rect.height * 0.5,
        with_alpha(fill, fill[3] * visual.opacity),
        size,
    );
    let accent_width = (chip_rect.height * 0.34).clamp(5.0, 8.0);
    push_rounded_rect(
        vertices,
        Rect {
            x: chip_rect.x + 5.0 * visual.scale,
            y: chip_rect.y + (chip_rect.height - accent_width) * 0.5,
            width: accent_width,
            height: accent_width,
        },
        2.5 * visual.scale,
        with_alpha(
            ATTACHMENT_CHIP_ACCENT_COLOR,
            ATTACHMENT_CHIP_ACCENT_COLOR[3] * visual.opacity,
        ),
        size,
    );
    push_rect(
        vertices,
        Rect {
            x: chip_rect.x + chip_rect.width * 0.45,
            y: chip_rect.y + chip_rect.height * 0.43,
            width: chip_rect.width * 0.32,
            height: 2.0 * visual.scale,
        },
        with_alpha(COMPOSER_FOCUS_RING_COLOR, 0.42 * visual.opacity),
        size,
    );
}

fn push_single_session_stdin_overlay(
    vertices: &mut Vec<Vertex>,
    app: &SingleSessionApp,
    size: PhysicalSize<u32>,
    rendered_body_lines: &[SingleSessionStyledLine],
    stdin_overlay_motion: Option<&StdinOverlayMotionFrame>,
) {
    let settled_current = stdin_overlay_target(app, rendered_body_lines)
        .map(|target| (target, StdinOverlayVisual::settled(target)));
    let current = stdin_overlay_motion
        .and_then(|motion| motion.current)
        .or(settled_current);
    if let Some((target, visual)) = current {
        push_single_session_stdin_overlay_visual(vertices, app, size, target, visual, false);
    }
    if let Some((target, visual)) = stdin_overlay_motion.and_then(|motion| motion.exiting) {
        push_single_session_stdin_overlay_visual(vertices, app, size, target, visual, true);
    }
}

fn push_single_session_stdin_overlay_visual(
    vertices: &mut Vec<Vertex>,
    app: &SingleSessionApp,
    size: PhysicalSize<u32>,
    target: StdinOverlayTarget,
    visual: StdinOverlayVisual,
    exiting: bool,
) {
    if visual.opacity <= 0.001 || visual.scale <= 0.05 {
        return;
    }
    let typography = single_session_typography_for_scale(app.text_scale());
    let line_height = typography.body_size * typography.body_line_height;
    let left = PANEL_TITLE_LEFT_PADDING - 10.0;
    let width = single_session_content_width(size) + 20.0;
    let body_top = single_session_body_top_for_app(app, size);
    let body_bottom = single_session_body_bottom_for_total_lines(app, size, target.line_count);
    let height = (visual.height_lines.max(1.0) * line_height + 18.0)
        .min((body_bottom - body_top + 20.0).max(line_height + 18.0));
    let rect = scaled_rect(
        Rect {
            x: left,
            y: body_top - 8.0 + visual.y_offset_pixels,
            width,
            height,
        },
        visual.scale,
    );
    if rect.width <= 12.0 || rect.height <= 10.0 {
        return;
    }

    let background = if exiting {
        mix_color(
            STDIN_OVERLAY_BACKGROUND_COLOR,
            STDIN_OVERLAY_EXIT_COLOR,
            0.42,
        )
    } else if target.password {
        mix_color(
            STDIN_OVERLAY_BACKGROUND_COLOR,
            [0.990, 0.968, 1.000, 0.660],
            0.36,
        )
    } else {
        STDIN_OVERLAY_BACKGROUND_COLOR
    };
    push_rounded_rect(
        vertices,
        inset_rect(rect, 2.0),
        15.0,
        stdin_overlay_alpha([0.020, 0.035, 0.080, 0.070], visual.opacity),
        size,
    );
    push_rounded_rect(
        vertices,
        rect,
        14.0,
        stdin_overlay_alpha(background, visual.opacity),
        size,
    );
    push_rounded_rect(
        vertices,
        Rect {
            x: rect.x + 7.0,
            y: rect.y + 7.0,
            width: 4.0,
            height: (rect.height - 14.0).max(1.0),
        },
        2.0,
        stdin_overlay_alpha(STDIN_OVERLAY_BORDER_COLOR, visual.opacity * 1.35),
        size,
    );
    push_top_and_side_surface_outline(
        vertices,
        rect,
        1.25,
        stdin_overlay_alpha(STDIN_OVERLAY_BORDER_COLOR, visual.opacity),
        size,
    );

    let input_top = body_top
        + target.input_line_start as f32 * line_height
        + visual.y_offset_pixels
        + line_height * 0.12;
    let input_height = (target.input_line_count as f32 * line_height - line_height * 0.24).max(8.0);
    let input_rect = Rect {
        x: rect.x + 16.0,
        y: input_top.max(rect.y + 8.0).min(rect.y + rect.height - 10.0),
        width: (rect.width - 32.0).max(1.0),
        height: input_height.min((rect.y + rect.height - input_top - 8.0).max(8.0)),
    };
    push_rounded_rect(
        vertices,
        input_rect,
        8.0,
        stdin_overlay_alpha(
            STDIN_OVERLAY_INPUT_RAIL_COLOR,
            visual.opacity * (0.55 + 0.45 * visual.input_glow),
        ),
        size,
    );

    if visual.submit_opacity > 0.001 {
        let pill_width = 44.0;
        let pill_height = 20.0;
        let pill = Rect {
            x: rect.x + rect.width - pill_width - 13.0,
            y: rect.y + rect.height - pill_height - 10.0,
            width: pill_width,
            height: pill_height,
        };
        push_rounded_rect(
            vertices,
            pill,
            pill_height * 0.5,
            stdin_overlay_alpha(
                STDIN_OVERLAY_SUBMIT_COLOR,
                visual.opacity * visual.submit_opacity,
            ),
            size,
        );
        let mark_alpha = visual.opacity * visual.submit_opacity * 0.74;
        push_rect(
            vertices,
            Rect {
                x: pill.x + pill.width * 0.30,
                y: pill.y + pill.height * 0.50,
                width: pill.width * 0.36,
                height: 2.0,
            },
            [1.0, 1.0, 1.0, mark_alpha],
            size,
        );
        push_rect(
            vertices,
            Rect {
                x: pill.x + pill.width * 0.55,
                y: pill.y + pill.height * 0.30,
                width: 2.0,
                height: pill.height * 0.42,
            },
            [1.0, 1.0, 1.0, mark_alpha],
            size,
        );
    }
}

fn stdin_overlay_alpha(mut color: [f32; 4], opacity: f32) -> [f32; 4] {
    color[3] = (color[3] * opacity.clamp(0.0, 1.0)).clamp(0.0, 1.0);
    color
}

#[derive(Clone, Copy, Debug)]
struct InlineWidgetCardStyle {
    background: [f32; 4],
    border: [f32; 4],
    highlight: [f32; 4],
    accent: [f32; 4],
}

#[derive(Clone, Copy, Debug)]
struct InlineWidgetCardLayout {
    card: Rect,
    radius: f32,
    padding_x: f32,
    selection_radius: f32,
    text_left: f32,
    text_top: f32,
    visible_text_right: f32,
    visible_text_bottom: f32,
}

fn inline_widget_card_layout(
    size: PhysicalSize<u32>,
    kind: Option<InlineWidgetKind>,
    typography: &SingleSessionTypography,
    line_count: usize,
    text_width: f32,
    text_top: f32,
    progress: f32,
) -> Option<InlineWidgetCardLayout> {
    inline_widget_card_layout_with_bottom_limit(
        size,
        kind,
        typography,
        line_count,
        text_width,
        text_top,
        progress,
        single_session_draft_top(size),
    )
}

#[allow(clippy::too_many_arguments)]
fn inline_widget_card_layout_with_bottom_limit(
    size: PhysicalSize<u32>,
    kind: Option<InlineWidgetKind>,
    typography: &SingleSessionTypography,
    line_count: usize,
    text_width: f32,
    text_top: f32,
    progress: f32,
    bottom_limit: f32,
) -> Option<InlineWidgetCardLayout> {
    if line_count == 0 {
        return None;
    }

    let progress = progress.clamp(0.0, 1.0);
    if progress <= 0.001 {
        return None;
    }

    let line_height = inline_widget_line_height(kind, typography);
    let padding_x = inline_widget_card_padding_x(kind);
    let padding_y = inline_widget_card_padding_y(kind);
    let text_left = inline_widget_text_left_for_kind(kind, size);
    let text_width = text_width
        .max(line_height * 8.0)
        .min(inline_widget_max_text_width_for_kind(kind, size))
        .max(1.0);
    let text_height = line_count as f32 * line_height;
    let requested_card_height = text_height + padding_y * 2.0;
    let card_y = (text_top - padding_y).max(PANEL_TITLE_TOP_PADDING);
    let draft_top = single_session_draft_top(size);
    let bottom_limit = bottom_limit.min(draft_top);
    let constrained_by_bottom = bottom_limit < draft_top - 0.001;
    let minimum_card_height = if constrained_by_bottom {
        (line_height * 0.72).min(requested_card_height)
    } else {
        (line_height + padding_y * 2.0).min(requested_card_height)
    };
    let available_card_height = if constrained_by_bottom {
        (bottom_limit - card_y).max(1.0)
    } else {
        (bottom_limit - card_y - 8.0).max(minimum_card_height)
    };
    let max_card_height = available_card_height
        .min((size.height as f32 * 0.56).max(line_height * 3.0 + padding_y * 2.0));
    let mut final_card_height = requested_card_height
        .min(max_card_height)
        .max(minimum_card_height.min(max_card_height));
    // When the card cannot fit all rows, quantize its height down to a whole
    // number of text rows so the bottom edge never slices through glyphs.
    if requested_card_height > max_card_height + 0.5 {
        let content_height = (final_card_height - padding_y * 2.0).max(line_height);
        let mut whole_rows = (content_height / line_height).floor().max(1.0);
        // Model picker rows are two-line groups (name + provider meta) after
        // a three-line header; end on a whole group so the last visible model
        // keeps its meta line.
        if kind == Some(InlineWidgetKind::ModelPicker) && whole_rows > 4.0 {
            let header_rows = 3.0;
            let group_rows = ((whole_rows - header_rows) / 2.0).floor() * 2.0;
            whole_rows = header_rows + group_rows.max(2.0);
        }
        final_card_height = whole_rows * line_height + padding_y * 2.0;
    }
    let final_card = Rect {
        x: (text_left - padding_x).max(0.0),
        y: card_y,
        width: text_width + padding_x * 2.0,
        height: final_card_height,
    };
    let start_width = (line_height * 2.0).min(final_card.width);
    let start_height = (line_height * 0.72).min(final_card.height);
    let card = Rect {
        x: final_card.x,
        y: final_card.y,
        width: start_width + (final_card.width - start_width) * progress,
        height: start_height + (final_card.height - start_height) * progress,
    };
    let visible_text_right = (card.x + card.width - padding_x)
        .max(text_left)
        .min(text_left + text_width);
    let visible_text_bottom = (card.y + card.height - padding_y)
        .max(text_top)
        .min(text_top + text_height);

    Some(InlineWidgetCardLayout {
        card,
        radius: inline_widget_card_radius(kind),
        padding_x,
        selection_radius: inline_widget_selection_radius(kind),
        text_left,
        text_top,
        visible_text_right,
        visible_text_bottom,
    })
}

fn inline_widget_line_height(
    kind: Option<InlineWidgetKind>,
    typography: &SingleSessionTypography,
) -> f32 {
    match kind {
        Some(InlineWidgetKind::SlashSuggestions) => {
            inline_widget_font_size(kind, typography) * typography.meta_line_height
        }
        Some(InlineWidgetKind::SessionSwitcher)
        | Some(InlineWidgetKind::HotkeyHelp)
        | Some(InlineWidgetKind::SessionInfo) => {
            inline_widget_font_size(kind, typography) * typography.body_line_height
        }
        _ => typography.body_size * typography.body_line_height,
    }
}

fn inline_widget_text_width_for_lines(
    kind: Option<InlineWidgetKind>,
    lines: &[SingleSessionStyledLine],
    size: PhysicalSize<u32>,
    ui_scale: f32,
) -> f32 {
    let typography = single_session_typography_for_scale(ui_scale);
    // JetBrains Mono advance width is 0.6em; under-estimating clips the
    // longest line at the card right clip edge.
    let average_char_width = inline_widget_font_size(kind, &typography) * 0.6;
    let max_columns = lines
        .iter()
        .map(|line| inline_widget_visual_columns(&line.text))
        .max()
        .unwrap_or_default() as f32;
    (max_columns * average_char_width)
        .ceil()
        .min(inline_widget_max_text_width_for_kind(kind, size))
}

fn inline_widget_font_size(
    kind: Option<InlineWidgetKind>,
    typography: &SingleSessionTypography,
) -> f32 {
    match kind {
        Some(InlineWidgetKind::SlashSuggestions) => {
            (typography.meta_size * SLASH_SUGGESTIONS_INLINE_FONT_SCALE).max(12.0)
        }
        // The switcher splits its card into a narrow rail + preview pane;
        // full body-size text fits so few characters per rail line that
        // headers wrap and push the session rows out of the card.
        Some(InlineWidgetKind::SessionSwitcher) => (typography.body_size * 0.72).max(13.0),
        // Dense reference tables: compact type keeps two-column rows on one
        // line instead of wrapping and breaking the table alignment.
        Some(InlineWidgetKind::HotkeyHelp) | Some(InlineWidgetKind::SessionInfo) => {
            (typography.body_size * 0.72).max(13.0)
        }
        _ => typography.body_size,
    }
}

fn inline_widget_card_padding_x(kind: Option<InlineWidgetKind>) -> f32 {
    match kind {
        Some(InlineWidgetKind::SlashSuggestions) => SLASH_SUGGESTIONS_INLINE_CARD_PADDING_X,
        Some(InlineWidgetKind::ModelPicker) => 18.0,
        _ => INLINE_WIDGET_CARD_PADDING_X,
    }
}

fn inline_widget_card_padding_y(kind: Option<InlineWidgetKind>) -> f32 {
    match kind {
        Some(InlineWidgetKind::SlashSuggestions) => SLASH_SUGGESTIONS_INLINE_CARD_PADDING_Y,
        Some(InlineWidgetKind::ModelPicker) => 11.0,
        _ => INLINE_WIDGET_CARD_PADDING_Y,
    }
}

fn inline_widget_card_radius(kind: Option<InlineWidgetKind>) -> f32 {
    match kind {
        Some(InlineWidgetKind::SlashSuggestions) => SLASH_SUGGESTIONS_INLINE_CARD_RADIUS,
        Some(InlineWidgetKind::ModelPicker) => 30.0,
        _ => INLINE_WIDGET_CARD_RADIUS,
    }
}

fn inline_widget_selection_radius(kind: Option<InlineWidgetKind>) -> f32 {
    match kind {
        Some(InlineWidgetKind::SlashSuggestions) => SLASH_SUGGESTIONS_INLINE_SELECTION_RADIUS,
        Some(InlineWidgetKind::ModelPicker) => 16.0,
        _ => INLINE_WIDGET_SELECTION_RADIUS,
    }
}

fn inline_widget_selection_top_offset(kind: Option<InlineWidgetKind>) -> f32 {
    match kind {
        Some(InlineWidgetKind::SlashSuggestions) => -1.0,
        Some(InlineWidgetKind::ModelPicker) => -4.5,
        _ => -2.0,
    }
}

fn inline_widget_selection_extra_height(kind: Option<InlineWidgetKind>) -> f32 {
    match kind {
        Some(InlineWidgetKind::SlashSuggestions) => 2.0,
        Some(InlineWidgetKind::ModelPicker) => 8.0,
        _ => 2.0,
    }
}

fn inline_widget_selection_background_color(kind: Option<InlineWidgetKind>) -> [f32; 4] {
    match kind {
        Some(InlineWidgetKind::SlashSuggestions) => {
            SLASH_SUGGESTIONS_INLINE_SELECTION_BACKGROUND_COLOR
        }
        Some(InlineWidgetKind::ModelPicker) => MODEL_PICKER_SELECTION_BACKGROUND_COLOR,
        _ => OVERLAY_SELECTION_BACKGROUND_COLOR,
    }
}

fn inline_widget_card_style(kind: Option<InlineWidgetKind>) -> InlineWidgetCardStyle {
    match kind {
        Some(InlineWidgetKind::SlashSuggestions) => InlineWidgetCardStyle {
            background: SLASH_SUGGESTIONS_INLINE_CARD_BACKGROUND_COLOR,
            border: SLASH_SUGGESTIONS_INLINE_CARD_BORDER_COLOR,
            highlight: SLASH_SUGGESTIONS_INLINE_CARD_HIGHLIGHT_COLOR,
            accent: SLASH_SUGGESTIONS_INLINE_CARD_ACCENT_COLOR,
        },
        Some(InlineWidgetKind::ModelPicker) => InlineWidgetCardStyle {
            background: MODEL_PICKER_CARD_BACKGROUND_COLOR,
            border: MODEL_PICKER_CARD_BORDER_COLOR,
            highlight: MODEL_PICKER_CARD_HIGHLIGHT_COLOR,
            accent: MODEL_PICKER_CARD_ACCENT_COLOR,
        },
        _ => InlineWidgetCardStyle {
            background: INLINE_WIDGET_CARD_BACKGROUND_COLOR,
            border: INLINE_WIDGET_CARD_BORDER_COLOR,
            highlight: INLINE_WIDGET_CARD_HIGHLIGHT_COLOR,
            accent: INLINE_WIDGET_CARD_ACCENT_COLOR,
        },
    }
}

fn inline_widget_visual_columns(text: &str) -> usize {
    text.chars()
        .map(|ch| match ch {
            '\t' => 4,
            '\u{200d}' | '\u{fe0e}' | '\u{fe0f}' => 0,
            ch if ch.is_control() => 0,
            ch if is_wide_inline_widget_char(ch) => 2,
            _ => 1,
        })
        .sum()
}

fn is_wide_inline_widget_char(ch: char) -> bool {
    matches!(
        ch as u32,
        0x1100..=0x115F
            | 0x2329..=0x232A
            | 0x2E80..=0xA4CF
            | 0xAC00..=0xD7A3
            | 0xF900..=0xFAFF
            | 0xFE10..=0xFE19
            | 0xFE30..=0xFE6F
            | 0xFF00..=0xFF60
            | 0xFFE0..=0xFFE6
            | 0x1F300..=0x1FAFF
    )
}

fn inline_widget_text_left(size: PhysicalSize<u32>) -> f32 {
    let preferred = PANEL_TITLE_LEFT_PADDING + INLINE_WIDGET_SIDE_GUTTER_EXTRA;
    let responsive_max = (size.width as f32 * 0.18).max(PANEL_TITLE_LEFT_PADDING);
    preferred.min(responsive_max).max(PANEL_TITLE_LEFT_PADDING)
}

fn inline_widget_text_left_for_kind(
    kind: Option<InlineWidgetKind>,
    size: PhysicalSize<u32>,
) -> f32 {
    match kind {
        Some(InlineWidgetKind::SlashSuggestions) => PANEL_TITLE_LEFT_PADDING + 4.0,
        _ => inline_widget_text_left(size),
    }
}

fn inline_widget_max_text_width(size: PhysicalSize<u32>) -> f32 {
    let gutter = inline_widget_text_left(size);
    let available_card_width = (size.width as f32 - gutter * 2.0).max(1.0);
    (available_card_width - INLINE_WIDGET_CARD_PADDING_X * 2.0).max(1.0)
}

fn inline_widget_max_text_width_for_kind(
    kind: Option<InlineWidgetKind>,
    size: PhysicalSize<u32>,
) -> f32 {
    match kind {
        Some(InlineWidgetKind::SlashSuggestions) => {
            let left = inline_widget_text_left_for_kind(kind, size);
            let padding_x = inline_widget_card_padding_x(kind);
            (single_session_content_right(size) - left - padding_x).max(1.0)
        }
        _ => inline_widget_max_text_width(size),
    }
}

pub(crate) fn push_streaming_activity_cue(
    vertices: &mut Vec<Vertex>,
    app: &SingleSessionApp,
    size: PhysicalSize<u32>,
    tick: u64,
    viewport: Option<&SingleSessionBodyViewport>,
    motion: Option<&StreamingActivityCueMotionFrame>,
) {
    let current = if app.streaming_activity_pill_visible() {
        Some(
            motion
                .and_then(StreamingActivityCueMotionFrame::current)
                .unwrap_or_else(StreamingActivityCueVisual::settled),
        )
    } else {
        None
    };
    let exiting = motion.and_then(StreamingActivityCueMotionFrame::exiting);
    if current.is_none() && exiting.is_none() {
        return;
    }

    if let Some(visual) = exiting {
        push_streaming_activity_cue_visual(vertices, app, size, tick, viewport, visual);
    }
    if let Some(visual) = current {
        push_streaming_activity_cue_visual(vertices, app, size, tick, viewport, visual);
    }
}

fn push_streaming_activity_cue_visual(
    vertices: &mut Vec<Vertex>,
    app: &SingleSessionApp,
    size: PhysicalSize<u32>,
    tick: u64,
    viewport: Option<&SingleSessionBodyViewport>,
    visual: StreamingActivityCueVisual,
) {
    if visual.opacity <= 0.001 || visual.scale <= 0.05 {
        return;
    }
    let typography = single_session_typography_for_scale(app.text_scale());
    let viewport = viewport
        .cloned()
        .unwrap_or_else(|| single_session_body_viewport_for_tick(app, size, tick, 0.0));
    let pill_width = (typography.body_size * 2.05).clamp(26.0, 34.0);
    let pill_height = (typography.body_size * 0.82).clamp(11.0, 15.0);
    let layout = single_session_layout_for_total_lines(app, size, viewport.total_lines);
    let activity_lane = layout.activity_lane.unwrap_or(Rect {
        x: PANEL_TITLE_LEFT_PADDING,
        y: layout.body_bottom(),
        width: layout.body.width,
        height: (layout.draft_top - layout.body_bottom()).max(pill_height),
    });
    let cue_y = activity_lane.y + (activity_lane.height - pill_height).max(0.0) * 0.5;
    let cue_x = activity_lane.x;
    let cue_rect = Rect {
        x: cue_x,
        y: cue_y + visual.y_offset_pixels,
        width: pill_width,
        height: pill_height,
    };
    let cue_rect = scaled_rect(cue_rect, visual.scale);
    push_rounded_rect(
        vertices,
        cue_rect,
        pill_height * 0.5,
        with_alpha(
            STREAMING_ACTIVITY_PILL_COLOR,
            STREAMING_ACTIVITY_PILL_COLOR[3] * visual.opacity,
        ),
        size,
    );
    push_rounded_rect_border(
        vertices,
        cue_rect,
        pill_height * 0.5,
        1.0,
        with_alpha(
            STREAMING_ACTIVITY_PILL_BORDER_COLOR,
            STREAMING_ACTIVITY_PILL_BORDER_COLOR[3] * visual.opacity,
        ),
        size,
    );

    let dot_radius = (typography.body_size * 0.105).clamp(1.8, 2.8);
    let dot_y = cue_rect.y + cue_rect.height * 0.50 - dot_radius;
    let dot_gap = dot_radius * 2.35;
    let dot_total_width = dot_radius * 2.0 * 3.0 + dot_gap * 2.0;
    let dot_start_x = cue_rect.x + (cue_rect.width - dot_total_width) * 0.5;
    for dot in 0..3 {
        let dot_phase = ((tick + dot as u64 * 4) % 18) as f32 / 18.0;
        let dot_pulse = 0.5 + 0.5 * (dot_phase * std::f32::consts::TAU).sin();
        let mut dot_color = NATIVE_SPINNER_HEAD_COLOR;
        let base_alpha = if app.streaming_response.is_empty() {
            0.34
        } else {
            0.46
        };
        dot_color[3] = (base_alpha + 0.38 * dot_pulse).clamp(0.30, 0.86) * visual.opacity;
        push_rounded_rect(
            vertices,
            Rect {
                x: dot_start_x + dot as f32 * (dot_radius * 2.0 + dot_gap),
                y: dot_y,
                width: dot_radius * 2.0,
                height: dot_radius * 2.0,
            },
            dot_radius,
            dot_color,
            size,
        );
    }
}

/// Soft breathing cursor at the end of the revealed streaming text. Replaces
/// the standalone activity pill once tokens are flowing, keeping the "alive"
/// cue exactly where the new text appears.
pub(crate) const STREAMING_TAIL_CURSOR_COLOR: [f32; 4] = [0.000, 0.260, 0.720, 0.55];
const STREAMING_TAIL_CURSOR_PULSE_PERIOD_SECONDS: f32 = 1.15;

#[allow(clippy::too_many_arguments)]
pub(crate) fn push_single_session_streaming_tail_cursor(
    vertices: &mut Vec<Vertex>,
    app: &SingleSessionApp,
    size: PhysicalSize<u32>,
    viewport: &SingleSessionBodyViewport,
    streaming_buffer: Option<&Buffer>,
    streaming_start_line: Option<usize>,
    pulse_seconds: f32,
) {
    if !app.has_activity_indicator() || app.streaming_response.is_empty() {
        return;
    }
    let Some(position) =
        streaming_tail_cursor_position(app, size, viewport, streaming_buffer, streaming_start_line)
    else {
        return;
    };

    let typography = single_session_typography_for_scale(app.text_scale());
    let cursor_width = (typography.body_size * 0.46).clamp(5.0, 9.0);
    let cursor_height = (typography.body_size * 0.92).clamp(9.0, 18.0);
    let alpha = if crate::animation::desktop_reduced_motion_enabled() {
        STREAMING_TAIL_CURSOR_COLOR[3]
    } else {
        let phase = (pulse_seconds / STREAMING_TAIL_CURSOR_PULSE_PERIOD_SECONDS).fract();
        let pulse = 0.5 + 0.5 * (phase * std::f32::consts::TAU).sin();
        STREAMING_TAIL_CURSOR_COLOR[3] * (0.45 + 0.55 * pulse)
    };
    let mut color = STREAMING_TAIL_CURSOR_COLOR;
    color[3] = alpha;
    push_rounded_rect(
        vertices,
        Rect {
            x: position.x,
            y: position.y,
            width: cursor_width,
            height: cursor_height,
        },
        cursor_width * 0.45,
        color,
        size,
    );
}

/// Position of the tail cursor: just after the last glyph of the streaming
/// buffer when available, otherwise approximated from the last rendered line.
fn streaming_tail_cursor_position(
    app: &SingleSessionApp,
    size: PhysicalSize<u32>,
    viewport: &SingleSessionBodyViewport,
    streaming_buffer: Option<&Buffer>,
    streaming_start_line: Option<usize>,
) -> Option<CaretPosition> {
    let layout = single_session_layout_for_total_lines(app, size, viewport.total_lines);
    let line_height = layout.metrics.body_line_height;
    let body_top = layout.body.y;
    let body_bottom = layout.body_bottom();
    let typography = single_session_typography_for_scale(app.text_scale());
    let gap = typography.body_size * 0.35;

    if let (Some(buffer), Some(start_line)) = (streaming_buffer, streaming_start_line) {
        let area_top = body_top
            + viewport.top_offset_pixels
            + start_line.saturating_sub(viewport.start_line) as f32 * line_height;
        let mut last: Option<(f32, f32)> = None;
        for run in buffer.layout_runs() {
            last = Some((run.line_w, run.line_top));
        }
        let (line_w, line_top) = last?;
        let y = area_top + line_top + (line_height - typography.body_size) * 0.5;
        if y + typography.body_size * 0.5 > body_bottom || y < body_top - line_height {
            return None;
        }
        return Some(CaretPosition {
            x: (PANEL_TITLE_LEFT_PADDING + line_w + gap)
                .min(single_session_content_right(size) - gap),
            y,
            height: typography.body_size,
        });
    }

    // Fallback: approximate from the last non-blank visible line.
    let (index, line) = viewport
        .lines
        .iter()
        .enumerate()
        .rev()
        .find(|(_, line)| !line.text.trim().is_empty())?;
    let char_width = typography.body_size * 0.52;
    let x = (PANEL_TITLE_LEFT_PADDING + line.text.chars().count() as f32 * char_width + gap)
        .min(single_session_content_right(size) - gap);
    let y = body_top
        + viewport.top_offset_pixels
        + index as f32 * line_height
        + (line_height - typography.body_size) * 0.5;
    if y + typography.body_size * 0.5 > body_bottom {
        return None;
    }
    Some(CaretPosition {
        x,
        y,
        height: typography.body_size,
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SingleSessionTranscriptCardRun {
    pub(crate) line: usize,
    pub(crate) line_count: usize,
    pub(crate) style: SingleSessionLineStyle,
}

#[cfg(test)]
mod tests;
