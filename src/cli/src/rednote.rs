//! 文章 → 小红书图集（1080×1440，3:4）。
//!
//! 路线：Markdown 块 → 确定性分页 → SVG → resvg 渲染 PNG。不背浏览器与
//! LLM 依赖：examples/rednote 的「JSON→HTML→截图」是探索原型，此处是
//! CLI 原生形态，同一目的地两条路。
//!
//! 版式规范以 dev-guide「小红书图集」节为正本：页面 1080×1440，左右边距
//! 90，正文 34/行距 1.65，标题 46，页脚 24。

use crate::render::split_title;
use anyhow::{Context, Result};
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use std::path::{Path, PathBuf};

pub const PAGE_W: u32 = 1080;
pub const PAGE_H: u32 = 1440;
const MARGIN_X: f32 = 90.0;
const TOP: f32 = 110.0;
const BOTTOM: f32 = 150.0;
const CONTENT_W: f32 = PAGE_W as f32 - MARGIN_X * 2.0;
const FONT: &str = "WenQuanYi Micro Hei";
const FONT_MONO: &str = "WenQuanYi Micro Hei Mono";
const C_TEXT: &str = "#333333";
const C_MUTED: &str = "#888888";
const C_GREEN: &str = "#07C160";
const C_SOFT: &str = "#f6f8f7";

/// 生成图集到 out_dir，返回 PNG 路径（封面在前）。
pub fn generate(markdown: &str, out_dir: &Path) -> Result<Vec<PathBuf>> {
    let (title, body) = split_title(markdown);
    let title = title.unwrap_or_else(|| "未命名".to_string());
    let blocks = extract_blocks(&body);
    let pages = paginate(&blocks);

    std::fs::create_dir_all(out_dir)
        .with_context(|| format!("创建输出目录失败：{}", out_dir.display()))?;
    let mut paths = Vec::new();
    let total = pages.len();
    for (i, page) in pages.iter().enumerate() {
        let svg = page_svg(page, &title, i + 1, total);
        let png = render_svg(&svg, i + 1)?;
        let path = out_dir.join(format!("page_{:02}.png", i + 1));
        std::fs::write(&path, png).with_context(|| format!("写入失败：{}", path.display()))?;
        paths.push(path);
    }
    Ok(paths)
}

// ── 块模型（pulldown 事件 → 简化块）───────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum RBlock {
    H(String),
    P(String),
    Bullets(Vec<String>),
    Quote(String),
    Code(String),
    Img { src: String, alt: String },
    Hr,
}

pub fn extract_blocks(markdown: &str) -> Vec<RBlock> {
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    let parser = Parser::new_ext(markdown, opts);

    let mut blocks: Vec<RBlock> = Vec::new();
    // 当前块：(种类标记, 文本累积)
    let mut cur: Option<(u8, String)> = None; // 1=H 2=P 3=Code 4=Img
    let mut quote_depth = 0usize;
    let mut quote = String::new();
    let mut list_items: Vec<String> = Vec::new();

    for ev in parser {
        match ev {
            Event::Start(Tag::Heading { .. }) => cur = Some((1, String::new())),
            Event::End(TagEnd::Heading(_)) => {
                if let Some((1, t)) = cur.take() {
                    blocks.push(RBlock::H(t.trim().to_string()));
                }
            }
            Event::Start(Tag::Paragraph) => cur = Some((2, String::new())),
            Event::End(TagEnd::Paragraph) => {
                if let Some((2, t)) = cur.take() {
                    let t = t.trim().to_string();
                    if t.is_empty() {
                        continue;
                    }
                    if quote_depth > 0 {
                        if !quote.is_empty() {
                            quote.push('\n');
                        }
                        quote.push_str(&t);
                    } else {
                        blocks.push(RBlock::P(t));
                    }
                }
            }
            Event::Start(Tag::BlockQuote(_)) => quote_depth += 1,
            Event::End(TagEnd::BlockQuote(_)) => {
                quote_depth = quote_depth.saturating_sub(1);
                if quote_depth == 0 {
                    let q = quote.trim().to_string();
                    if !q.is_empty() {
                        blocks.push(RBlock::Quote(q));
                    }
                    quote.clear();
                }
            }
            Event::Start(Tag::CodeBlock(_)) => cur = Some((3, String::new())),
            Event::End(TagEnd::CodeBlock) => {
                if let Some((3, t)) = cur.take() {
                    blocks.push(RBlock::Code(t));
                }
            }
            Event::Start(Tag::List(_)) => list_items.clear(),
            Event::End(TagEnd::List(_)) => {
                let items: Vec<String> = list_items
                    .iter()
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                if !items.is_empty() {
                    blocks.push(RBlock::Bullets(items));
                }
            }
            Event::Start(Tag::Item) => list_items.push(String::new()),
            Event::Start(Tag::Image { dest_url, .. }) => {
                cur = Some((4, dest_url.to_string()));
            }
            Event::End(TagEnd::Image) => {
                if let Some((4, raw)) = cur.take() {
                    let (src, alt) = match raw.split_once('\u{1}') {
                        Some((s, a)) => (s.to_string(), a.to_string()),
                        None => (raw, String::new()),
                    };
                    blocks.push(RBlock::Img { src, alt });
                }
            }
            Event::Rule => blocks.push(RBlock::Hr),
            Event::Text(t) | Event::Code(t) => match cur.as_mut() {
                Some((3, s)) => s.push_str(&t),
                Some((4, src)) => {
                    // 图片 alt 文本：拼在 src 后以 \u{1} 分隔
                    src.push('\u{1}');
                    src.push_str(&t);
                }
                Some((_, s)) => s.push_str(&t),
                None => {
                    if let Some(last) = list_items.last_mut() {
                        if !last.is_empty() {
                            last.push(' ');
                        }
                        last.push_str(&t);
                    }
                }
            },
            Event::SoftBreak | Event::HardBreak => match cur.as_mut() {
                Some((_, s)) => s.push('\n'),
                None => {
                    if let Some(last) = list_items.last_mut() {
                        last.push('\n');
                    }
                }
            },
            _ => {}
        }
    }
    // 未闭合的悬挂块（防御性收尾）
    if let Some((1, t)) = cur {
        blocks.push(RBlock::H(t.trim().to_string()));
    }
    blocks
}

