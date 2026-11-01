//! 公众号历史文章 HTML → Markdown（取回后编辑用）。
//!
//! 微信返回的正文是重内联样式 HTML：嵌套 section、span 带 style、
//! 图片走 data-src。转换只取结构与文本，样式一律丢弃；视频/音频卡片
//! 等富组件降级为占位文本。

/// HTML → Markdown。
pub fn html_to_markdown(html: &str) -> String {
    let mut w = Writer::default();
    let bytes = html.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'<'
            && let Some(next) = w.tag_at(html, i)
        {
            i = next;
            continue;
        }
        // 文本（或不成标签的裸 '<'）
        let end = html[i..].find('<').map(|p| i + p).unwrap_or(bytes.len());
        if w.skipping.is_none() {
            w.push_text(&html[i..end]);
        }
        i = end.max(i + 1);
    }
    w.finish()
}

#[derive(Default)]
struct Writer {
    out: String,
    /// > 引用嵌套深度
    bq: usize,
    /// 列表栈：(有序, 计数)
    lists: Vec<(bool, u32)>,
    /// pre 原样区（不做内联标记）
    pre: bool,
    /// 跳过中的标签名（script/style 等）
    skipping: Option<String>,
    /// 挂起的链接 href（`[` 已输出，等 `](url)`）
    link: Option<String>,
}

