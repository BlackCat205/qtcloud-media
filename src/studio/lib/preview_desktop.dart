import 'package:flutter/material.dart';

import 'md.dart';
import 'theme.dart';

/// 桌面预览：按块模型直接构建 Widget，样式参数与公众号 HTML 同一份主题。
Widget buildPreview(List<Block> blocks, ThemeStyles theme) => ListView(
  padding: const EdgeInsets.symmetric(horizontal: 32, vertical: 24),
  children: [for (final b in blocks) _blockWidget(b, theme)],
);

FontWeight _weight(double w) => switch (w.round()) {
  100 => FontWeight.w100,
  200 => FontWeight.w200,
  300 => FontWeight.w300,
  400 => FontWeight.w400,
  500 => FontWeight.w500,
  700 => FontWeight.w700,
  800 => FontWeight.w800,
  900 => FontWeight.w900,
  _ => FontWeight.w600,
};

Color _hex(String s) =>
    Color(int.parse('FF${s.replaceFirst('#', '')}', radix: 16));

TextSpan _span(List<Inline> inlines, ThemeStyles t, {TextStyle? inherit}) {
  TextSpan walk(Inline run) => switch (run) {
    TextRun(:final text) => TextSpan(text: text),
    CodeRun(:final text) => TextSpan(
      text: text,
      style: TextStyle(
        fontFamily: 'monospace',
        fontSize: 14,
        color: _hex(t.codeColor),
        background: Paint()..color = _hex(t.codeBg),
      ),
    ),
    StrongRun(:final children) => TextSpan(
      children: [walk(children.firstOrNull ?? const TextRun(''))],
      style: TextStyle(fontWeight: FontWeight.w700, color: _hex(t.strongColor)),
    ),
    EmRun(:final children) => TextSpan(
      children: [walk(children.firstOrNull ?? const TextRun(''))],
      style: const TextStyle(fontStyle: FontStyle.italic),
    ),
    DelRun(:final children) => TextSpan(
      children: [walk(children.firstOrNull ?? const TextRun(''))],
      style: const TextStyle(
        color: Color(0xFF9CA3AF),
        decoration: TextDecoration.lineThrough,
      ),
    ),
    LinkRun(:final text) => TextSpan(
      text: text,
      style: TextStyle(
        color: _hex(t.linkColor),
        decoration: TextDecoration.underline,
      ),
    ),
    ImageInline(:final alt) => TextSpan(
      text: '【图片：${alt.isEmpty ? '配图' : alt}】',
    ),
  };
  final spans = inlines.map(walk).toList();
  return TextSpan(style: inherit, children: spans);
}

Widget _blockWidget(Block block, ThemeStyles t) {
  switch (block) {
    case HeadingBlock(:final level, :final inlines):
      final size = switch (level) {
        1 => t.h1Size,
        2 => t.h2Size,
        3 => t.h3Size,
        _ => t.h4Size,
      };
      return Padding(
        padding: EdgeInsets.only(
          top: t.headingTopSpacing,
          bottom: t.headingBottomSpacing,
        ),
        child: Text.rich(
          _span(
            inlines,
            t,
            inherit: TextStyle(
              fontSize: size,
              height: 1.4,
              color: _hex(t.headingColor),
              fontWeight: _weight(t.headingWeight),
            ),
          ),
        ),
      );
    case ParagraphBlock(:final inlines):
      return Padding(
        padding: EdgeInsets.only(bottom: t.paragraphSpacing),
        child: Text.rich(_span(inlines, t), textAlign: TextAlign.justify),
      );
    case QuoteBlock(:final paragraphs):
      return Container(
        margin: EdgeInsets.only(bottom: t.paragraphSpacing),
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
        decoration: BoxDecoration(
          color: _hex(t.quoteBg),
          border: Border(
            left: BorderSide(color: _hex(t.quoteBorderColor), width: 3),
          ),
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            for (final p in paragraphs)
              Padding(
                padding: const EdgeInsets.only(bottom: 8),
                child: Text.rich(
                  _span(
                    p.inlines,
                    t,
                    inherit: TextStyle(
                      fontSize: 15,
                      height: 1.7,
                      color: _hex(t.quoteColor),
                    ),
                  ),
                ),
              ),
          ],
        ),
      );
    case CodeBlockText(:final text):
      return Container(
        margin: EdgeInsets.only(bottom: t.paragraphSpacing),
        padding: const EdgeInsets.all(14),
        decoration: BoxDecoration(
          color: _hex(t.codeBg),
          borderRadius: BorderRadius.circular(6),
        ),
        child: SelectableText(
          text,
          style: TextStyle(
            fontSize: 14,
            height: 1.7,
            color: _hex(t.textColor),
            fontFamily: 'monospace',
          ),
        ),
      );
    case ListBlock(:final ordered, :final items):
      return Padding(
        padding: const EdgeInsets.only(bottom: 16, left: 6),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            for (var i = 0; i < items.length; i++)
              Padding(
                padding: const EdgeInsets.only(bottom: 8),
                child: Row(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    SizedBox(
                      width: 28,
                      child: Text(
                        ordered ? '${i + 1}.' : '•',
                        style: TextStyle(
                          fontSize: t.fontSize,
                          height: t.lineHeight,
                          color: _hex(t.textColor),
                        ),
                      ),
                    ),
                    Expanded(child: Text.rich(_span(items[i], t))),
                  ],
                ),
              ),
          ],
        ),
      );
    case RuleBlock():
      return const Padding(
        padding: EdgeInsets.symmetric(vertical: 12),
        child: Divider(color: Color(0xFFE5E7EB), height: 1),
      );
  }
}