// ── 分页 ────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Piece {
    /// 封面整页
    Title,
    Text {
        text: String,
        size: f32,
        color: &'static str,
        /// 块首行的上间距（续行恒为 0）
        gap_top: f32,
        mono: bool,
    },
    Bullet {
        text: String,
    },
    Image {
        src: String,
        alt: String,
    },
    Divider,
}

#[derive(Debug, Clone)]
pub struct Page(pub Vec<Piece>);

fn text_h(size: f32, lines: usize, gap_top: f32) -> f32 {
    gap_top + lines as f32 * size * 1.65
}

fn line_count(text: &str, size: f32, width: f32) -> usize {
    wrap(text, size, width).len()
}

/// 按字符宽度估算断行（CJK 全宽、ASCII 半宽），返回行列表。
pub fn wrap(text: &str, size: f32, width: f32) -> Vec<String> {
    let mut lines = Vec::new();
    for raw in text.split('\n') {
        let mut line = String::new();
        let mut w = 0.0f32;
        let mut last_space: Option<usize> = None;
        for ch in raw.chars() {
            let cw = char_width(ch, size);
            if w + cw > width && !line.is_empty() {
                if let Some(sp) = last_space
                    && sp > 0
                {
                    let boundary = line_char_boundary(&line, sp);
                    let rest: String = line[boundary + 1..].chars().collect();
                    lines.push(line[..boundary].trim_end().to_string());
                    line = rest;
                } else {
                    lines.push(std::mem::take(&mut line));
                }
                w = est_width(&line, size);
            }
            line.push(ch);
            w += cw;
            last_space = (ch == ' ').then_some(line.chars().count() - 1);
        }
        lines.push(line);
    }
    lines
}

fn line_char_boundary(s: &str, char_idx: usize) -> usize {
    s.char_indices()
        .nth(char_idx)
        .map(|(i, _)| i)
        .unwrap_or(s.len())
}

pub fn char_width(c: char, size: f32) -> f32 {
    let u = c as u32;
    if u >= 0x2E80 {
        size // CJK 与全角标点
    } else if c.is_ascii_alphanumeric() {
        size * 0.56
    } else if c == ' ' {
        size * 0.30
    } else {
        size * 0.50
    }
}

fn est_width(s: &str, size: f32) -> f32 {
    s.chars().map(|c| char_width(c, size)).sum()
}

