/// 主题样式参数。与 CLI（Rust theme.rs）同一套字段，
/// TOML 为扁平 key = value，两边读同一份文件。
library;

class ThemeStyles {
  const ThemeStyles({
    this.fontFamily =
        '-apple-system,BlinkMacSystemFont,\'Helvetica Neue\',\'PingFang SC\',\'Microsoft YaHei\',sans-serif',
    this.codeFontFamily = 'SFMono-Regular,Menlo,Consolas,monospace',
    this.fontSize = 16,
    this.lineHeight = 1.8,
    this.textColor = '#3f3f3f',
    this.paragraphSpacing = 16,
    this.headingColor = '#1f2937',
    this.headingWeight = 600,
    this.h2Size = 21,
    this.h3Size = 18,
    this.h4Size = 17,
    this.headingTopSpacing = 28,
    this.headingBottomSpacing = 14,
    this.quoteBorderColor = '#07c160',
    this.quoteBg = '#f6f8f7',
    this.quoteColor = '#6a737d',
    this.codeBg = '#f6f8f7',
    this.codeColor = '#c7254e',
    this.linkColor = '#576b95',
    this.strongColor = '#1f2937',
  });

  final String fontFamily;
  final String codeFontFamily;
  final double fontSize;
  final double lineHeight;
  final String textColor;
  final double paragraphSpacing;
  final String headingColor;
  final double headingWeight;
  final double h2Size;
  final double h3Size;
  final double h4Size;
  final double headingTopSpacing;
  final double headingBottomSpacing;
  final String quoteBorderColor;
  final String quoteBg;
  final String quoteColor;
  final String codeBg;
  final String codeColor;
  final String linkColor;
  final String strongColor;

  double get h1Size => h2Size + 3;

  /// 解析扁平 TOML（key = value）。未知键忽略，缺失键用内置默认。
  static ThemeStyles fromToml(String raw) {
    final map = <String, String>{};
    for (final line in raw.split('\n')) {
      final t = line.trim();
      if (t.isEmpty || t.startsWith('#')) continue;
      final eq = t.indexOf('=');
      if (eq <= 0) continue;
      map[t.substring(0, eq).trim()] = t
          .substring(eq + 1)
          .trim()
          .replaceAll('"', '');
    }
    double num(String key, double fallback) {
      final v = double.tryParse(map[key] ?? '');
      return v ?? fallback;
    }

    String str(String key, String fallback) => map[key] ?? fallback;
    return ThemeStyles(
      fontFamily: str('font_family', const ThemeStyles().fontFamily),
      codeFontFamily: str(
        'code_font_family',
        const ThemeStyles().codeFontFamily,
      ),
      fontSize: num('font_size', 16),
      lineHeight: num('line_height', 1.8),
      textColor: str('text_color', '#3f3f3f'),
      paragraphSpacing: num('paragraph_spacing', 16),
      headingColor: str('heading_color', '#1f2937'),
      headingWeight: num('heading_weight', 600),
      h2Size: num('h2_size', 21),
      h3Size: num('h3_size', 18),
      h4Size: num('h4_size', 17),
      headingTopSpacing: num('heading_top_spacing', 28),
      headingBottomSpacing: num('heading_bottom_spacing', 14),
      quoteBorderColor: str('quote_border_color', '#07c160'),
      quoteBg: str('quote_bg', '#f6f8f7'),
      quoteColor: str('quote_color', '#6a737d'),
      codeBg: str('code_bg', '#f6f8f7'),
      codeColor: str('code_color', '#c7254e'),
      linkColor: str('link_color', '#576b95'),
      strongColor: str('strong_color', '#1f2937'),
    );
  }

  // ── 元素样式（与 Rust render.rs 同式）───────────────────

  /// CSS 数值：整数不带小数点（24px 而非 24.0px），与 CLI 输出一致。
  static String _n(double v) =>
      v == v.roundToDouble() ? v.toInt().toString() : v.toString();

  String get wrapperStyle =>
      'font-family:$fontFamily;font-size:${_n(fontSize)}px;color:$textColor;'
      'line-height:$lineHeight;letter-spacing:0.2px;';

  String get pStyle =>
      'margin:0 0 ${_n(paragraphSpacing)}px;line-height:$lineHeight;font-size:${_n(fontSize)}px;text-align:justify;';

  String get quotePStyle => 'margin:0 0 8px;line-height:1.7;font-size:15px;';

  String headingStyle(double size) =>
      'color:$headingColor;font-size:${_n(size)}px;font-weight:${_n(headingWeight)};'
      'line-height:1.4;margin:${_n(headingTopSpacing)}px 0 ${_n(headingBottomSpacing)}px;';

  String get quoteStyle =>
      'margin:0 0 ${_n(paragraphSpacing)}px;padding:12px 16px;border-left:3px solid $quoteBorderColor;background:$quoteBg;color:$quoteColor;';

  String get listStyle =>
      'margin:0 0 ${_n(paragraphSpacing)}px;padding-left:22px;';

  String get liStyle => 'margin:0 0 8px;line-height:$lineHeight;';

  String get strongStyle => 'color:$strongColor;font-weight:700;';

  String get emStyle => 'font-style:italic;';

  String get delStyle => 'color:#9ca3af;text-decoration:line-through;';

  String get linkStyle => 'color:$linkColor;text-decoration:none;';

  String get imgStyle =>
      'max-width:100%;height:auto;display:block;margin:0 auto 12px;';

  String get codeStyle =>
      'background:$codeBg;color:$codeColor;font-family:$codeFontFamily;font-size:14px;padding:2px 5px;border-radius:4px;';

  String get preStyle =>
      'margin:0 0 ${_n(paragraphSpacing)}px;padding:14px 16px;background:$codeBg;border-radius:6px;overflow-x:auto;white-space:pre-wrap;word-break:break-all;';

  String get preCodeStyle =>
      'background:none;color:$textColor;font-family:$codeFontFamily;font-size:14px;line-height:1.7;';

  String get hrStyle =>
      'border:none;border-top:1px solid #e5e7eb;margin:${_n(paragraphSpacing)}px 0;';
}