impl Writer {
    /// 处理从 `pos`（`<` 处）开始的标签，返回下一个解析位置；不是标签返回 None。
    fn tag_at(&mut self, s: &str, pos: usize) -> Option<usize> {
        let rest = &s[pos..];
        if rest.starts_with("<!--") {
            let end = rest.find("-->").map(|p| pos + p + 3)?;
            return Some(end);
        }
        let close = rest.find('>')? + pos;
        let raw = &s[pos + 1..close];
        let body = raw.strip_prefix('/').unwrap_or(raw);
        let name_end = body
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '-'))
            .unwrap_or(body.len());
        let name = body[..name_end].to_ascii_lowercase();
        let attrs = &body[name_end..];
        let closing = raw.starts_with('/');
        let next = close + 1;

        if self.skipping.is_some() {
            if closing && self.skipping.as_deref() == Some(name.as_str()) {
                self.skipping = None;
            }
            return Some(next);
        }
        // script/style/head：整段跳过
        if !closing && matches!(name.as_str(), "script" | "style" | "head") {
            self.skipping = Some(name);
            return Some(next);
        }
        // 富组件卡片：降级占位
        if !closing
            && matches!(
                name.as_str(),
                "mpvideosnap" | "mpvoice" | "iframe" | "mp-common-qqmusic" | "mp-common-snsvideo"
            )
        {
            if !self.out.is_empty() {
                self.end_block();
            }
            let ph = match name.as_str() {
                "mpvoice" => "【音频卡片】",
                "mpvideosnap" | "iframe" | "mp-common-snsvideo" => "【视频卡片】",
                _ => "【卡片】",
            };
            self.push_line_text(ph);
            // 跳到配对结束标签
            let lower = s[next..].to_ascii_lowercase();
            if let Some(p) = lower.find(&format!("</{name}"))
                && let Some(e) = s[next + p..].find('>')
            {
                return Some(next + p + e + 1);
            }
            return Some(next);
        }
        if closing && name.is_empty() {
            return Some(next); // 裸 </> 之类
        }

        match (name.as_str(), closing) {
            ("br", false) => {
                self.out.push('\n');
            }
            ("hr", false) => {
                self.end_block();
                self.push_line_text("---");
                self.end_block();
            }
            ("pre", false) => {
                self.end_block();
                self.out.push_str("```\n");
                self.pre = true;
            }
            ("pre", true) => {
                if !self.out.ends_with('\n') {
                    self.out.push('\n');
                }
                self.out.push_str("```\n");
                self.pre = false;
            }
            ("code", false) if !self.pre => self.out.push('`'),
            ("code", true) if !self.pre => self.out.push('`'),
            ("strong" | "b", false) if !self.pre => self.out.push_str("**"),
            ("strong" | "b", true) if !self.pre => self.out.push_str("**"),
            ("em" | "i", false) if !self.pre => self.out.push('*'),
            ("em" | "i", true) if !self.pre => self.out.push('*'),
            ("del" | "s" | "strike", false) if !self.pre => self.out.push_str("~~"),
            ("del" | "s" | "strike", true) if !self.pre => self.out.push_str("~~"),
            ("a", false) if !self.pre => {
                self.end_inline();
                self.out.push('[');
                self.link = Some(attr(attrs, "href").unwrap_or_default());
            }
            ("a", true) if !self.pre => {
                let href = self.link.take().unwrap_or_default();
                if href.is_empty() {
                    self.out.push(']');
                } else {
                    self.out.push_str(&format!("]({href})"));
                }
            }
            ("img", false) if !self.pre => {
                self.end_inline();
                let src = attr(attrs, "data-src").or_else(|| attr(attrs, "src"));
                if let Some(src) = src.filter(|s| !s.is_empty()) {
                    let alt = attr(attrs, "alt").unwrap_or_default();
                    self.push_line_text(&format!("![{alt}]({src})"));
                }
            }
            ("h1" | "h2" | "h3" | "h4" | "h5" | "h6", false) if !self.pre => {
                self.end_block();
                let n = name
                    .get(1..)
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(2)
                    .min(4);
                self.push_line_text(&"#".repeat(n));
                self.out.push(' ');
            }
            ("h1" | "h2" | "h3" | "h4" | "h5" | "h6", true) => self.end_block(),
            ("blockquote", false) => {
                self.end_block();
                self.bq += 1;
            }
            ("blockquote", true) => {
                self.end_block();
                self.bq = self.bq.saturating_sub(1);
            }
            ("ul", false) => {
                self.end_block();
                self.lists.push((false, 0));
            }
            ("ol", false) => {
                self.end_block();
                let start = attr(attrs, "start")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(1);
                self.lists.push((true, start - 1));
            }
            ("ul" | "ol", true) => {
                self.end_block();
                self.lists.pop();
            }
            ("li", false) => {
                self.end_block();
                let marker = self.lists.last_mut().map(|(ordered, n)| {
                    if *ordered {
                        *n += 1;
                        format!("{n}. ")
                    } else {
                        "- ".to_string()
                    }
                });
                if let Some(marker) = marker {
                    self.push_line_text(&marker);
                }
            }
            ("li", true) => self.end_block(),
            ("table", false) => self.end_block(),
            ("table", true) => self.end_block(),
            ("tr", false) | ("tr", true) => self.end_block(),
            ("td" | "th", false) => {
                self.end_block();
                self.push_line_text("| ");
            }
            ("td" | "th", true) => {}
            // 段落级容器：闭合即断块
            (
                "p" | "section" | "div" | "article" | "figure" | "figcaption" | "header" | "footer"
                | "main" | "aside",
                true,
            ) => self.end_block(),
            // 其余标签（span、样式容器、微信私有标签等）一律忽略
            _ => {}
        }
        Some(next)
    }

    fn push_text(&mut self, text: &str) {
        if self.pre {
            self.out.push_str(text);
            return;
        }
        let decoded = decode_entities(text);
        let cleaned: String = decoded
            .chars()
            .map(|c| match c {
                '\u{a0}' => ' ',
                c if c.is_control() && c != '\n' && c != '\t' => ' ',
                c => c,
            })
            .collect();
        if cleaned.trim().is_empty() {
            // 纯空白：标签间的空格不产出，避免行首杂散空格
            return;
        }
        self.push_line_text(&cleaned);
    }

    /// 输出一段行内文本，行首按上下文补引用/列表前缀。
    fn push_line_text(&mut self, text: &str) {
        let at_line_start = self.out.is_empty() || self.out.ends_with('\n');
        if at_line_start {
            for _ in 0..self.bq {
                self.out.push_str("> ");
            }
        }
        self.out.push_str(text);
    }

    /// 行内标记（加粗等）闭合处不需要断块，只是直接写。
    fn end_inline(&mut self) {}

    /// 断块：确保空行分隔。
    fn end_block(&mut self) {
        if self.out.is_empty() {
            return;
        }
        while !self.out.ends_with("\n\n") {
            self.out.push('\n');
        }
    }

    fn finish(mut self) -> String {
        // 清理：行尾空白、连续空行压成一个空行
        let mut lines: Vec<String> = Vec::new();
        for raw in self.out.lines() {
            let line = raw.trim_end();
            if line.is_empty() {
                if lines.last().is_some_and(|l: &String| !l.is_empty()) {
                    lines.push(String::new());
                }
            } else {
                lines.push(line.to_string());
            }
        }
        while lines.last().is_some_and(|l| l.is_empty()) {
            lines.pop();
        }
        let mut out = lines.join("\n");
        if !out.ends_with('\n') {
            out.push('\n');
        }
        self.out = out;
        self.out
    }
}