/// 正文分页：封面固定第一页，其余贪心填充；标题不留在页尾。
#[allow(unused_assignments)] // 末尾 flush 后的归零无人再读
pub fn paginate(blocks: &[RBlock]) -> Vec<Page> {
    let mut pages: Vec<Page> = vec![Page(vec![Piece::Title])];
    let mut cur: Vec<Piece> = Vec::new();
    let mut used = 0.0f32;
    let avail = PAGE_H as f32 - TOP - BOTTOM;

    macro_rules! flush {
        () => {
            if !cur.is_empty() {
                pages.push(Page(std::mem::take(&mut cur)));
                used = 0.0;
            }
        };
    }

    for (i, b) in blocks.iter().enumerate() {
        // 块 → (片段, 块高)；片段首行带 gap_top，续行为 0
        let (pieces, h): (Vec<Piece>, f32) = match b {
            RBlock::H(t) => {
                let wrapped = wrap(t, 46.0, CONTENT_W);
                // 标题孤儿控制：页尾容不下「标题 + 一段」就翻页
                let next_peek = blocks.get(i + 1).map(block_height).unwrap_or(0.0);
                let need = text_h(46.0, wrapped.len(), 40.0) + 110.0f32.min(next_peek);
                let pieces = wrapped
                    .into_iter()
                    .enumerate()
                    .map(|(k, l)| Piece::Text {
                        text: l,
                        size: 46.0,
                        color: C_GREEN,
                        gap_top: if k == 0 { 40.0 } else { 0.0 },
                        mono: false,
                    })
                    .collect();
                (pieces, need)
            }
            RBlock::P(t) => {
                let wrapped = wrap(t, 34.0, CONTENT_W);
                let h = text_h(34.0, wrapped.len(), 24.0);
                let pieces = wrapped
                    .into_iter()
                    .enumerate()
                    .map(|(k, l)| Piece::Text {
                        text: l,
                        size: 34.0,
                        color: C_TEXT,
                        gap_top: if k == 0 { 24.0 } else { 0.0 },
                        mono: false,
                    })
                    .collect();
                (pieces, h)
            }
            RBlock::Bullets(items) => {
                let mut pieces = Vec::new();
                let mut h = 0.0;
                for it in items {
                    let wrapped = wrap(it, 34.0, CONTENT_W - 44.0);
                    h += text_h(34.0, wrapped.len(), 16.0);
                    for l in wrapped {
                        pieces.push(Piece::Bullet { text: l });
                    }
                }
                (pieces, h)
            }
            RBlock::Quote(t) => {
                let wrapped = wrap(t, 32.0, CONTENT_W - 24.0);
                let h = text_h(32.0, wrapped.len(), 24.0) + 16.0;
                let pieces = wrapped
                    .into_iter()
                    .enumerate()
                    .map(|(k, l)| Piece::Text {
                        text: format!("｜{l}"),
                        size: 32.0,
                        color: C_MUTED,
                        gap_top: if k == 0 { 24.0 } else { 0.0 },
                        mono: false,
                    })
                    .collect();
                (pieces, h)
            }
            RBlock::Code(t) => {
                let wrapped: Vec<String> = t
                    .lines()
                    .flat_map(|l| wrap(l, 28.0, CONTENT_W - 48.0))
                    .collect();
                let n = wrapped.len().max(1);
                let h = text_h(28.0, n, 24.0) + 32.0;
                let pieces = wrapped
                    .into_iter()
                    .enumerate()
                    .map(|(k, l)| Piece::Text {
                        text: l,
                        size: 28.0,
                        color: "#c7254e",
                        gap_top: if k == 0 { 24.0 } else { 0.0 },
                        mono: true,
                    })
                    .collect();
                (pieces, h)
            }
            RBlock::Img { src, alt } => (
                vec![Piece::Image {
                    src: src.clone(),
                    alt: alt.clone(),
                }],
                404.0,
            ),
            RBlock::Hr => (vec![Piece::Divider], 48.0),
        };

        if used + h > avail && !cur.is_empty() {
            flush!();
        }
        for p in pieces {
            let ph = match &p {
                Piece::Text { size, gap_top, .. } => text_h(*size, 1, *gap_top),
                Piece::Bullet { .. } => text_h(34.0, 1, 16.0),
                Piece::Divider => 48.0,
                Piece::Image { .. } => 404.0,
                Piece::Title => 0.0,
            };
            if used + ph > avail && !cur.is_empty() {
                flush!();
            }
            cur.push(p);
            used += ph;
        }
    }
    flush!();
    pages
}

