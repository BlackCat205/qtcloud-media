/// Markdown 解析与公众号 HTML 渲染。
///
/// 解析产物为块模型（[Block]），供两处消费：
/// - [toHtml]：逐元素内联样式的公众号 HTML（与 CLI render 同规则：
///   无 <style> 块、无 class、开头 H1 不进正文）
/// - 桌面预览：按块模型直接构建 Flutter Widget（见 preview_desktop.dart）
library;

// ── 块模型 ──────────────────────────────────────────────

sealed class Block {
  const Block();
}

class HeadingBlock extends Block {
  const HeadingBlock(this.level, this.inlines);
  final int level; // 2..4（开头 H1 已在解析时丢弃，文中 H1 归为 1）
  final List<Inline> inlines;
}

class ParagraphBlock extends Block {
  const ParagraphBlock(this.inlines);
  final List<Inline> inlines;
}

class QuoteBlock extends Block {
  const QuoteBlock(this.paragraphs);
  final List<ParagraphBlock> paragraphs;
}

class CodeBlockText extends Block {
  const CodeBlockText(this.text);
  final String text;
}

class ListBlock extends Block {
  const ListBlock(this.ordered, this.items);
  final bool ordered;
  final List<List<Inline>> items;
}

class RuleBlock extends Block {
  const RuleBlock();
}

sealed class Inline {
  const Inline();
}

class TextRun extends Inline {
  const TextRun(this.text);
  final String text;
}

class CodeRun extends Inline {
  const CodeRun(this.text);
  final String text;
}

class StrongRun extends Inline {
  const StrongRun(this.children);
  final List<Inline> children;
}

class EmRun extends Inline {
  const EmRun(this.children);
  final List<Inline> children;
}

class DelRun extends Inline {
  const DelRun(this.children);
  final List<Inline> children;
}

class LinkRun extends Inline {
  const LinkRun(this.href, this.text);
  final String href;
  final String text;
}

class ImageInline extends Inline {
  const ImageInline(this.src, this.alt);
  final String src;
  final String alt;
}

// ── 解析 ────────────────────────────────────────────────

final _inlineRe = RegExp(
    r'`([^`]+)`'
    r'|\*\*([^*]+)\*\*'
    r'|\*([^*]+)\*'
    r'|~~([^~]+)~~'
    r'|!\[([^\]]*)\]\(([^)]+)\)'
    r'|\[([^\]]+)\]\(([^)]+)\)');

List<Inline> parseInline(String s) {
  final out = <Inline>[];
  var pos = 0;
  for (final m in _inlineRe.allMatches(s)) {
    if (m.start > pos) out.add(TextRun(s.substring(pos, m.start)));
    if (m.group(1) != null) {
      out.add(CodeRun(m.group(1)!));
    } else if (m.group(2) != null) {
      out.add(StrongRun(parseInline(m.group(2)!)));
    } else if (m.group(3) != null) {
      out.add(EmRun(parseInline(m.group(3)!)));
    } else if (m.group(4) != null) {
      out.add(DelRun(parseInline(m.group(4)!)));
    } else if (m.group(5) != null) {
      out.add(ImageInline(m.group(6)!, m.group(5)!));
    } else {
      out.add(LinkRun(m.group(8)!, m.group(7)!));
    }
    pos = m.end;
  }
  if (pos < s.length) out.add(TextRun(s.substring(pos)));
  return out;
}

bool _isSpecial(String l) =>
    l.startsWith('#') ||
    l.startsWith('```') ||
    l.startsWith('- ') ||
    l.startsWith('> ') ||
    RegExp(r'^\d+\.\s').hasMatch(l) ||
    l.trim() == '---';