/// 标签属性取值（去引号 + 实体解码：正文里的 `&amp;` 要还原回 `&`）。
fn attr(attrs: &str, key: &str) -> Option<String> {
    let re = regex::Regex::new(&format!(
        r#"(?i)\b{key}\s*=\s*("([^"]*)"|'([^']*)'|([^\s">]+))"#
    ))
    .ok()?;
    let caps = re.captures(attrs)?;
    caps.get(2)
        .or_else(|| caps.get(3))
        .or_else(|| caps.get(4))
        .map(|m| decode_entities(m.as_str()))
}

/// HTML 实体解码（常见集 + 数字引用）。
fn decode_entities(s: &str) -> String {
    if !s.contains('&') {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(pos) = rest.find('&') {
        out.push_str(&rest[..pos]);
        let tail = &rest[pos..];
        let end = tail.find(';').map(|p| p + 1).unwrap_or(1.min(tail.len()));
        let entity = &tail[..end];
        match entity {
            "&nbsp;" => out.push(' '),
            "&amp;" => out.push('&'),
            "&lt;" => out.push('<'),
            "&gt;" => out.push('>'),
            "&quot;" => out.push('"'),
            "&#39;" | "&apos;" => out.push('\''),
            "&hellip;" => out.push('…'),
            "&mdash;" => out.push('—'),
            _ if entity.starts_with("&#x") || entity.starts_with("&#X") => {
                if let Some(c) = entity[3..entity.len() - 1]
                    .parse::<u32>()
                    .ok()
                    .and_then(char::from_u32)
                {
                    out.push(c);
                } else {
                    out.push_str(entity);
                }
            }
            _ if entity.starts_with("&#") => {
                if let Some(c) = entity[2..entity.len() - 1]
                    .parse::<u32>()
                    .ok()
                    .and_then(char::from_u32)
                {
                    out.push(c);
                } else {
                    out.push_str(entity);
                }
            }
            _ => out.push_str(entity),
        }
        rest = &tail[end..];
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wechat_sections_spans_and_styles_are_flattened() {
        let html = r#"<section style="margin:0"><p style="color:#3f3f3f"><span style="font-size:16px">尊敬的各位：</span></p><p style="text-align:justify;">正文<strong>重点</strong>一段。<em>强调</em>。</p></section>"#;
        let md = html_to_markdown(html);
        assert!(md.contains("尊敬的各位："));
        assert!(md.contains("**重点**"));
        assert!(md.contains("*强调*"));
        assert!(!md.contains("style="));
        assert!(!md.contains('<'), "不应残留标签");
    }

    #[test]
    fn images_use_data_src() {
        let html = r#"<img data-src="https://mmbiz.qpic.cn/x/y/0" src="data:image/gif;base64,xxx" alt="配图" />"#;
        let md = html_to_markdown(html);
        assert_eq!(md.trim(), "![配图](https://mmbiz.qpic.cn/x/y/0)");
    }

    #[test]
    fn lists_headings_and_quotes() {
        let html = r#"<h2>小节</h2><ul><li>第一条</li><li>第二条</li></ul><blockquote><p>引用一句</p></blockquote><p>尾段</p>"#;
        let md = html_to_markdown(html);
        assert!(md.contains("## 小节"));
        assert!(md.contains("- 第一条"));
        assert!(md.contains("- 第二条"));
        assert!(md.contains("> 引用一句"));
        assert!(md.contains("尾段"));
    }

    #[test]
    fn ordered_list_numbering() {
        let html = r#"<ol><li>甲</li><li>乙</li></ol>"#;
        let md = html_to_markdown(html);
        assert!(md.contains("1. 甲"));
        assert!(md.contains("2. 乙"));
    }

    #[test]
    fn inline_code_links_and_entities() {
        let html =
            r#"<p><code>let x</code> 与 <a href="https://e.com">链接</a>，A&nbsp;B &amp; C</p>"#;
        let md = html_to_markdown(html);
        assert!(md.contains("`let x`"));
        assert!(md.contains("[链接](https://e.com)"));
        assert!(md.contains("A B & C"));
    }

    #[test]
    fn video_card_becomes_placeholder() {
        let html = r#"<p>开头</p><mpvideosnap data-vid="x"></mpvideosnap><p>结尾</p>"#;
        let md = html_to_markdown(html);
        assert!(md.contains("开头"));
        assert!(md.contains("【视频卡片】"));
        assert!(md.contains("结尾"));
        assert!(!md.contains("mpvideosnap"));
    }

    #[test]
    fn script_and_style_dropped() {
        let html = r#"<style>p{color:red}</style><p>正文</p><script>var x=1;</script>"#;
        let md = html_to_markdown(html);
        assert_eq!(md.trim(), "正文");
    }
}