fn block_height(b: &RBlock) -> f32 {
    match b {
        RBlock::H(t) => text_h(46.0, line_count(t, 46.0, CONTENT_W), 40.0),
        RBlock::P(t) => text_h(34.0, line_count(t, 34.0, CONTENT_W), 24.0),
        RBlock::Bullets(items) => items
            .iter()
            .map(|it| text_h(34.0, line_count(it, 34.0, CONTENT_W - 44.0), 16.0))
            .sum(),
        RBlock::Quote(t) => text_h(32.0, line_count(t, 32.0, CONTENT_W - 24.0), 24.0) + 16.0,
        RBlock::Code(t) => {
            let n: usize = t
                .lines()
                .map(|l| line_count(l, 28.0, CONTENT_W - 48.0))
                .sum::<usize>()
                .max(1);
            text_h(28.0, n, 24.0) + 32.0
        }
        RBlock::Img { .. } => 404.0,
        RBlock::Hr => 48.0,
    }
}

// ── SVG ─────────────────────────────────────────────────────────

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn text_el(x: f32, y: f32, size: f32, fill: &str, body: &str, mono: bool) -> String {
    let family = if mono { FONT_MONO } else { FONT };
    format!(
        r#"<text x="{x}" y="{y}" font-family="{family}" font-size="{size}" fill="{fill}" xml:space="preserve">{}</text>"#,
        esc(body)
    )
}

fn page_svg(page: &Page, title: &str, idx: usize, total: usize) -> String {
    let mut els = format!(r##"<rect width="{PAGE_W}" height="{PAGE_H}" fill="#ffffff"/>"##);
    let mut y = TOP;

    if matches!(page.0.as_slice(), [Piece::Title]) {
        // 封面：品牌条 + 品牌名 + 大标题 + 绿色短杠
        els.push_str(&format!(
            r#"<rect x="{MARGIN_X}" y="150" width="120" height="10" fill="{C_GREEN}"/>"#
        ));
        els.push_str(&text_el(MARGIN_X, 240.0, 36.0, C_GREEN, "量潮科技", false));
        let wrapped = wrap(title, 80.0, CONTENT_W);
        let mut ty = 520.0;
        for l in wrapped {
            els.push_str(&text_el(MARGIN_X, ty, 80.0, "#111111", &l, false));
            ty += 118.0;
        }
        els.push_str(&format!(
            r#"<rect x="{MARGIN_X}" y="{}" width="96" height="6" fill="{C_GREEN}"/>"#,
            ty + 48.0
        ));
        els.push_str(&text_el(
            MARGIN_X,
            PAGE_H as f32 - BOTTOM + 90.0,
            24.0,
            C_MUTED,
            "量潮媒体云 · 小红书图集",
            false,
        ));
    } else {
        for p in &page.0 {
            match p {
                Piece::Text {
                    text,
                    size,
                    color,
                    gap_top,
                    mono,
                } => {
                    y += gap_top;
                    els.push_str(&text_el(MARGIN_X, y + size, *size, color, text, *mono));
                    y += size * 1.65;
                }
                Piece::Bullet { text } => {
                    y += 16.0;
                    els.push_str(&text_el(MARGIN_X, y + 34.0, 34.0, C_GREEN, "•", false));
                    els.push_str(&text_el(
                        MARGIN_X + 44.0,
                        y + 34.0,
                        34.0,
                        C_TEXT,
                        text,
                        false,
                    ));
                    y += 34.0 * 1.65;
                }
                Piece::Image { src, alt } => {
                    let h = 380.0;
                    match image_data_uri(src) {
                        Some(uri) => els.push_str(&format!(
                            r#"<image x="{MARGIN_X}" y="{y}" width="{CONTENT_W}" height="{h}" preserveAspectRatio="xMidYMid meet" href="{uri}"/>"#
                        )),
                        None => {
                            els.push_str(&format!(
                                r#"<rect x="{MARGIN_X}" y="{y}" width="{CONTENT_W}" height="{h}" fill="{C_SOFT}"/>"#
                            ));
                            let caption = if alt.is_empty() {
                                "配图见原文".to_string()
                            } else {
                                format!("配图：{alt}")
                            };
                            els.push_str(&text_el(
                                MARGIN_X + (CONTENT_W - est_width(&caption, 30.0)) / 2.0,
                                y + h / 2.0,
                                30.0,
                                C_MUTED,
                                &caption,
                                false,
                            ));
                        }
                    }
                    y += h + 24.0;
                }
                Piece::Divider => {
                    els.push_str(&format!(
                        r##"<rect x="{MARGIN_X}" y="{}" width="{CONTENT_W}" height="2" fill="#e5e7eb"/>"##,
                        y + 24.0
                    ));
                    y += 48.0;
                }
                Piece::Title => {}
            }
        }
        // 页脚：分隔线 + 品牌 + 页码
        els.push_str(&format!(
            r##"<rect x="{MARGIN_X}" y="{}" width="{CONTENT_W}" height="2" fill="#e5e7eb"/>"##,
            PAGE_H as f32 - BOTTOM + 40.0
        ));
        els.push_str(&text_el(
            MARGIN_X,
            PAGE_H as f32 - BOTTOM + 90.0,
            24.0,
            C_MUTED,
            "量潮媒体云",
            false,
        ));
        let pager = format!("{idx}/{total}");
        els.push_str(&text_el(
            PAGE_W as f32 - MARGIN_X - est_width(&pager, 24.0),
            PAGE_H as f32 - BOTTOM + 90.0,
            24.0,
            C_MUTED,
            &pager,
            false,
        ));
    }

    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{PAGE_W}" height="{PAGE_H}">{els}</svg>"#
    )
}

