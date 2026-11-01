import 'package:flutter_test/flutter_test.dart';
import 'package:qtcloud_media_studio/md.dart';
import 'package:qtcloud_media_studio/theme.dart';

void main() {
  test('标题规则 1：YAML title 优先于正文 #', () {
    final a = parseArticle('---\ntitle: YAML 标题\n---\n\n正文。\n\n# 文中标题\n');
    expect(a.title, 'YAML 标题');
    final html = articleHtml(a.blocks, ThemeStyles(), title: a.title);
    expect(html, contains('YAML 标题'));
    expect(html, contains('正文。'));
    expect(html, contains('文中标题')); // body H1 保留正文地位
    // front matter 不进正文
    expect(html, isNot(contains('title:')));
  });

  test('标题规则 2：无 YAML 且唯一 H1 是标题，不进正文', () {
    final a = parseArticle('# 文章标题\n\n正文。\n');
    expect(a.title, '文章标题');
    final bodyHtml = blocksToHtml(a.blocks, ThemeStyles());
    expect(bodyHtml, isNot(contains('文章标题'))); // 正文不含标题
    final html = articleHtml(a.blocks, ThemeStyles(), title: a.title);
    expect(html, contains('>文章标题<')); // 预览顶部渲染标题
    expect(html, contains('正文。'));
  });

  test('标题规则 3：多个 H1 都不是标题，都留正文', () {
    final a = parseArticle('# 一\n\n正文。\n\n# 二\n');
    expect(a.title, isNull);
    final html = articleHtml(a.blocks, ThemeStyles());
    expect(html, contains('>一<'));
    expect(html, contains('>二<'));
    expect(html, contains('正文。'));
  });

  test('全内联样式：无 style 块、无 class', () {
    final html = renderArticle(
      '## 小节\n\n**重点** 与 `代码`\n\n```bash\necho hi\n```\n\n- 项\n\n> 引用\n',
    );
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

  test('主题 TOML 解析与应用', () {
    const toml =
        'name = "night"\ntext_color = "#111111"\nh2_size = 24\nline_height = 2.0\n';
    final t = ThemeStyles.fromToml(toml);
    expect(t.textColor, '#111111');
    expect(t.h2Size, 24);
    expect(t.lineHeight, 2.0);
    // 未写入的键回退内置默认
    expect(t.codeColor, '#c7254e');
    final html = articleHtml(parseMarkdown('## 小节\n'), t);
    expect(html, contains('color:#111111'));
    expect(html, contains('font-size:24px'));
  });
}
