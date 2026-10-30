/// Markdown 解析与公众号 HTML 渲染。
///
/// 解析产物为块模型（[Block]），供两处消费：
/// - HTML 渲染（[blocksToHtml]）：逐元素内联样式，与 CLI render 同规则：
///   无 <style> 块、无 class、开头 H1 不进正文
/// - 桌面预览：按块模型直接构建 Flutter Widget（见 preview_desktop.dart）
library;

import 'theme.dart';

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

// ── HTML 渲染（逐元素内联样式，样式随主题）──────────────────

String escapeHtml(String s) =>
    s.replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('>', '&gt;');

String escapeAttr(String s) => escapeHtml(s).replaceAll('"', '&quot;');

String inlineToHtml(List<Inline> inlines, ThemeStyles t) => inlines
    .map((run) => switch (run) {
          TextRun(:final text) => escapeHtml(text),
          CodeRun(:final text) =>
            '<code style="${t.codeStyle}">${escapeHtml(text)}</code>',
          StrongRun(:final children) =>
            '<strong style="${t.strongStyle}">${inlineToHtml(children, t)}</strong>',
          EmRun(:final children) =>
            '<em style="${t.emStyle}">${inlineToHtml(children, t)}</em>',
          DelRun(:final children) =>
            '<del style="${t.delStyle}">${inlineToHtml(children, t)}</del>',
          LinkRun(:final href, :final text) =>
            '<a style="${t.linkStyle}" href="${escapeAttr(href)}">${escapeHtml(text)}</a>',
          ImageInline(:final src, :final alt) =>
            '<img style="${t.imgStyle}" src="${escapeAttr(src)}" alt="${escapeAttr(alt)}" />',
        })
    .join();

String blockToHtml(Block block, ThemeStyles t) {
  return switch (block) {
    HeadingBlock(:final level, :final inlines) => () {
        final size = switch (level) {
          1 => t.h1Size,
          2 => t.h2Size,
          3 => t.h3Size,
          _ => t.h4Size,
        };
        return '<h$level style="${t.headingStyle(size)}">${inlineToHtml(inlines, t)}</h$level>';
      }(),
    ParagraphBlock(:final inlines) =>
      '<p style="${t.pStyle}">${inlineToHtml(inlines, t).replaceAll('\n', '<br/>')}</p>',
    QuoteBlock(:final paragraphs) =>
      '<blockquote style="${t.quoteStyle}">${paragraphs.map((p) => '<p style="${t.quotePStyle}">${inlineToHtml(p.inlines, t)}</p>').join()}</blockquote>',
    CodeBlockText(:final text) =>
      '<pre style="${t.preStyle}"><code style="${t.preCodeStyle}">${escapeHtml(text)}</code></pre>',
    ListBlock(:final ordered, :final items) =>
      '<${ordered ? 'ol' : 'ul'} style="${t.listStyle}">${items.map((it) => '<li style="${t.liStyle}">${inlineToHtml(it, t)}</li>').join()}</${ordered ? 'ol' : 'ul'}>',
    RuleBlock() => '<hr style="${t.hrStyle}" />',
  };
}

/// 块列表 → 正文 HTML（不含外层 section）。
String blocksToHtml(List<Block> blocks, ThemeStyles t) =>
    blocks.map((b) => blockToHtml(b, t)).join();

/// 块列表 → 完整文章片段（含外层 section）。
String articleHtml(List<Block> blocks, ThemeStyles t) =>
    '<section style="${t.wrapperStyle}">${blocksToHtml(blocks, t)}</section>';

/// 渲染整篇文章为公众号草稿 content 片段（内置默认主题）。
String renderArticle(String markdown) =>
    articleHtml(parseMarkdown(markdown), ThemeStyles());