/// 远程配图 → data URI；取不到返回 None（渲染占位块）。
fn image_data_uri(src: &str) -> Option<String> {
    if !src.starts_with("http") {
        return None;
    }
    let resp = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .ok()?
        .get(src)
        .send()
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let bytes = resp.bytes().ok()?;
    let first = *bytes.first()?;
    let mime = match first {
        0x89 => "image/png",
        0xFF => "image/jpeg",
        b'G' => "image/gif",
        b'R' => "image/webp",
        _ => return None,
    };
    use base64::Engine as _;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Some(format!("data:{mime};base64,{b64}"))
}

fn render_svg(svg: &str, idx: usize) -> Result<Vec<u8>> {
    use resvg::usvg::{self, fontdb};

    let mut db = fontdb::Database::new();
    db.load_system_fonts();
    let options = usvg::Options {
        font_family: FONT.to_string(),
        fontdb: std::sync::Arc::new(db),
        ..Default::default()
    };

    let tree = usvg::Tree::from_str(svg, &options).context("解析 SVG 失败")?;
    let mut pixmap = tiny_skia::Pixmap::new(PAGE_W, PAGE_H)
        .with_context(|| format!("分配画布失败：第 {idx} 页"))?;
    resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());
    pixmap.encode_png().context("编码 PNG 失败")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_cjk_and_ascii() {
        let lines = wrap("中文断行测试中文断行测试中文", 34.0, 100.0);
        assert!(lines.len() > 1);
        assert!(lines.iter().all(|l| est_width(l, 34.0) <= 100.0 + 34.0));
        let lines = wrap("hello world hello", 34.0, 150.0);
        assert!(lines.len() >= 2);
        assert!(lines[0].starts_with("hello"));
    }

    #[test]
    fn extract_blocks_basic() {
        let blocks = extract_blocks("## 小节\n\n一段。\n\n- 甲\n- 乙\n\n> 引用句\n");
        assert!(
            blocks
                .iter()
                .any(|b| matches!(b, RBlock::H(t) if t == "小节"))
        );
        assert!(
            blocks
                .iter()
                .any(|b| matches!(b, RBlock::P(t) if t == "一段。"))
        );
        assert!(
            blocks
                .iter()
                .any(|b| matches!(b, RBlock::Bullets(items) if items.len() == 2))
        );
        assert!(
            blocks
                .iter()
                .any(|b| matches!(b, RBlock::Quote(t) if t == "引用句"))
        );
    }

    #[test]
    fn paginate_cover_plus_content() {
        let blocks = extract_blocks(
            &"一段较长的正文内容用来撑满页面高度，测试分页是否正常翻页。\n".repeat(120),
        );
        let pages = paginate(&blocks);
        assert!(pages.len() > 1, "长文应分多页");
        assert!(matches!(pages[0].0.as_slice(), [Piece::Title]));
        assert!(pages.iter().skip(1).all(|p| !p.0.is_empty()));
    }

    #[test]
    fn svg_escapes_text() {
        assert_eq!(esc("a<b>&c"), "a&lt;b&gt;&amp;c");
    }

    #[test]
    fn image_alt_split() {
        let blocks = extract_blocks("![说明文字](https://e.com/a.png)\n");
        match &blocks[0] {
            RBlock::Img { src, alt } => {
                assert_eq!(src, "https://e.com/a.png");
                assert_eq!(alt, "说明文字");
            }
            other => panic!("应为图片块：{other:?}"),
        }
    }
}
