import 'package:flutter/material.dart';

import 'md.dart';

/// 桌面预览：按块模型直接构建 Widget，样式与公众号 HTML 同一套参数。
Widget buildPreview(List<Block> blocks) => ListView(
      padding: const EdgeInsets.symmetric(horizontal: 32, vertical: 24),
      children: [for (final b in blocks) _blockWidget(b)],
    );

const _textColor = Color(0xFF3F3F3F);
const _headingColor = Color(0xFF1F2937);
const _mono = 'monospace';

TextStyle _base({double size = 16, double height = 1.8, Color? color}) =>
    TextStyle(fontSize: size, height: height, color: color ?? _textColor);

TextSpan _span(List<Inline> inlines, {TextStyle? inherit}) {
  TextSpan walk(Inline run) => switch (run) {
        TextRun(:final text) => TextSpan(text: text),
        CodeRun(:final text) => TextSpan(
            text: text,
            style: TextStyle(
                fontFamily: _mono,
                fontSize: 14,
                color: const Color(0xFFC7254E),
                background: Paint()..color = const Color(0xFFF6F8F7))),
        StrongRun(:final children) => TextSpan(
            children: [walk(children.firstOrNull ?? const TextRun(''))],
            style: const TextStyle(
                fontWeight: FontWeight.w700, color: _headingColor)),
        EmRun(:final children) => TextSpan(
            children: [walk(children.firstOrNull ?? const TextRun(''))],
            style: const TextStyle(fontStyle: FontStyle.italic)),
        DelRun(:final children) => TextSpan(
            children: [walk(children.firstOrNull ?? const TextRun(''))],
            style: const TextStyle(
                color: Color(0xFF9CA3AF), decoration: TextDecoration.lineThrough)),
        LinkRun(:final text) => TextSpan(
            text: text,
            style: const TextStyle(
                color: Color(0xFF576B95), decoration: TextDecoration.underline)),
        ImageInline(:final alt) =>
          TextSpan(text: '【图片：${alt.isEmpty ? '配图' : alt}】'),
      };
  final spans = inlines.map(walk).toList();
  return TextSpan(style: inherit, children: spans);
}

Widget _blockWidget(Block block) {
  switch (block) {
    case HeadingBlock(:final level, :final inlines):
      final size = switch (level) {
        1 => 24.0,
        2 => 21.0,
        3 => 18.0,
        _ => 17.0,
      };
      return Padding(
        padding: const EdgeInsets.only(top: 28, bottom: 14),
        child: Text.rich(
          _span(inlines,
              inherit: _base(size: size, height: 1.4, color: _headingColor)
                  .copyWith(fontWeight: FontWeight.w600)),
        ),
      );
    case ParagraphBlock(:final inlines):
      return Padding(
        padding: const EdgeInsets.only(bottom: 16),
        child: Text.rich(_span(inlines),
            textAlign: TextAlign.justify),
      );
    case QuoteBlock(:final paragraphs):
      return Container(
        margin: const EdgeInsets.only(bottom: 16),
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
        decoration: const BoxDecoration(
          color: Color(0xFFF6F8F7),
          border: Border(left: BorderSide(color: Color(0xFF07C160), width: 3)),
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            for (final p in paragraphs)
              Padding(
                padding: const EdgeInsets.only(bottom: 8),
                child: Text.rich(
                  _span(p.inlines,
                      inherit: _base(size: 15, height: 1.7, color: const Color(0xFF6A737D))),
                ),
              ),
          ],
        ),
      );
    case CodeBlockText(:final text):
      return Container(
        margin: const EdgeInsets.only(bottom: 16),
        padding: const EdgeInsets.all(14),
        decoration: BoxDecoration(
          color: const Color(0xFFF6F8F7),
          borderRadius: BorderRadius.circular(6),
        ),
        child: SelectableText(text, style: _base(size: 14, height: 1.7)),
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
                      child: Text(ordered ? '${i + 1}.' : '•',
                          style: _base(height: 1.8)),
                    ),
                    Expanded(child: Text.rich(_span(items[i]))),
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