/// 解析 Markdown 为块模型。开头 H1 是文章标题，标题由草稿 title 字段
/// 承载，不进正文。
List<Block> parseMarkdown(String markdown) {
  final lines = markdown.replaceAll('\r\n', '\n').split('\n');
  final blocks = <Block>[];
  var i = 0;
  var firstBlock = true;

  while (i < lines.length) {
    final line = lines[i];
    if (line.trim().isEmpty) {
      i++;
      continue;
    }

    if (line.startsWith('```')) {
      final code = <String>[];
      i++;
      while (i < lines.length && !lines[i].startsWith('```')) {
        code.add(lines[i]);
        i++;
      }
      if (i < lines.length) i++;
      blocks.add(CodeBlockText(code.join('\n')));
      firstBlock = false;
      continue;
    }

    if (line.startsWith('# ')) {
      if (!firstBlock) {
        blocks.add(HeadingBlock(1, parseInline(line.substring(2))));
      }
      i++;
      firstBlock = false;
      continue;
    }

    final headingMatch = RegExp(r'^(#{1,4})\s+(.*)$').firstMatch(line);
    if (headingMatch != null) {
      blocks.add(HeadingBlock(
          headingMatch.group(1)!.length, parseInline(headingMatch.group(2)!)));
      i++;
      firstBlock = false;
      continue;
    }

    if (line.trim() == '---') {
      blocks.add(const RuleBlock());
      i++;
      firstBlock = false;
      continue;
    }

    if (line.startsWith('- ') || RegExp(r'^\d+\.\s').hasMatch(line)) {
      final ordered = RegExp(r'^\d+\.\s').hasMatch(line);
      final items = <List<Inline>>[];
      while (i < lines.length &&
          ((ordered && RegExp(r'^\d+\.\s').hasMatch(lines[i])) ||
              (!ordered && lines[i].startsWith('- ')))) {
        final m = ordered
            ? RegExp(r'^\d+\.\s(.*)$').firstMatch(lines[i])
            : RegExp(r'^-\s(.*)$').firstMatch(lines[i]);
        items.add(parseInline(m!.group(1)!));
        i++;
      }
      blocks.add(ListBlock(ordered, items));
      firstBlock = false;
      continue;
    }

    if (line.startsWith('> ')) {
      final paras = <ParagraphBlock>[];
      while (i < lines.length && lines[i].startsWith('> ')) {
        paras.add(ParagraphBlock(parseInline(lines[i].substring(2))));
        i++;
      }
      blocks.add(QuoteBlock(paras));
      firstBlock = false;
      continue;
    }

    final para = <String>[line];
    i++;
    while (i < lines.length &&
        lines[i].trim().isNotEmpty &&
        !_isSpecial(lines[i])) {
      para.add(lines[i]);
      i++;
    }
    blocks.add(ParagraphBlock(parseInline(para.join('\n'))));
    firstBlock = false;
  }
  return blocks;
}

// ── HTML 渲染（逐元素内联样式）───────────────────────────

String escapeHtml(String s) => s
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;');

String escapeAttr(String s) => escapeHtml(s).replaceAll('"', '&quot;');

const _wrapperStyle =
    'font-family:-apple-system,BlinkMacSystemFont,\'Helvetica Neue\',\'PingFang SC\',\'Microsoft YaHei\',sans-serif;'
    'font-size:16px;color:#3f3f3f;line-height:1.8;letter-spacing:0.2px;';

const _pStyle = 'margin:0 0 16px;line-height:1.8;font-size:16px;text-align:justify;';
const _h1Style =
    'color:#1f2937;font-size:24px;font-weight:600;line-height:1.4;margin:28px 0 14px;';
const _h2Style =
    'color:#1f2937;font-size:21px;font-weight:600;line-height:1.4;margin:28px 0 14px;';
const _h3Style =
    'color:#1f2937;font-size:18px;font-weight:600;line-height:1.45;margin:28px 0 14px;';
const _h4Style =
    'color:#1f2937;font-size:17px;font-weight:600;line-height:1.45;margin:28px 0 14px;';
const _quoteStyle =
    'margin:0 0 16px;padding:12px 16px;border-left:3px solid #07c160;background:#f6f8f7;color:#6a737d;';
