use crate::frontend::style;
use egui::{
    Color32, FontId, Stroke, TextFormat, TextStyle, Ui,
    text::{LayoutJob, LayoutSection},
};
use regex::Regex;

pub struct NotesSubsystem {
    pub text: String,
    pub requests_focus: bool,
}

impl NotesSubsystem {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            requests_focus: false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum DelimiterType {
    Nothing,
    LeadingTrailing,
    LeadingMany(usize),
}

pub fn get_notes_textedit_layouter(
    ctx: &egui::Context,
) -> impl Fn(&Ui, &str, f32) -> std::sync::Arc<egui::Galley> {
    // I am convinced that only the rust analyzer can understand the return type of this function
    |ui: &Ui, text: &str, wrap_width: f32| {
        // define text stlyes for tags
        let normal_font = ctx.style().text_styles[&TextStyle::Body].clone();
        let normal = TextFormat {
            font_id: normal_font.clone(),
            italics: false,
            underline: Default::default(),
            strikethrough: Default::default(),
            background: Default::default(),
            color: ctx.style().visuals.widgets.noninteractive.fg_stroke.color,
            extra_letter_spacing: Default::default(),
            line_height: Default::default(),
            valign: Default::default(),
        };
        let heading = TextFormat { ..normal.clone() };

        let italic = TextFormat {
            italics: true,
            color: style::COLOR_NOTES_ITALIC,
            ..normal.clone()
        };

        let underlined = TextFormat {
            color: style::COLOR_NOTES_UNDERLINED,
            underline: Stroke {
                width: 1.,
                color: style::COLOR_NOTES_UNDERLINED,
            }, // underline instead as bold doesn't seem possible
            ..normal.clone()
        };

        let code = TextFormat {
            font_id: FontId::monospace(normal_font.size - 1.),
            background: egui::Color32::from_black_alpha(200),
            color: egui::Color32::LIGHT_YELLOW,
            ..normal.clone()
        };

        // TODO: optimize -> compile regex once statically // once_cell::sync::Lazy
        let code_re = Regex::new(r"`+[^`]+?`+").unwrap();
        let italic_re = Regex::new(r"\*[^\n]+?\*").unwrap();
        let underline_re = Regex::new(r"_[^\n]+?_").unwrap();
        let head_re = Regex::new(r"(?m)^(#{1,6})[ \t]+(\S[^\r\n]*)$").unwrap(); // (?m)^(#{1,6})\s+(.+)$

        let mut ranges = Vec::new();

        // headings need size and color per level => manually build format
        for cap in head_re.captures_iter(text) {
            let lvl = cap[1].len(); // number of '#' chars
            let size = match lvl {
                1 => 24.0,
                2 => 20.0,
                3 => 18.0,
                4 => 16.0,
                5 => 15.0,
                _ => 14.0,
            };
            let color = match lvl {
                1 => style::COLOR_HEADING_1,
                2 => style::COLOR_HEADING_2,
                3 => style::COLOR_HEADING_3,
                4 => style::COLOR_HEADING_4,
                5 => style::COLOR_HEADING_5,
                6 => style::COLOR_HEADING_6,
                _ => style::COLOR_TEXT,
            };
            let mut fmt = heading.clone();
            fmt.font_id.size = size;
            fmt.color = color;

            let m = cap.get(0).unwrap();
            ranges.push((m.start(), m.end(), fmt, DelimiterType::LeadingMany(lvl)));
        }

        // helper to push every capture from a regex
        let mut push_re = |re: &Regex, fmt: TextFormat| {
            for m in re.find_iter(text) {
                ranges.push((
                    m.start(),
                    m.end(),
                    fmt.clone(),
                    DelimiterType::LeadingTrailing,
                ));
            }
        };

        // now inline styles
        push_re(&code_re, code);
        push_re(&italic_re, italic);
        push_re(&underline_re, underlined);

        // Sort by start index
        ranges.sort_by_key(|r| r.0);

        let mut job = LayoutJob::default();

        // init with full text -> avoids layout-& source-desyncs
        job.append(text, 0.0, normal.clone());
        // clear original text styling sections so i can style on my own, but keep text
        job.sections.clear();

        // Build from non-overlapping sections
        let mut last = 0;
        for (start, end, fmt, del_type) in ranges {
            if start < last {
                if end < last {
                    // fully contained in some previous element
                    continue;
                } else {
                    // overlapping with some previous element -> render as normal text
                    push_style(normal.clone(), last, end, DelimiterType::Nothing)
                        .iter()
                        .for_each(|section| {
                            job.sections.push(section.clone());
                        });

                    last = end;
                    continue;
                }
            } else {
                // render text since last tag
                push_style(normal.clone(), last, start, DelimiterType::Nothing)
                    .iter()
                    .for_each(|section| {
                        job.sections.push(section.clone());
                    });

                //render current tag
                push_style(fmt.clone(), start, end, del_type)
                    .iter()
                    .for_each(|section| {
                        job.sections.push(section.clone());
                    });

                last = end;
            }
        }
        job.sections.push(LayoutSection {
            byte_range: last..text.len(),
            format: normal.clone(),
            leading_space: 0.0,
        });

        job.wrap.max_width = wrap_width;

        ui.fonts(|f| f.layout_job(job))
    }
}

fn push_style(
    format: TextFormat,
    start: usize,
    last: usize,
    delimiter_type: DelimiterType,
) -> Vec<LayoutSection> {
    let mut sections = Vec::new();
    let delimiter = TextFormat {
        color: Color32::from_white_alpha(10),
        italics: false,
        underline: Stroke {
            width: 0.,
            color: Color32::TRANSPARENT,
        },
        ..format.clone()
    };

    match delimiter_type {
        DelimiterType::Nothing => sections.push(LayoutSection {
            byte_range: start..last,
            format,
            leading_space: 0.0,
        }),
        DelimiterType::LeadingTrailing => {
            sections.push(LayoutSection {
                leading_space: 0.0,
                byte_range: start..(start + 1),
                format: delimiter.clone(),
            });
            sections.push(LayoutSection {
                leading_space: 0.0,
                byte_range: (start + 1)..(last - 1),
                format,
            });
            sections.push(LayoutSection {
                leading_space: 0.0,
                byte_range: (last - 1)..last,
                format: delimiter.clone(),
            });
        }
        DelimiterType::LeadingMany(many) => {
            sections.push(LayoutSection {
                leading_space: 0.0,
                byte_range: start..(start + many),
                format: delimiter.clone(),
            });
            sections.push(LayoutSection {
                leading_space: 0.0,
                byte_range: (start + many)..(last),
                format,
            });
        }
    };

    sections
}
