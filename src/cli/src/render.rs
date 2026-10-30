//! Markdown → 公众号草稿 HTML。
//!
//! 全程逐元素内联样式，不使用 `<style>` 块与 class——微信草稿箱会清洗
//! 样式块与选择器，内联是唯一稳定通道（WeMD 走剪贴板 + DOM 规范化路线，
//! 依赖浏览器环境，与 API 通道不通用；此处只取其「适配微信清洗」的目标）。

use crate::theme::Theme;
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};

pub fn render(markdown: &str, theme: &Theme) -> String {
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TASKLISTS);
    let parser = Parser::new_ext(markdown, opts);

    let mut w = Writer {
        theme,
        out: format!(r#"<section style="{}">"#, wrapper_style(theme)),
        quote_depth: 0,
        in_thead: false,
        in_image_alt: false,
    };
    for ev in parser {
        w.event(ev);
    }
    w.out.push_str("</section>");
    w.out
}

struct Writer<'a> {
    theme: &'a Theme,
    out: String,
    quote_depth: u32,
    in_thead: bool,
    in_image_alt: bool,
}

impl Writer<'_> {
    fn event(&mut self, ev: Event<'_>) {
        match ev {
            Event::Start(tag) => self.start(tag),
            Event::End(tag) => self.end(tag),
            Event::Text(t) => {
                if !self.in_image_alt {
                    self.out.push_str(&escape(&t));
                }
            }
            Event::Code(c) => {
                if !self.in_image_alt {
                    self.out.push_str(&format!(
                        r#"<code style="{}">{}</code>"#,
                        inline_code_style(self.theme),
                        escape(&c)
                    ));
                }
            }
            Event::SoftBreak | Event::HardBreak => self.out.push_str("<br/>"),
            Event::Rule => self
                .out
                .push_str(&format!(r#"<hr style="{}" />"#, hr_style(self.theme))),
            Event::Html(h) | Event::InlineHtml(h) => self.out.push_str(&h),
            Event::TaskListMarker(done) => {
                self.out
                    .push_str(if done { "✅&nbsp;" } else { "⬜&nbsp;" })
            }
            Event::FootnoteReference(name) => self
                .out
                .push_str(&format!(r#"<sup>[{}]</sup>"#, escape(&name))),
            _ => {}
        }
    }

    fn start(&mut self, tag: Tag<'_>) {
        match tag {
            Tag::Paragraph => {
                let style = if self.quote_depth > 0 {
                    paragraph_in_quote_style(self.theme)
                } else {
                    paragraph_style(self.theme)
                };
                self.out.push_str(&format!(r#"<p style="{style}">"#));
            }
            Tag::Heading { level, .. } => {
                let (name, size) = heading_spec(level, self.theme);
                self.out.push_str(&format!(
                    r#"<{name} style="{}">"#,
                    heading_style(self.theme, size)
                ));
            }
            Tag::BlockQuote(_) => {
                self.quote_depth += 1;
                self.out.push_str(&format!(
                    r#"<blockquote style="{}">"#,
                    blockquote_style(self.theme)
                ));
            }
            Tag::CodeBlock(_) => self.out.push_str(&format!(
                r#"<pre style="{}"><code style="{}">"#,
                pre_style(self.theme),
                pre_code_style(self.theme)
            )),
            Tag::List(None) => self
                .out
                .push_str(&format!(r#"<ul style="{}">"#, list_style(self.theme))),
            Tag::List(Some(start)) => self.out.push_str(&format!(
                r#"<ol style="{}" start="{start}">"#,
                list_style(self.theme)
            )),
            Tag::Item => self
                .out
                .push_str(&format!(r#"<li style="{}">"#, li_style(self.theme))),
            Tag::Emphasis => self.out.push_str(r#"<em style="font-style:italic;">"#),
            Tag::Strong => self
                .out
                .push_str(&format!(r#"<strong style="{}">"#, strong_style(self.theme))),
            Tag::Strikethrough => self
                .out
                .push_str(r#"<del style="color:#9ca3af;text-decoration:line-through;">"#),
            Tag::Link { dest_url, .. } => self.out.push_str(&format!(
                r#"<a style="{}" href="{}">"#,
                link_style(self.theme),
                escape_attr(&dest_url)
            )),
            Tag::Image { dest_url, .. } => {
                self.in_image_alt = true;
                self.out.push_str(&format!(
                    r#"<img style="{}" src="{}" alt="" />"#,
                    img_style(self.theme),
                    escape_attr(&dest_url)
                ));
            }
            Tag::Table { .. } => self
                .out
                .push_str(&format!(r#"<table style="{}">"#, table_style(self.theme))),
            Tag::TableHead => {
                self.in_thead = true;
                self.out.push_str("<thead>");
            }
            Tag::TableRow => self.out.push_str("<tr>"),
            Tag::TableCell => {
                let name = if self.in_thead { "th" } else { "td" };
                self.out.push_str(&format!(
                    r#"<{name} style="{}">"#,
                    cell_style(self.theme, self.in_thead)
                ));
            }
            _ => {}
        }
    }

    fn end(&mut self, tag: TagEnd) {
        match tag {
            TagEnd::Paragraph => self.out.push_str("</p>"),
            TagEnd::Heading(level) => {
                let (name, _) = heading_spec(level, self.theme);
                self.out.push_str(&format!("</{name}>"));
            }
            TagEnd::BlockQuote(_) => {
                self.quote_depth = self.quote_depth.saturating_sub(1);
                self.out.push_str("</blockquote>");
            }
            TagEnd::CodeBlock => self.out.push_str("</code></pre>"),
            TagEnd::List(_) => self.out.push_str("</ul>"),
            TagEnd::Item => self.out.push_str("</li>"),
            TagEnd::Emphasis => self.out.push_str("</em>"),
            TagEnd::Strong => self.out.push_str("</strong>"),
            TagEnd::Strikethrough => self.out.push_str("</del>"),
            TagEnd::Link => self.out.push_str("</a>"),
            TagEnd::Image => self.in_image_alt = false,
            TagEnd::Table => self.out.push_str("</table>"),
            TagEnd::TableHead => {
                self.in_thead = false;
                self.out.push_str("</thead>");
            }
            TagEnd::TableRow => self.out.push_str("</tr>"),
            TagEnd::TableCell => {
                let name = if self.in_thead { "th" } else { "td" };
                self.out.push_str(&format!("</{name}>"));
            }
            _ => {}
        }
    }
}

// ── 样式表（全部来自主题参数，逐元素内联）──────────────────

fn wrapper_style(t: &Theme) -> String {
    format!(
        "font-family:{};font-size:{}px;color:{};line-height:{};letter-spacing:0.2px;",
        t.font_family, t.font_size, t.text_color, t.line_height
    )
}

fn paragraph_style(t: &Theme) -> String {
    format!(
        "margin:0 0 {}px;line-height:{};font-size:{}px;text-align:justify;",
        t.paragraph_spacing, t.line_height, t.font_size
    )
}

fn paragraph_in_quote_style(_t: &Theme) -> String {
    "margin:0 0 8px;line-height:1.7;font-size:15px;".to_string()
}

fn heading_spec(level: HeadingLevel, t: &Theme) -> (&'static str, u32) {
    match level {
        HeadingLevel::H1 => ("h1", t.h2_size + 3),
        HeadingLevel::H2 => ("h2", t.h2_size),
        HeadingLevel::H3 => ("h3", t.h3_size),
        HeadingLevel::H4 => ("h4", t.h4_size),
        HeadingLevel::H5 => ("h5", t.h4_size),
        HeadingLevel::H6 => ("h6", t.h4_size),
    }
}

fn heading_style(t: &Theme, size: u32) -> String {
    format!(
        "color:{};font-size:{}px;font-weight:{};line-height:1.4;margin:{}px 0 {}px;",
        t.heading_color, size, t.heading_weight, t.heading_top_spacing, t.heading_bottom_spacing
    )
}

fn blockquote_style(t: &Theme) -> String {
    format!(
        "margin:0 0 {}px;padding:12px 16px;border-left:3px solid {};background:{};color:{};",
        t.paragraph_spacing, t.quote_border_color, t.quote_bg, t.quote_color
    )
}

fn list_style(t: &Theme) -> String {
    format!("margin:0 0 {}px;padding-left:22px;", t.paragraph_spacing)
}

fn li_style(t: &Theme) -> String {
    format!("margin:0 0 8px;line-height:{};", t.line_height)
}

fn strong_style(t: &Theme) -> String {
    format!("color:{};font-weight:700;", t.strong_color)
}

fn link_style(t: &Theme) -> String {
    format!("color:{};text-decoration:none;", t.link_color)
}

fn img_style(_t: &Theme) -> String {
    "max-width:100%;height:auto;display:block;margin:0 auto 12px;".to_string()
}

fn inline_code_style(t: &Theme) -> String {
    format!(
        "background:{};color:{};font-family:{};font-size:14px;padding:2px 5px;border-radius:4px;",
        t.code_bg, t.code_color, t.code_font_family
    )
}

fn pre_style(t: &Theme) -> String {
    format!(
        "margin:0 0 {}px;padding:14px 16px;background:{};border-radius:6px;overflow-x:auto;white-space:pre-wrap;word-break:break-all;",
        t.paragraph_spacing, t.code_bg
    )
}

fn pre_code_style(t: &Theme) -> String {
    format!(
        "background:none;color:{};font-family:{};font-size:14px;line-height:1.7;",
        t.text_color, t.code_font_family
    )
}

fn table_style(t: &Theme) -> String {
    format!(
        "border-collapse:collapse;margin:0 0 {}px;width:100%;",
        t.paragraph_spacing
    )
}

fn cell_style(t: &Theme, is_head: bool) -> String {
    if is_head {
        format!(
            "border:1px solid #e5e7eb;padding:8px 10px;font-size:14px;background:#f6f8f7;color:{};font-weight:600;",
            t.heading_color
        )
    } else {
        "border:1px solid #e5e7eb;padding:8px 10px;font-size:14px;".to_string()
    }
}

fn hr_style(t: &Theme) -> String {
    format!(
        "border:none;border-top:1px solid #e5e7eb;margin:{}px 0;",
        t.paragraph_spacing
    )
}

// ── 转义 ──────────────────────────────────────────────

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn escape_attr(s: &str) -> String {
    escape(s).replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_blocks_with_inline_styles_only() {
        let md =
            "# 标题\n\n正文一段。\n\n## 二级\n\n- 列表项\n\n> 引用\n\n```rust\nlet x = 1;\n```\n";
        let html = render(md, &Theme::default());
        assert!(html.contains("<h2 style=\""));
        assert!(html.contains("<blockquote style=\""));
        assert!(html.contains("<pre style=\""));
        assert!(html.contains("color:#3f3f3f"));
        // 微信会清洗样式块：产物里不允许出现 <style> 与 class
        assert!(!html.contains("<style"));
        assert!(!html.contains("class="));
    }

    #[test]
    fn body_text_is_untouched() {
        let md = "排版归排版，正文一个字不改。\n";
        let html = render(md, &Theme::default());
        assert!(html.contains("排版归排版，正文一个字不改。"));
    }

    #[test]
    fn escapes_raw_text_but_not_structure() {
        let md = "数学 < b & c > 公式\n";
        let html = render(md, &Theme::default());
        assert!(html.contains("数学 &lt; b &amp; c &gt; 公式"));
    }
}
