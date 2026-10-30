//! Markdown → 公众号草稿 HTML。样式随主题整体注入，正文一个字不改。

use crate::theme::Theme;
use pulldown_cmark::{Options, Parser, html};

/// 把 Markdown 渲染成可直接放进公众号草稿 content 字段的 HTML 片段。
///
/// v0 采用 `<style>` + 容器内联的组合：基础字体走容器内联样式，
/// 元素级排版走 `<style>` 块。若日后实测微信侧剥掉样式块，
/// 再退化为逐元素内联（改这一处即可，调用方不受影响）。
pub fn render(markdown: &str, theme: &Theme) -> String {
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TASKLISTS);
    let parser = Parser::new_ext(markdown, opts);
    let mut body = String::new();
    html::push_html(&mut body, parser);

    let head = format!(
        r#"<section class="qtm-article" style="font-family:{font_family};font-size:{font_size}px;color:{text_color};line-height:{line_height};letter-spacing:0.2px;"><style>{css}</style>"#,
        font_family = theme.font_family,
        font_size = theme.font_size,
        text_color = theme.text_color,
        line_height = theme.line_height,
        css = css(theme),
    );
    format!("{head}{body}</section>")
}

fn css(t: &Theme) -> String {
    format!(
        ".qtm-article p{{margin:0 0 {p}px;word-break:break-word;}}
.qtm-article h2{{color:{hc};font-size:{h2}px;font-weight:{hw};line-height:1.4;margin:{htop}px 0 {hbot}px;}}
.qtm-article h3{{color:{hc};font-size:{h3}px;font-weight:{hw};line-height:1.45;margin:{htop}px 0 {hbot}px;}}
.qtm-article h4{{color:{hc};font-size:{h4}px;font-weight:{hw};line-height:1.45;margin:{htop}px 0 {hbot}px;}}
.qtm-article strong{{color:{strong};font-weight:700;}}
.qtm-article a{{color:{link};text-decoration:none;}}
.qtm-article blockquote{{margin:0 0 {p}px;padding:12px 16px;border-left:3px solid {qb};background:{qbg};color:{qcolor};}}
.qtm-article blockquote p{{margin:0 0 8px;}}
.qtm-article blockquote p:last-child{{margin-bottom:0;}}
.qtm-article ul,.qtm-article ol{{margin:0 0 {p}px;padding-left:22px;}}
.qtm-article li{{margin:0 0 8px;}}
.qtm-article pre{{margin:0 0 {p}px;padding:14px 16px;background:{cbg};border-radius:6px;overflow-x:auto;}}
.qtm-article pre code{{background:none;color:{ccolor};font-family:{cfont};font-size:14px;line-height:1.7;}}
.qtm-article code{{background:{cbg};color:{ccolor};font-family:{cfont};font-size:14px;padding:2px 5px;border-radius:4px;}}
.qtm-article img{{max-width:100%;height:auto;display:block;margin:0 auto 12px;}}
.qtm-article hr{{border:none;border-top:1px solid #e5e7eb;margin:{p}px 0;}}
.qtm-article table{{border-collapse:collapse;margin:0 0 {p}px;width:100%;}}
.qtm-article th,.qtm-article td{{border:1px solid #e5e7eb;padding:8px 10px;font-size:14px;}}
.qtm-article th{{background:#f6f8f7;color:{hc};}}",
        p = t.paragraph_spacing,
        hc = t.heading_color,
        h2 = t.h2_size,
        h3 = t.h3_size,
        h4 = t.h4_size,
        hw = t.heading_weight,
        htop = t.heading_top_spacing,
        hbot = t.heading_bottom_spacing,
        strong = t.strong_color,
        link = t.link_color,
        qb = t.quote_border_color,
        qbg = t.quote_bg,
        qcolor = t.quote_color,
        cbg = t.code_bg,
        ccolor = t.code_color,
        cfont = t.code_font_family,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_blocks_with_theme() {
        let md =
            "# 标题\n\n正文一段。\n\n## 二级\n\n- 列表项\n\n> 引用\n\n```rust\nlet x = 1;\n```\n";
        let html = render(md, &Theme::default());
        assert!(html.contains("<h1"));
        assert!(html.contains("<h2"));
        assert!(html.contains("<blockquote>"));
        assert!(html.contains("<pre>"));
        assert!(html.contains("qtm-article"));
        assert!(html.contains("#3f3f3f"));
    }

    #[test]
    fn body_text_is_untouched() {
        let md = "排版归排版，正文一个字不改。\n";
        let html = render(md, &Theme::default());
        assert!(html.contains("排版归排版，正文一个字不改。"));
    }
}