const _quotePStyle = 'margin:0 0 8px;line-height:1.7;font-size:15px;';
const _listStyle = 'margin:0 0 16px;padding-left:22px;';
const _liStyle = 'margin:0 0 8px;line-height:1.8;';
const _strongStyle = 'color:#1f2937;font-weight:700;';
const _emStyle = 'font-style:italic;';
const _delStyle = 'color:#9ca3af;text-decoration:line-through;';
const _linkStyle = 'color:#576b95;text-decoration:none;';
const _imgStyle = 'max-width:100%;height:auto;display:block;margin:0 auto 12px;';
const _codeStyle =
    'background:#f6f8f7;color:#c7254e;font-family:SFMono-Regular,Menlo,Consolas,monospace;font-size:14px;padding:2px 5px;border-radius:4px;';
const _preStyle =
    'margin:0 0 16px;padding:14px 16px;background:#f6f8f7;border-radius:6px;overflow-x:auto;white-space:pre-wrap;word-break:break-all;';
const _preCodeStyle =
    'background:none;color:#3f3f3f;font-family:SFMono-Regular,Menlo,Consolas,monospace;font-size:14px;line-height:1.7;';
const _hrStyle = 'border:none;border-top:1px solid #e5e7eb;margin:16px 0;';

String inlineToHtml(List<Inline> inlines) => inlines.map((run) {
      return switch (run) {
        TextRun(:final text) => escapeHtml(text),
        CodeRun(:final text) =>
          '<code style="$_codeStyle">${escapeHtml(text)}</code>',
        StrongRun(:final children) =>
          '<strong style="$_strongStyle">${inlineToHtml(children)}</strong>',
        EmRun(:final children) =>
          '<em style="$_emStyle">${inlineToHtml(children)}</em>',
        DelRun(:final children) =>
          '<del style="$_delStyle">${inlineToHtml(children)}</del>',
        LinkRun(:final href, :final text) =>
          '<a style="$_linkStyle" href="${escapeAttr(href)}">${escapeHtml(text)}</a>',
        ImageInline(:final src, :final alt) =>
          '<img style="$_imgStyle" src="${escapeAttr(src)}" alt="${escapeAttr(alt)}" />',
      };
    }).join();

String blockToHtml(Block block) {
  return switch (block) {
    HeadingBlock(:final level, :final inlines) => () {
        final style = switch (level) {
          1 => _h1Style,
          2 => _h2Style,
          3 => _h3Style,
          _ => _h4Style,
        };
        return '<h$level style="$style">${inlineToHtml(inlines)}</h$level>';
      }(),
    ParagraphBlock(:final inlines) =>
      '<p style="$_pStyle">${inlineToHtml(inlines).replaceAll('\n', '<br/>')}</p>',
    QuoteBlock(:final paragraphs) =>
      '<blockquote style="$_quoteStyle">${paragraphs.map((p) => '<p style="$_quotePStyle">${inlineToHtml(p.inlines)}</p>').join()}</blockquote>',
    CodeBlockText(:final text) =>
      '<pre style="$_preStyle"><code style="$_preCodeStyle">${escapeHtml(text)}</code></pre>',
    ListBlock(:final ordered, :final items) =>
      '<${ordered ? 'ol' : 'ul'} style="$_listStyle">${items.map((it) => '<li style="$_liStyle">${inlineToHtml(it)}</li>').join()}</${ordered ? 'ol' : 'ul'}>',
    RuleBlock() => '<hr style="$_hrStyle" />',
  };
}

/// 块列表 → 正文 HTML（不含外层 section）。
String blocksToHtml(List<Block> blocks) => blocks.map(blockToHtml).join();

/// 渲染整篇文章为公众号草稿 content 片段。
String renderArticle(String markdown) =>
    '<section style="$_wrapperStyle">${blocksToHtml(parseMarkdown(markdown))}</section>';
