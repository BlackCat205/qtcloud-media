/// Markdown → 公众号草稿 HTML（逐元素内联样式）。
///
/// 与 CLI（Rust）的 render 保持同一套规则：无 <style> 块、无 class、
/// 开头 H1 不进正文（标题归草稿 title 字段）。此处为 studio 预览与
/// 复制到公众号的最小实现，覆盖常用块级与行内语法。
library;

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

String escapeHtml(String s) => s
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;');

String escapeAttr(String s) => escapeHtml(s).replaceAll('"', '&quot;');

String inline(String text) {
  var s = escapeHtml(text);
  s = s.replaceAllMapped(
      RegExp(r'`([^`]+)`'), (m) => '<code style="$_codeStyle">${m[1]}</code>');
  s = s.replaceAllMapped(RegExp(r'\*\*([^*]+)\*\*'),
      (m) => '<strong style="$_strongStyle">${m[1]}</strong>');
  s = s.replaceAllMapped(RegExp(r'\*([^*]+)\*'),
      (m) => '<em style="$_emStyle">${m[1]}</em>');
  s = s.replaceAllMapped(RegExp(r'~~([^~]+)~~'),
      (m) => '<del style="$_delStyle">${m[1]}</del>');
  s = s.replaceAllMapped(RegExp(r'!\[([^\]]*)\]\(([^)]+)\)'),
      (m) => '<img style="$_imgStyle" src="${escapeAttr(m[2]!)}" alt="${escapeAttr(m[1]!)}" />');
  s = s.replaceAllMapped(RegExp(r'\[([^\]]+)\]\(([^)]+)\)'),
      (m) => '<a style="$_linkStyle" href="${escapeAttr(m[2]!)}">${m[1]}</a>');
  return s;
}

/// 渲染整篇文章，返回可直接放进公众号草稿 content 的 HTML 片段。
String renderArticle(String markdown) {
  final lines = markdown.replaceAll('\r\n', '\n').split('\n');
  final buf = StringBuffer();
  var i = 0;
  var firstBlock = true;

  bool isSpecial(String l) =>
      l.startsWith('#') ||
      l.startsWith('```') ||
      l.startsWith('- ') ||
      l.startsWith('> ') ||
      RegExp(r'^\d+\.\s').hasMatch(l) ||
      l.trim() == '---';

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
      buf.write('<pre style="$_preStyle"><code style="$_preCodeStyle">'
          '${escapeHtml(code.join('\n'))}</code></pre>');
      firstBlock = false;
      continue;
    }

    if (line.startsWith('# ')) {
      // 开头 H1 是文章标题，标题由草稿 title 字段承载，不进正文
      if (!firstBlock) {
        buf.write(
            '<h1 style="$_h1Style">${inline(line.substring(2))}</h1>');
      }
      i++;
      firstBlock = false;
      continue;
    }

    final headingMatch = RegExp(r'^(#{2,4})\s+(.*)$').firstMatch(line);
    if (headingMatch != null) {
      final level = headingMatch.group(1)!.length;
      final style = switch (level) {
        2 => _h2Style,
        3 => _h3Style,
        _ => _h4Style,
      };
      buf.write(
          '<h$level style="$style">${inline(headingMatch.group(2)!)}</h$level>');
      i++;
      firstBlock = false;
      continue;
    }

    if (line.trim() == '---') {
      buf.write('<hr style="$_hrStyle" />');
      i++;
      firstBlock = false;
      continue;
    }

    if (line.startsWith('- ') || RegExp(r'^\d+\.\s').hasMatch(line)) {
      final ordered = RegExp(r'^\d+\.\s').hasMatch(line);
      final items = <String>[];
      while (i < lines.length &&
          ((ordered && RegExp(r'^\d+\.\s').hasMatch(lines[i])) ||
              (!ordered && lines[i].startsWith('- ')))) {
        final m = ordered
            ? RegExp(r'^\d+\.\s(.*)$').firstMatch(lines[i])
            : RegExp(r'^-\s(.*)$').firstMatch(lines[i]);
        items.add('<li style="$_liStyle">${inline(m!.group(1)!)}</li>');
        i++;
      }
      final tag = ordered ? 'ol' : 'ul';
      buf.write('<$tag style="$_listStyle">${items.join()}</$tag>');
      firstBlock = false;
      continue;
    }

    if (line.startsWith('> ')) {
      final paras = <String>[];
      while (i < lines.length && lines[i].startsWith('> ')) {
        paras.add('<p style="$_quotePStyle">${inline(lines[i].substring(2))}</p>');
        i++;
      }
      buf.write(
          '<blockquote style="$_quoteStyle">${paras.join()}</blockquote>');
      firstBlock = false;
      continue;
    }

    // 段落：收集到下一个块级边界
    final para = <String>[line];
    i++;
    while (i < lines.length && lines[i].trim().isNotEmpty && !isSpecial(lines[i])) {
      para.add(lines[i]);
      i++;
    }
    buf.write('<p style="$_pStyle">${inline(para.join('\n')).replaceAll('\n', '<br/>')}</p>');
    firstBlock = false;
  }

  return '<section style="$_wrapperStyle">$buf</section>';
}
