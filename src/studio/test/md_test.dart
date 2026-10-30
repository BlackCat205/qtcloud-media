import 'package:flutter_test/flutter_test.dart';
import 'package:qtcloud_media_studio/md.dart';

void main() {
  test('开头 H1 不进正文，后续 H1 保留', () {
    final html = renderArticle('# 标题\n\n正文。\n\n# 文中标题\n');
    expect(html, isNot(contains('>标题<')));
    expect(html, contains('正文。'));
    expect(html, contains('文中标题'));
  });

  test('全内联样式：无 style 块、无 class', () {
    final html = renderArticle('## 小节\n\n**重点** 与 `代码`\n\n```bash\necho hi\n```\n\n- 项\n\n> 引用\n');
    expect(html, contains('<h2 style='));
    expect(html, contains('<strong style='));
    expect(html, contains('<code style='));
    expect(html, contains('<pre style='));
    expect(html, contains('<li style='));
    expect(html, contains('<blockquote style='));
    expect(html, isNot(contains('<style')));
    expect(html, isNot(contains('class=')));
  });

  test('正文一个字不改', () {
    final html = renderArticle('排版归排版，正文一个字不改。\n');
    expect(html, contains('排版归排版，正文一个字不改。'));
  });

  test('转义原始 HTML', () {
    final html = renderArticle('数学 < b > 公式\n');
    expect(html, contains('数学 &lt; b &gt; 公式'));
  });
}
